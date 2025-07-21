use winit::window::Window;
use std::sync::Arc;
use anyhow::{Result, Context};


pub struct State {
    pub window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    surf_config: wgpu::SurfaceConfiguration,
    is_surface_configured: bool,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pub scale_factor: Option<f64>

}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

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
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::defaults(),
            trace: wgpu::Trace::Off,
            memory_hints: Default::default(),
        }).await?;

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

        let scale_factor = Some(window.as_ref().scale_factor()); 

        //surface.configure(&device, &config);

        Ok (
            Self { 
                window, 
                device,
                queue,
                surface,
                scale_factor,
                surf_config: config,
                is_surface_configured: false,
            }
        )
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surf_config.width = width;
            self.surf_config.height = height;
            self.surface.configure(&self.device, &self.surf_config);
            self.is_surface_configured = true;
        }
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

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if!self.is_surface_configured {
            self.surface.configure(&self.device, &self.surf_config);
        } 
        // this owns the texture, wrapping it with some extra swapchain-related info
        let output = self.surface.get_current_texture()?;
        // this defines how the texture is interpreted (sampled) to produce the actual pixel outputs to the surface
        // texel -> pixel
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default()); // both associated with surface

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Command Encoder")
        });
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor { // mutable borrow of encoder here
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    depth_slice: None,
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        } // encoder borrow dropped here
    
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish())); // allowing encoder call here
        output.present();
    
        Ok(())

    }
     

}