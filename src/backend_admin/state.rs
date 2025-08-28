use winit::{dpi::{PhysicalPosition, PhysicalSize}, window::Window};
use std::{sync::Arc};
use crate::{
    backend_admin::{
        bridge::Bridge, 
        gpu::{
            compute::Compute, gfx_context::GraphicsContext, render::Render, resources::Resources}}, 
    world::{
        voxel_grid::Dims3, 
        world::{World}}
    };
use std::error::Error;

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

    pub mouse_pressed: bool,
    pub mouse_pressed_pos: Option<PhysicalPosition<f64>>,
}

impl State {
    
    pub async fn new(window: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        let mut gfx_ctx: GraphicsContext = GraphicsContext::new(window).await?;
        let dims: Dims3 = [200, 200, 200];
        // World contains voxel_grid and camera
        let world = World::new(dims, &gfx_ctx);

        // Bridge holds rand seed and maintains dispatch dims for raymarch and laplacian
        let bridge = Bridge::new(&world.voxel_grid, &gfx_ctx);

        let resources = Resources::new(&dims, &world, &bridge, &mut gfx_ctx);
        
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

                mouse_pressed: false,
                mouse_pressed_pos: None
                }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        println!("Resize called\n");
        if (width != self.gfx_ctx.surface_config.width || height != self.gfx_ctx.surface_config.height) && (width > 0 && height > 0) { 
            println!("Resize if passed\n");
            self.gfx_ctx.update_surface_config();
            self.world.camera.update(None, None, None, Some(&PhysicalSize {width, height})); // TODO: REPLACE OPTIONS WITH ENUMS

            self.bridge.update_raymarch_dispatch(self.world.bbox);

            self.resources.on_resize(&self.dims, width, height, &self.gfx_ctx, &self.world, &self.bridge);

            self.compute.on_resize(&self.dims, &self.gfx_ctx, &self.resources);

            self.render.on_resize(&self.gfx_ctx, &self.resources);
        }
    }   

    
    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let surface_texture = self.gfx_ctx.surface.get_current_texture()?;
        // this defines how the texture is interpreted (sampled) to produce the actual pixel outputs to the surface
        // texel -> pixel
        let surface_texture_view = surface_texture.texture.create_view(&wgpu::TextureViewDescriptor::default()); // both associated with surface

        self.world.generate_bb_projection(&self.gfx_ctx); 

        let mut encoder = self.gfx_ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder")
        });

        // UPDATE TIMESTEP //
        let now = std::time::Instant::now();
        let duration = ((now - self.time).as_secs_f32()).min(0.1666666); // stability bound for 3D euler integration
        // let fps = 1.0 / duration;
        //println!("fps: {}\n", fps);
        self.time = now;

        // Ping pong flag
        self.read_ping = if self.init_complete{ !self.read_ping }
        else { self.read_ping };

        // UPDATE AND WRITE NEW UNIFORMS BUFFER TO QUEUE
        self.resources.uniforms_refresh(&self.gfx_ctx, &self.read_ping, duration, self.world.bbox, &self.dims, &self.world);

        if !self.init_complete {
        {   
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Init"),
                timestamp_writes: None
                });

            // RAND SEED INIT
            compute_pass.set_pipeline(&self.compute.init_p);
            compute_pass.set_bind_group(0, &self.compute.bg, &[]); 
            let [x, y, z] = self.bridge.init_dispatch;
            compute_pass.dispatch_workgroups(x, y, z);  // group size is 8 * 4 * 8 <= 256 (256, 256, 64 respective limits)
            self.init_complete = true;

            // RAYMARCH
            compute_pass.set_pipeline(&self.compute.raymarch_p);
            compute_pass.set_bind_group(0, &self.compute.bg, &[]); 
            let [x, y, z] = self.bridge.raymarch_dispatch;
            compute_pass.dispatch_workgroups(x, y, z); 
        }
        }
        else {
            {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor{
                label: Some("Laplacian"),
                timestamp_writes: None
                });
    

            compute_pass.set_pipeline(&self.compute.laplacian_p);
            compute_pass.set_bind_group(0, &self.compute.bg, &[]); 
             let [x, y, z] = self.bridge.laplacian_dispatch;
            compute_pass.dispatch_workgroups(x, y, z);  // group size is 8 * 4 * 8 <= 256 (256, 256, 64 respective limits)
            // Raymarch
            compute_pass.set_pipeline(&self.compute.raymarch_p);
            compute_pass.set_bind_group(0, &self.compute.bg, &[]); 
            let [x, y, z] = self.bridge.raymarch_dispatch; 
            compute_pass.dispatch_workgroups(x, y, z);
            }
        }
        
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { // mutable borrow of encoder here
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment { // framebuffer
                    depth_slice: None,
                    view: &surface_texture_view,
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

            render_pass.set_pipeline(&self.render.p);
            render_pass.set_bind_group(0, Some(&self.render.bg), &[]);
            render_pass.draw(0..6, 0..1);
        } // encoder borrow dropped here
        
        // submit will accept anything that implements IntoIter
        self.gfx_ctx.queue.submit(std::iter::once(encoder.finish())); // allowing encoder call here
        surface_texture.present();
    
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