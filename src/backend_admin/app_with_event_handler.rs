use std::num::NonZeroU32;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wgpu::{Instance, InstanceDescriptor};
use wgpu::{RequestAdapterOptions, PowerPreference, DeviceDescriptor, Device, Queue, Features, FeaturesWGPU, FeaturesWebGPU, Limits, MemoryHints, Trace} ;
use tokio::{spawn};
use wgpu::{Surface, CommandEncoderDescriptor, 
    RenderPassDescriptor, 
    RenderPassColorAttachment, 
    Operations, LoadOp, StoreOp,
    SurfaceConfiguration,TextureUsages, CompositeAlphaMode};

/// Used to match user event proxies in App::user_event
pub enum Outcome {
    ADAPTER((Device, Queue)),
    DEVICE_READY
}

/// Setup for logical App struct \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n
#[derive(Default)]
pub struct App {
    setup: bool,
    graphics_instance: Option<Instance>,
    surface_configuration: Option<SurfaceConfiguration>,
    proxy: Option<EventLoopProxy<Outcome>>,
    window: Option<Window>,
    scale_factor: Option<f64>,
   device: Option<Device>,
    queue: Option<Queue>,
}

impl App  {
    pub fn new (fun: impl FnOnce()-> EventLoopProxy<Outcome>) -> App {
        App {
            setup: true,
            graphics_instance: None,
            surface_configuration: None,
            proxy:  Some(fun()), // smuggle proxy into app using move closure for downstream requests
            window: None,
            scale_factor: None,
            device: None,
            queue: None
        }
    }
    /// Updates physical size of self.surface_configuration \n
    /// Should be called whenever DPI scale factor changes,\n
    /// Or whenever the window is resized \n
    pub fn reconfigure_surface(&self) -> Option<Surface> {
        if let (Some(dev), Some(surf_conf), Some(win), Some(graph_inst)) = (self.device.as_ref(), self.surface_configuration.as_ref(), self.window.as_ref(), self.graphics_instance.as_ref()) {
            let surface: Surface = self.create_surface().expect("mesg");
           // .configure(dev, surf_conf));
            surface.configure(dev, surf_conf);
            Some(surface)
        }
        else {
            println!("\x1b[1;31mSurface reconfigure failed. Either device, surface configuration, window, or graphics instance is None\x1b[0m\n");
            None
        }
        }

    /// Utility that returns a surface using the instance, window\n
    /// configuration should be kept up to date at all times, but that is not handled by this function\n
    /// see App::reconfigure_surface() to keep surface parameters up to date
    pub fn create_surface(&self) -> Option<Surface> {
        if let (Some(window), Some(instance)) = (self.window.as_ref(), self.graphics_instance.as_ref()) {
            Some(instance.create_surface(window).expect("\x1b[1;31mError creating surface\x1b[0m\n"))
        }
        else {
            println!("\x1b[1;31mNo surface could be made, meaning window or instance is None\x1b[0m\n");
            None
        }
    }
}

