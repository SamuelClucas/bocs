use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};
use std::{num::NonZero, sync::Arc};
use crate::{world::camera::OrbitalCamera, 
    backend_admin::gpu::{enums::{Access, 
                                OffsetBehaviour}, 
                        builders::{BindGroupLayoutBuilder}}};
use anyhow::{Result};
use wgpu::{util::DeviceExt, wgt::TextureDescriptor, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BufferBinding, BufferUsages, ComputePipeline, Extent3d, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView, TextureViewDescriptor};
use rand::prelude::*;
use wgpu::TextureUsages;


#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    /// World -> Camera basis vectors, timestep, and random seed for voxel grid init
    /// Wgsl expects Vec4<f32> (16 byte alignment
    dims: [u32; 4], // i, j, k, ij plane stride for k
    cam_pos: [f32; 4], // [2]< padding
    forward: [f32; 4], // [2]< padding
    centre: [f32; 4],
    up: [f32; 4], // [2]< padding
    right: [f32; 4], // [2]< padding
    timestep: [f32; 4], // only [0]
    seed: [f32; 4], // only [0]
    flags: [u32; 4]

}

impl Uniforms {
    pub fn flatten_u8(&self) -> &[u8] {
        let ptr = self as *const _ as *const u8;

        let len = std::mem::size_of::<Uniforms>(); 
        // Each f32/u32 padded rhs with 12 bytes to 16 byte alignment
        
        unsafe {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}

struct VoxelDims {
    i: u32,
    j: u32,
    k: u32,
}

struct VoxelVertices {
    vertices: Vec<[f32; 3]>

}

impl VoxelVertices {
    pub fn  centre_at_origin(mut self, dims: &VoxelDims) -> Self{
        let i = (dims.i as f32) / 2.0;
        let j = (dims.j as f32) / 2.0;
        let k = (dims.k as f32) / 2.0;

        // rh coordinates looking down k,-ijk first (i major, k minor), bottom left, counterclockwise
        self.vertices.push([-i, -j, -k]);
        self.vertices.push([-i, j, -k]);
        self.vertices.push([i, j, -k]);
        self.vertices.push([i, -j, -k]); // face in -k ij plane 

        self.vertices.push([-i, -j, k]);
        self.vertices.push([-i, j, k]);
        self.vertices.push([i, j, k]);
        self.vertices.push([i, -j, k]); // face in k ij plane
        self
    }
}   

pub struct State {
    pub mouse_is_pressed: bool,
    pub mouse_down: Option<PhysicalPosition<f64>>,
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surf_config: wgpu::SurfaceConfiguration,
    pub is_surface_configured: bool,
    device: wgpu::Device,
    queue: wgpu::Queue,

    init_pipeline: Option<ComputePipeline>,
    laplacian_pipeline: Option<ComputePipeline>,
    raymarch_pipeline: ComputePipeline,

    uniforms_voxels_storagetexture: Option<BindGroup>,
    pub camera: OrbitalCamera,
    pipeline: Option<wgpu::RenderPipeline>,
    render_bind_group: Option<BindGroup>,
    uniform_buffer: Option<wgpu::Buffer>,
    voxel_grid_buffer_a: Option<wgpu::Buffer>,
    voxel_grid_buffer_b: Option<wgpu::Buffer>,
    compute_bind_group_layout: Option<BindGroupLayout>,
    render_bind_group_layout: Option<BindGroupLayout>,
    init_complete: bool,
    dims: VoxelDims,
    i_ceil: u32,
    j_ceil: u32,
    k_ceil: u32,
    time: std::time::Instant,
    rng: ThreadRng, // Save for field hot reinit of voxel grid
    texture_view: TextureView,
    read_a: bool,
    world_vertices: VoxelVertices,
    camera_vertices: VoxelVertices

}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let camera = OrbitalCamera::new(200.0, 0.0, 0.0, &size);

        let dims = VoxelDims {
            i: 200,
            j: 200,
            k: 200
        };

        let world_vertices = VoxelVertices{vertices: Vec::new()}.centre_at_origin(&dims);

        let i_ceil = if dims.i % 8 == 0 {
            0
        }
        else { 1};

        let j_ceil = if dims.j % 4 == 0 {
            0
        }
        else { 1 };

        let k_ceil = if dims.k % 8 == 0 {
            0
        }
        else { 1};

        // Instance == handle to GPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface == handle to window (GPU output)
        let surface = instance.create_surface(window.clone())?; // clone here otherwise surface takes ownership of window

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label: None,
            required_features: wgpu::Features::default(), //wgpu::Features::POLYGON_MODE_LINE,
            required_limits: wgpu::Limits::defaults(),
            trace: wgpu::Trace::Off,
            memory_hints: Default::default(),
        }).await?;

        // COMPUTE //
        let init = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Init"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/init.wgsl").into())
        });
        
        let voxel_grid_buffer_a = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store a"),
            size:  (std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false // see shader for init
        });

        let voxel_grid_buffer_b = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store b"),
            size:  (std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false // see shader for init
        }); 
        
        
        let mut rng = rand::rng();
        let uniforms = Uniforms {
            dims: [dims.i as u32, dims.j as u32, dims.k as u32, (dims.i * dims.j) as u32],
            cam_pos: [camera.c[0], camera.c[1], camera.c[2], 0.0 as f32],
            forward: [camera.f[0], camera.f[1], camera.f[2], 0.0 as f32],
            centre: [camera.centre[0], camera.centre[1], camera.centre[2], 0.0 as f32],
            up: [camera.u[0], camera.u[1], camera.u[2], 0.0 as f32],
            right: [camera.r[0], camera.r[1], camera.r[2], 0.0 as f32],
            timestep: [0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [rng.random::<f32>(), 0.0 as f32, 0.0 as f32, 0.0 as f32],
            flags: [1, 0, 0, 0]
        };
        
        let uni = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: uniforms.flatten_u8(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let storage_texture = device.create_texture(&TextureDescriptor{
            label: Some("Storage Texture"),
            size: Extent3d {
                width: size.width, // TODO: CREATE NEW TEXTURE ON WINDOW RESIZE
                height: size.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm]
        });

        let texture_view = storage_texture.create_view(&TextureViewDescriptor{
            label: Some("Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count:None
        });

        let compute_bind_group_layout = BindGroupLayoutBuilder::new("Compute Bind Group".to_string())
            .with_uniform_buffer(
                ShaderStages::COMPUTE, 
                OffsetBehaviour::Static)
            .with_storage_buffer(
                ShaderStages::COMPUTE, 
                OffsetBehaviour::Static, 
                Access::ReadWrite)
            .with_storage_buffer(
                ShaderStages::COMPUTE,
                OffsetBehaviour::Static,
                Access::ReadWrite)// no need to alternate read/write only on buffers a and b
            .with_storage_texture(
                ShaderStages::COMPUTE, 
                TextureFormat::Rgba8Unorm, 
                wgpu::StorageTextureAccess::WriteOnly,
            wgpu::TextureViewDimension::D2)
            .build(&device);

        let render_bind_group_layout = BindGroupLayoutBuilder::new("Render Bind Group".to_string())
                .with_storage_texture(
                    ShaderStages::FRAGMENT, 
                    TextureFormat::Rgba8Unorm, 
                    wgpu::StorageTextureAccess::ReadOnly, 
                    wgpu::TextureViewDimension::D2)
                .build(&device);

        let bind_group_descriptor = &wgpu::BindGroupDescriptor {
            label: Some("Bind group descriptor"),
            layout: &compute_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding { 
                    buffer: &uni, 
                    offset: 0, 
                    size: NonZero::new((std::mem::size_of::<Uniforms>()) as u64)
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  &voxel_grid_buffer_a,
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64)
            })
            },
            BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  &voxel_grid_buffer_b,
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64)
            })
            },
            BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&texture_view)
            }
            ]
        };

        // binding 1, assigned to index 0 in render
        let uniforms_voxels_storagetexture = device.create_bind_group(bind_group_descriptor);

        // compute pipeline setup
        let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&compute_bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        // Entry Points
        let init_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Init"),
            layout: Some(&compute_pipeline_layout),
            module: &init,
            entry_point: Some("init"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: false 
            }
        });

        let laplacian_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Laplacian"),
            layout: Some(&compute_pipeline_layout),
            module: &init,
            entry_point: Some("laplacian"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: false 
            }
        });

        let raymarch_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raymarch"),
            layout: Some(&compute_pipeline_layout),
            module: &init,
            entry_point: Some("raymarch"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: false 
            }
        });

       
        // END of COMPUTE //

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        // TEXTURES //
        let fragment = device.create_shader_module(ShaderModuleDescriptor{
            label: Some("Fragment shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/fragment.wgsl").into())
        });

        let vertex = device.create_shader_module(
            ShaderModuleDescriptor { 
                label: Some("Vertex shader module"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/vertex.wgsl").into()) 
            });


        let render_bind_group = device.create_bind_group(&BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view)
            }]
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[]
        });


        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("MeowPipeline"), 
            layout: Some(&render_pipeline_layout), 
            vertex: wgpu::VertexState{
                module: &vertex,
                entry_point: Some("main"), 
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[]
            }, 
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState {
                    count: 1, 
                    mask: !0, 
                    alpha_to_coverage_enabled: false, 
                }, 
            fragment: Some(wgpu::FragmentState { // needed to store colour data to the surface
               module: &fragment,
               entry_point: Some("main"),
               compilation_options: wgpu::PipelineCompilationOptions::default(),
               targets: &[Some(wgpu::ColorTargetState {
                    format: config.format, // format of surface
                    blend: Some(wgpu::BlendState::REPLACE), // replace old colour with new colour
                    write_mask: wgpu::ColorWrites::ALL // write to all channels
               })]
            }), 
            multiview: None, 
            cache: None, 
        });


        Ok (
            Self { 
                i_ceil: i_ceil,
                j_ceil: j_ceil,
                k_ceil: k_ceil,
                mouse_is_pressed: false,
                window, 
                device,
                queue,
                surface,
                init_pipeline: Some(init_pipeline),
                laplacian_pipeline: Some(laplacian_pipeline),
                pipeline: Some(render_pipeline),
                surf_config: config,
                is_surface_configured: false,
                mouse_down: None,
                camera: camera,
                uniforms_voxels_storagetexture: Some(uniforms_voxels_storagetexture),
                render_bind_group: Some(render_bind_group),
                voxel_grid_buffer_a: Some(voxel_grid_buffer_a),
                voxel_grid_buffer_b: Some(voxel_grid_buffer_b),
                uniform_buffer: Some(uni),
                compute_bind_group_layout: Some(compute_bind_group_layout),
                render_bind_group_layout: Some(render_bind_group_layout),
                init_complete: false,
                dims: dims,
                time: std::time::Instant::now(),
                rng: rng,
                texture_view: texture_view,
                read_a: true,
                raymarch_pipeline: raymarch_pipeline,
                world_vertices: world_vertices,
                camera_vertices: VoxelVertices{ vertices: Vec::new() }

            }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surf_config.width = width;
            self.surf_config.height = height;
            self.camera.update(None, None, None, Some(&PhysicalSize {width, height}));
            self.surface.configure(&self.device, &self.surf_config);
            self.is_surface_configured = true;
        }

        let storage_texture = 
            self.device.create_texture(&TextureDescriptor{
            label: Some("Storage Texture"),
            size: Extent3d {
                width: width, 
                height: height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm]
        });
        self.texture_view = storage_texture.create_view(&TextureViewDescriptor{
            label: Some("Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count:None
        });

        let bind_group_descriptor = &wgpu::BindGroupDescriptor {
            label: Some("Bind group descriptor"),
            layout: self.compute_bind_group_layout.as_ref().unwrap(),
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding { 
                    buffer: self.uniform_buffer.as_ref().unwrap(), 
                    offset: 0, 
                    size: NonZero::new((std::mem::size_of::<Uniforms>()) as u64)
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  self.voxel_grid_buffer_a.as_ref().unwrap(),
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * self.dims.i * self.dims.j * self.dims.k) as u64)
            })
            },
            BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  self.voxel_grid_buffer_b.as_ref().unwrap(),
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * self.dims.i * self.dims.j * self.dims.k) as u64)
            })
            },
            BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&self.texture_view)
            }
            ]
        };

        self.uniforms_voxels_storagetexture = Some(self.device.create_bind_group(bind_group_descriptor));

        self.render_bind_group = Some(self.device.create_bind_group(&BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: self.render_bind_group_layout.as_ref().unwrap(),
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&self.texture_view)
            }]
        }));

        }

    pub fn handle_key(&self, event_loop: &winit::event_loop::ActiveEventLoop, code: winit::keyboard::KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (winit::keyboard::KeyCode::Escape, true) => {
                println!("Slipping through my fingers all the time\nI try to capture every minute... \nThe feeling in it, slipping through my fingers all the time... \nUntil next time!");
                event_loop.exit()
            },
            _ => {}
        }
    }

    pub fn render(&mut self, size: Option<PhysicalSize<u32>>) -> Result<(), wgpu::SurfaceError> {
        if let Some(size) = size {

            if! self.is_surface_configured {
                self.resize(size.width, size.height); // reconfigs surface to match new size dims
            } 
            let _ = self.window.request_inner_size(size);
        }
        else {println!("No size passed to render\n") }

        // Fresh compute voxel grid coords from world -> camera
        self.camera_vertices.vertices.clear();
        for i in self.world_vertices.vertices.iter() {
            self.camera_vertices.vertices.push(self.camera.world_to_ruf(i));
        }

        // this owns the texture, wrapping it with some extra swapchain-related info
        let output = self.surface.get_current_texture()?;
        // this defines how the texture is interpreted (sampled) to produce the actual pixel outputs to the surface
        // texel -> pixel
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default()); // both associated with surface

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder")
        });

        // UPDATE TIMESTEP //
        let now = std::time::Instant::now();
        let duration = ((now - self.time).as_secs_f32()).min(0.1666666); // stability bound for 3D euler integration
        self.time = now;

        self.read_a = if self.init_complete{ !self.read_a}
        else { self.read_a};

        let uniforms = Uniforms {
            dims: [self.dims.i as u32, self.dims.j as u32, self.dims.k as u32, (self.dims.i * self.dims.j) as u32],
            cam_pos: [self.camera.c[0], self.camera.c[1], self.camera.c[2], 0.0 as f32],
            forward: [self.camera.f[0], self.camera.f[1], self.camera.f[2], 0.0 as f32],
            centre: [self.camera.centre[0], self.camera.centre[1], self.camera.centre[2], 0.0 as f32],
            up: [self.camera.u[0], self.camera.u[1], self.camera.u[2], 0.0 as f32],
            right: [self.camera.r[0], self.camera.r[1], self.camera.r[2], 0.0 as f32],
            timestep: [duration, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [0.0, 0.0 as f32, 0.0 as f32, 0.0 as f32], // could later reintroduce seed here for hot sim resizing 
            flags: [self.read_a as u32, 0, 0, 0]
        };
        let uniforms = uniforms.flatten_u8();
        self.queue.write_buffer(self.uniform_buffer.as_ref().unwrap(), 0, uniforms);

        // UPDATE TIMESTEP COMPLETE //

        if !self.init_complete {
        {   
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Init"),
                timestamp_writes: None
                });
    

            compute_pass.set_pipeline(self.init_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.uniforms_voxels_storagetexture.as_ref().unwrap(), &[]); 
            compute_pass.dispatch_workgroups((self.dims.i/8) + self.i_ceil, (self.dims.j/4) + self.j_ceil, (self.dims.k/8) + self.k_ceil);  // group size is 8 * 4 * 8 <= 256 (256, 256, 64 respective limits)
            self.init_complete = true;
        }
        }
        else {
            {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Laplacian"),
                timestamp_writes: None
                });
    

            compute_pass.set_pipeline(self.laplacian_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.uniforms_voxels_storagetexture.as_ref().unwrap(), &[]); 
            
            compute_pass.dispatch_workgroups((self.dims.i/8) + self.i_ceil, (self.dims.j/4) + self.j_ceil, (self.dims.k/8) + self.k_ceil); 
            }
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { // mutable borrow of encoder here
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { // framebuffer
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.75,
                            g: 0.75,
                            b: 0.75,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(self.pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, Some(self.render_bind_group.as_ref().unwrap()), &[]);
            render_pass.draw(0..3, 0..1);
        } // encoder borrow dropped here
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish())); // allowing encoder call here
        output.present();
    
        Ok(())

    }

}