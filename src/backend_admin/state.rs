use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};
use core::f32;
use std::{num::NonZero, sync::Arc};
use crate::{world::{camera::OrbitalCamera, voxel_grid::*}, 
    backend_admin::gpu::{enums::{Access, 
                                OffsetBehaviour}, 
                        builders::{BindGroupLayoutBuilder}}};
use anyhow::{Result};
use wgpu::{util::DeviceExt, wgt::TextureDescriptor, BindGroup, BindGroupEntry, BindGroupLayout, BufferBinding, BufferUsages, ComputePipeline, Extent3d, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView, TextureViewDescriptor};
use rand::prelude::*;
use wgpu::TextureUsages;


#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
    /// World -> Camera basis vectors, timestep, and random seed for voxel grid init
    /// Wgsl expects Vec4<f32> (16 byte alignment
    window_dims: [u32; 4],
    dims: [u32; 4], // i, j, k, ij plane stride for k
    bounding_box: [i32; 4],
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

pub struct State {
    pub mouse_is_pressed: bool,
    pub mouse_down: Option<PhysicalPosition<f64>>,
    pub window: Arc<Window>,

    init_pipeline: Option<ComputePipeline>,
    laplacian_pipeline: Option<ComputePipeline>,
    raymarch_pipeline: ComputePipeline,

    resources: Option<BindGroup>,
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
    voxelgrid_vertices: VoxelVertices,
    w_ceil: i32,
    h_ceil: i32,
    raymarch_group: i32,
    sampler: wgpu::Sampler
}