/// implements ApplicationHandler for logical App
impl ApplicationHandler<Outcome> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // ran once on init
        if self.setup == true {
            self.setup = false;
            // window: logical representation of gpu output, managed by system os
            let window_attributes = Window::default_attributes()
                    .with_title("📦")
                    .with_blur(true);
            self.window = Some(event_loop.create_window(window_attributes).expect("\x1b[1;31mError creating window\x1b[0m\n"));
            self.scale_factor = Some(self.window.as_ref().unwrap().scale_factor());
            // Infos for surface config 
            let window_size = self.window.as_ref().unwrap().inner_size();
       
            // This line is untested!!
            let inst_descriptor = InstanceDescriptor { backends: (wgpu::Backends::METAL), ..InstanceDescriptor::from_env_or_default()}; 
            self.graphics_instance = Some(Instance::new(&inst_descriptor));
            
            // TODO: create surface function using instance, window, and surface configuration
            self.surface_configuration = Some(SurfaceConfiguration {
                width: NonZeroU32::new(window_size.width).unwrap().into(),
                height: NonZeroU32::new(window_size.height).unwrap().into(),
                usage: TextureUsages::RENDER_ATTACHMENT, // can be used as render pass output
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                present_mode: wgpu::PresentMode::Fifo, // vsync! cool right?!
                desired_maximum_frame_latency: 2,
                view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb;0],
                alpha_mode: CompositeAlphaMode::Opaque // later change for transparency
            });
            let surface = self.create_surface(); // used to wrap window
            
            let request_adapter_options = RequestAdapterOptions { // see here for an explanation of each option :) https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions
                power_preference: PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: surface.as_ref()
            };
            
            // device descriptor and request (device, queue)
            let desc = DeviceDescriptor { 
                required_features: Features {features_wgpu: FeaturesWGPU::empty(),
                                            features_webgpu: FeaturesWebGPU::DEPTH_CLIP_CONTROL}, // only request what you require (see about OptionalCapabilities - limits and features - here: https://gpuweb.github.io/gpuweb/#feature also https://gpuweb.github.io/gpuweb/#feature-index)
                required_limits: Limits::downlevel_defaults(),
                trace: Trace::Off,
                memory_hints: MemoryHints::Performance,
                label: Some("wan adapter")

            }; // see here for dev desc debrief: https://gpuweb.github.io/gpuweb/#dictdef-gpudevicedescriptor
            let adapter_request = self.graphics_instance.as_ref().unwrap().request_adapter(&request_adapter_options);
            
            //  cannot call await in App due to ApplicationHandler constraint 
            // for this reason I will spawn a task using a tokio runtime, use a user event proxy to get dev and queue into app
            let prx = self.proxy.clone().take(); // deep copy
            spawn(async move {
                // now await adapter
                println!("\x1b[0;33mAwaiting adapter...\n\x1b[0m"); // in Rust, escape code for CSI isn't \033, it's \x1b. reset with [0m in case errors 
                let adapter_wan = adapter_request.await.expect("\x1b[1;31mCouldn't fetch adapter\x1b[0m\n");
                println!("\x1b[0;33mAwaiting device...\x1b[0m\n");
                let dev_req = adapter_wan.request_device(&desc).await.expect("\x1b[1;31mDevice request failed\x1b[0m\n"); // logical *handle* for physical GPU!
                let _ = prx.unwrap().send_event(Outcome::ADAPTER(dev_req));
            });
        } // end of setup: go to App::user_event() :)
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Outcome) {
            match event {
                Outcome::ADAPTER(adapter_wan) => {
                    self.device = Some(adapter_wan.0);
                    self.queue = Some(adapter_wan.1);
                    println!("\x1b[1;32mDevice and Queue ready! \x1b[0m\n");
                    let _ = self.proxy.clone().unwrap().send_event(Outcome::DEVICE_READY);
                }, 
                Outcome::DEVICE_READY => {
                    // check device is ready, redundant error catch 
                    println!("\x1b[1;32mDevice injected successfully into App!\x1b[0m\n");
                    let comm_enc_desc = CommandEncoderDescriptor::default();
                    if let Some(dev) = &self.device {
                        // this specifies which swapchain buffer (texture) to render to with target
                        let surface = self.create_surface().expect("\x1b[1;31mWarning: Failed to create surface in Outcome::DEVICE_READY\x1b[0m\n");
                        let current_texture = surface.get_current_texture().expect("\x1b[1;31mWarning: Failed to get current texture from surface in Outcome::DEVICE_READY\x1b[0m\n");
                        let col_attach: [Option<RenderPassColorAttachment<'_>>;1] = [Some(RenderPassColorAttachment {
                            view:,
                            depth_slice: None, // None for now, but plan to extend to 3D
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(wgpu::Color::GREEN), // loads green buff
                                store: StoreOp::Store // presents to gpu output
                            },
                        }) ];

                        let comm_encoder = dev.create_command_encoder(&comm_enc_desc);

                        let render_pass_desc = RenderPassDescriptor {
                            label: Some("Validate me."),
                            color_attachments: &col_attach,
                            depth_stencil_attachment: None,
                            occlusion_query_set: None,
                            timestamp_writes: None
                        };
                        // begin render pass
                        /// TODO: configure surface (get inner window size updates through RedrawRequested), getCurrentTexture
                        /// use that as target in color attachment, begin render pass (with rpdesc),
                        /// finish rpass, call Queue::Submit, then you must call SurfaceTexture::present

                    }
                }
            }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Slipping through my fingers all the time\nI try to capture every minute... \nThe feeling in it, slipping through my fingers all the time... \nUntil next time!");
                event_loop.exit();
            },
            WindowEvent::ScaleFactorChanged { .. } => {
                // Store scale factor for dynamic DPI-aware resizing for conversion into virtual size (i.e., not physical pixels)
                self.scale_factor = Some(self.window.as_ref().unwrap().scale_factor()); 
                if let (Some(config), Some(window)) = (self.surface_configuration.as_mut(), self.window.as_ref()) {
                    let size = window.inner_size();
                    let (width, height) = (size.width, size.height);
                    config.width = (width as f64) as u32; // conversion to logical size
                    config.height = (height as f64) as u32; 
                    self.reconfigure_surface();
                } else {
                    eprintln!("\x1b[1;31mWarning: Missing scale factor or surface config during resize.\x1b[0m\n");
                }
                // call reconfig
            },
            
            WindowEvent::Resized(phys) => {
                if let Some(config) = self.surface_configuration.as_mut() {
                    config.width = phys.width; // surface config should always be in physical size, no scale factor use
                    config.height = phys.height; 
                    let _ = self.reconfigure_surface();
                } else {
                    eprintln!("\x1b[1;31mWarning: Missing scale factor or surface config during resize.\x1b[0m\n");
                }
            
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            },
            WindowEvent::RedrawRequested => {
                // Redraw the application.
                //
                // It's preferable for applications that do not render continuously to render in
                // this event rather than in AboutToWait, since rendering in here allows
                // the program to gracefully handle redraws requested by the OS.

                // Draw.

                // Queue a RedrawRequested event.
                //
                // You only need to call this if you've determined that you need to redraw in
                // applications which do not always need to. Applications that redraw continuously
                // can render here instead.
                self.window.as_ref().unwrap().request_redraw();
            },
            _ => {()},
        }
    }
}
