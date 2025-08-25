use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};
use core::f32;
use std::{num::NonZero, sync::Arc};
use crate::{
    world::{world::World, camera::OrbitalCamera, voxel_grid::{Dims3, VoxelGrid}}, 
    backend_admin::{
        bridge::Bridge, 
        gpu::{
            enums::{Access, OffsetBehaviour}, 
            builders::{BindGroupLayoutBuilder},
            gfx_context::GraphicsContext}}
    };
use anyhow::{Result};
use wgpu::{wgt::TextureDescriptor, BindGroup, BindGroupEntry, BindGroupLayout, BufferBinding, ComputePipeline, Extent3d, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView, TextureViewDescriptor};

use std::error::Error;
use wgpu::TextureUsages;



pub struct State {
    gfx_ctx: GraphicsContext,
    world: World,
    bridge: Bridge,
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
    
    pub async fn new(window: Arc<Window>, size: PhysicalSize<u32>) -> Result<Self, Box<dyn Error>> {
        let gfx_context = GraphicsContext::new(window).await?;
        let dims: Dims3 = [200, 200, 200];
        // World contains voxel_grid and camera
        let world = World::new(dims, &gfx_context);

        // Bridge holds rand seed and maintains dispatch dims for raymarch and laplacian
        let bridge = Bridge::new(&world.voxel_grid, &gfx_context);
        

        // COMPUTE //
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
                gfx_context: gfx_context,
                world: world,
                bridge: bridge,

                mouse_is_pressed: false,

                window, 

                init_pipeline: Some(init_pipeline),
                laplacian_pipeline: Some(laplacian_pipeline),
                pipeline: Some(render_pipeline),
            
                mouse_down: None,
      
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


                read_a: true,
                raymarch_pipeline: raymarch_pipeline,

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
        self.world.generate_bb_projection(&self.gfx_ctx);

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
        self.gfx_ctx.queue.write_buffer(self.uniform_buffer.as_ref().unwrap(), 0, uniforms);

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
            let [dispatch_x, dispatch_y, z] = self.bridge.raymarch_dispatch;
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, z); 
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
            let [dispatch_x, dispatch_y, z] = self.bridge.raymarch_dispatch;
            compute_pass.dispatch_workgroups(dispatch_x, dispatch_y, z);
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
        self.gfx_ctx.queue.submit(std::iter::once(encoder.finish())); // allowing encoder call here
        self.gfx_ctx.surface_texture.present();
    
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