impl State {
    /// Determines dispatch dims on each render() 
    pub fn update_raygroup_ceil(&mut self, bounding_box: [i32; 4]) -> (u32, u32) {
        let width = bounding_box[2] - bounding_box[0];  
        let height = bounding_box[3] - bounding_box[1];
        self.w_ceil = if width % self.raymarch_group == 0 { 0 }
            else { 1 };
        self.h_ceil = if height % self.raymarch_group == 0 { 0 } 
            else { 1 };
        
        (
            ((width / self.raymarch_group) + self.w_ceil) as u32, 
            ((height / self.raymarch_group) + self.h_ceil) as u32
        )
    }
    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> Result<Self> {
        let raymarch_group = 16;
        let w_ceil= 0; // updated on each pass in render()
        let h_ceil= 0;

        let camera = OrbitalCamera::new(200.0, 0.0, 0.0, &size);

        let dims = VoxelDims {
            i: 200,
            j: 200,
            k: 200
        };

        let voxelgrid_vertices = VoxelVertices::centre_at_origin(&dims);

        let i_ceil = if dims.i % 8 == 0 { 0 }
        else { 1 };
        let j_ceil = if dims.j % 4 == 0 { 0 }
        else { 1 };
        let k_ceil = if dims.k % 8 == 0 { 0 }
        else { 1};

        


        

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
            window_dims: [size.width/2, size.height/2, 0, 0],
            dims: [dims.i as u32, dims.j as u32, dims.k as u32, (dims.i * dims.j) as u32],
            bounding_box: [0, 0, 0, 0], // set in render() 
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
                width: size.width, 
                height: size.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
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
                Access::ReadWrite)
            .with_storage_texture(
                ShaderStages::COMPUTE, 
                TextureFormat::Rgba8Unorm, 
                wgpu::StorageTextureAccess::WriteOnly,
            wgpu::TextureViewDimension::D2)
            .build(&device);

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor{
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None
        });

        
        let render_bind_group_layout = BindGroupLayoutBuilder::new("Render Bind Group".to_string())
                .with_sampler(ShaderStages::FRAGMENT)
                .with_sampled_texture(ShaderStages::FRAGMENT)
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
        let resources = device.create_bind_group(bind_group_descriptor);

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
                zero_initialize_workgroup_memory: true
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
                zero_initialize_workgroup_memory: true 
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
                zero_initialize_workgroup_memory: true 
            }
        });
        
       
        // END of COMPUTE //

        
         
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
        
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: &render_bind_group_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&sampler)},

                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view)
                }
            ]
        });

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&render_bind_group_layout],
            push_constant_ranges: &[]
        });


        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Render Pipeline"), 
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
                    format: surface_config.format, // format of surface
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
                surf_config: surface_config,
                is_surface_configured: true,
                mouse_down: None,
                camera: camera,
                resources: Some(resources),
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
                voxelgrid_vertices: voxelgrid_vertices,
                raymarch_group: raymarch_group,
                w_ceil: w_ceil,
                h_ceil: h_ceil,
                sampler
                }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width != self.surf_config.width || height != self.surf_config.height { 
            println!("Resize called\n");
            self.surf_config.width = width;
            self.surf_config.height = height;
            self.camera.update(None, None, None, Some(&PhysicalSize {width, height})); // TODO: REPLACE OPTIONS WITH ENUMS
            self.surface.configure(&self.device, &self.surf_config);
            self.is_surface_configured = true;
        
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
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
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

        if let (Some(uniforms), 
            Some(grid_a), 
            Some(grid_b), 
            Some(compute_bg_layout),
            Some(render_bg_layout)) = (
                &self.uniform_buffer, 
                &self.voxel_grid_buffer_a, 
                &self.voxel_grid_buffer_b, 
                &self.compute_bind_group_layout,
                &self.render_bind_group_layout) {

        let bind_group_descriptor = &wgpu::BindGroupDescriptor {
            label: Some("Bind group descriptor"),
            layout: compute_bg_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding { 
                    buffer: uniforms, 
                    offset: 0, 
                    size: NonZero::new((std::mem::size_of::<Uniforms>()) as u64)
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  grid_a,
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * self.dims.i * self.dims.j * self.dims.k) as u64)
            })
            },
            BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  grid_b,
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

        self.resources = Some(self.device.create_bind_group(bind_group_descriptor));
 
        self.render_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: render_bg_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&self.sampler)},

                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.texture_view)
                }
            ]
            }));
        }

        }
    }   

    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // Pixel into world units
        let centre_top_d = self.surf_config.height as f32 / 2.0; // 1:1 vertical pixels and up vector
        let right_scale = self.surf_config.width as f32 / 2.0 / centre_top_d; // garantees FOV 90 in vertical
        println!("Right scale: {}", right_scale);
        let centre_right_d = centre_top_d * right_scale;
        
        let bounding_box = {
            let mut max_r = f32::NEG_INFINITY;
            let mut max_u = f32::NEG_INFINITY;
            let mut min_r = f32::INFINITY;
            let mut min_u = f32::INFINITY;

            for i in 0..8 {
                // VOXEL VERTICES INTO RUF
                match self.voxelgrid_vertices.get_point(i, SystemGet::WORLD) {
                    SystemSet::WORLD(point) => {
                        let ruf_point = self.camera.world_to_ruf(&point);
                        self.voxelgrid_vertices.set_point(i, SystemSet::RUF(ruf_point))
                    },
                    _ => { println!("Couldn't get voxelgrid WORLD vertex.\n"); } }
                // PROJECT ONTO NEAR PLANE
                match self.voxelgrid_vertices.get_point(i, SystemGet::RUF) {
                    SystemSet::RUF(point) => {
                        let projection = self.camera.ruf_to_ru_plane(&point, &right_scale);
                        self.voxelgrid_vertices.set_point(i, SystemSet::PLANE(projection));
                    },
                    _ => { println!("Couldn't get voxelgrid RUF vertex.\n"); }
                }
                // COMPUTE BOUNDING BOX
                match self.voxelgrid_vertices.get_point(i, SystemGet::PLANE) {
                    SystemSet::PLANE(point) => {
                        max_r = point[0].max(max_r);
                        max_u = point[1].max(max_u);
                        min_r = point[0].min(min_r);
                        min_u = point[1].min(min_u);
                    },
                    _ => { println!("Couldn't get voxelgrid PLANE vertex.\n"); }
                }
            }
            [(min_r - 1.0).max(-centre_right_d) as i32, 
            (min_u - 1.0).max(-centre_top_d) as i32, 
            (max_r + 1.0).min(centre_right_d) as i32, 
            (max_u + 1.0).min(centre_top_d) as i32]
        };

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
        // let fps = 1.0 / duration;
        //println!("fps: {}\n", fps);
        self.time = now;

        self.read_a = if self.init_complete{ !self.read_a}
        else { self.read_a};

        let uniforms = Uniforms {
            window_dims: [self.surf_config.width/2, self.surf_config.height/2, 0, 0],
            dims: [self.dims.i as u32, self.dims.j as u32, self.dims.k as u32, (self.dims.i * self.dims.j) as u32],
            bounding_box: [bounding_box[0], bounding_box[1], bounding_box[2], bounding_box[3]],
            cam_pos: [self.camera.c[0], self.camera.c[1], self.camera.c[2], 0.0 as f32],
            forward: [self.camera.f[0], self.camera.f[1], self.camera.f[2], 0.0 as f32],
            centre: [self.camera.centre[0], self.camera.centre[1], self.camera.centre[2], 0.0 as f32],
            up: [self.camera.u[0], self.camera.u[1], self.camera.u[2], 0.0 as f32],
            right: [self.camera.r[0], self.camera.r[1], self.camera.r[2], right_scale],
            timestep: [duration, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [0.0, 0.0 as f32, 0.0 as f32, 0.0 as f32], // could later reintroduce seed here for hot sim resizing 
            flags: [self.read_a as u32, 0, 0, 0]
        };

        let uniforms = uniforms.flatten_u8();
        self.queue.write_buffer(self.uniform_buffer.as_ref().unwrap(), 0, uniforms);

        // UPDATE TIMESTEP COMPLETE //

        if !self.init_complete {
        {   // INIT
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Init"),
                timestamp_writes: None
                });

            compute_pass.set_pipeline(self.init_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.resources.as_ref().unwrap(), &[]); 
            compute_pass.dispatch_workgroups((self.dims.i/8) + self.i_ceil, (self.dims.j/4) + self.j_ceil, (self.dims.k/8) + self.k_ceil);  // group size is 8 * 4 * 8 <= 256 (256, 256, 64 respective limits)
            self.init_complete = true;
            // Raymarch
            compute_pass.set_pipeline(&self.raymarch_pipeline);
            compute_pass.set_bind_group(0, self.resources.as_ref().unwrap(), &[]); 
            let (dispatch_x, dispatch_y) = self.update_raygroup_ceil(bounding_box);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1); 
        }
        }
        else {
            {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Laplacian"),
                timestamp_writes: None
                });
    

            compute_pass.set_pipeline(self.laplacian_pipeline.as_ref().unwrap());
            compute_pass.set_bind_group(0, self.resources.as_ref().unwrap(), &[]); 
            compute_pass.dispatch_workgroups((self.dims.i/8) + self.i_ceil, (self.dims.j/4) + self.j_ceil, (self.dims.k/8) + self.k_ceil);  // group size is 8 * 4 * 8 <= 256 (256, 256, 64 respective limits)
            // Raymarch
            compute_pass.set_pipeline(&self.raymarch_pipeline);
            compute_pass.set_bind_group(0, self.resources.as_ref().unwrap(), &[]); 
            let (dispatch_x, dispatch_y) = self.update_raygroup_ceil(bounding_box);
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, 1);
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
            render_pass.draw(0..6, 0..1);
        } // encoder borrow dropped here
        
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish())); // allowing encoder call here
        output.present();
    
        Ok(())

    }

    pub fn handle_key(&self, event_loop: &winit::event_loop::ActiveEventLoop, code: winit::keyboard::KeyCode, is_pressed: bool) {
        match (code, is_pressed) {
            (winit::keyboard::KeyCode::Escape, true) => {
                event_loop.exit()
            },
            _ => {}
        }
    
    }
}