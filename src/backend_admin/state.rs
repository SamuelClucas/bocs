use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};
use core::f32;
use std::{num::NonZero, sync::Arc};
use crate::{
    backend_admin::{
        bridge::Bridge, 
        gpu::{
            builders::BindGroupLayoutBuilder, compute::Compute, gfx_context::GraphicsContext, render::Render, resources::Resources}}, 
    world::{
        camera::OrbitalCamera, 
        voxel_grid::Dims3, 
        world::World}
    };
use anyhow::{Result};
use wgpu::{wgt::TextureDescriptor, BindGroup, BindGroupEntry, BindGroupLayout, BufferBinding, ComputePipeline, Extent3d, PipelineCompilationOptions, PipelineLayoutDescriptor, ShaderModuleDescriptor, ShaderStages, TextureFormat, TextureView, TextureViewDescriptor};

use std::error::Error;
use wgpu::TextureUsages;



pub struct State {
    pub gfx_ctx: GraphicsContext,
    pub world: World,
    bridge: Bridge,
    resources: Resources,
    compute: Compute,
    render: Render,

    dims: Dims3,
    init_complete: bool,
    read_ping: bool,
    time: std::time::Instant,

    pub mouse_is_pressed: bool,
    pub mouse_down: Option<PhysicalPosition<f64>>,
}

impl State {
    
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let mut gfx_ctx: = GraphicsContext::new(window).await?;
        let dims: Dims3 = [200, 200, 200];
        // World contains voxel_grid and camera
        let world = World::new(dims, &gfx_ctx);

        // Bridge holds rand seed and maintains dispatch dims for raymarch and laplacian
        let bridge = Bridge::new(&world.voxel_grid, &gfx_ctx);

        let resources = Resources::new(&dims, &world, &bridge, &mut gfx_ctx)?;
        
        let compute = Compute::new(&dims, &resources, &gfx_ctx);
        
        let render = Render::new(&resources, &gfx_ctx);
        
        Ok (
            Self { 
                gfx_ctx: gfx_ctx,
                world: world,
                bridge: bridge,
                resources: resources,
                compute: compute,
                render: render,

                init_complete: false,
                read_ping: true,
                dims: dims,
                time: std::time::Instant::now(),

                mouse_is_pressed: false,
                mouse_down: None
                }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        println!("Resize called\n");
        if width != self.gfx_ctx.surface_config.width || height != self.gfx_ctx.surface_config.height && width > 0 && height > 0 { 
            println!("Resize if passed\n");
            self.gfx_ctx.update_surface_config();

            self.world.camera.update(None, None, None, Some(&PhysicalSize {width, height})); // TODO: REPLACE OPTIONS WITH ENUMS

            self.resources.on_resize(&self.dims, width, height, &self.gfx_ctx, &self.world, &self.bridge);

            self.compute.on_resize(&self.dims, &self.gfx_ctx, &self.resources);

            self.render.on_resize(&self.gfx_ctx, &self.resources);
        }
    }   

    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.world.generate_bb_projection(&self.gfx_ctx);

     
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