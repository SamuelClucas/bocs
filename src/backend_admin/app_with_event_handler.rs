use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window, WindowId};
use wgpu::{Instance, InstanceDescriptor};
use wgpu::{RequestAdapterOptions, PowerPreference, DeviceDescriptor, Device, Queue, Features, FeaturesWGPU, FeaturesWebGPU, Limits, MemoryHints, Trace} ;
use tokio::{spawn};


/// Used to match user event proxies in App::user_event
pub enum Outcome {
    ADAPTER((Device, Queue))
}

/// Setup for logical App struct \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n
#[derive(Default)]
pub struct App {
    setup: bool,
    proxy: Option<EventLoopProxy<Outcome>>,
    window: Option<Window>, 
   device: Option<Device>,
    queue: Option<Queue>,
}

impl  App  {
    pub fn new (prox: EventLoopProxy<Outcome>) -> App {
        App  {
            setup: true,
            proxy:  Some(prox), // smuggle proxy into app for downstream requests
            window: None,
            device: None,
            queue: None
        }
    }
}
/// implements ApplicationHandler for logical App
impl ApplicationHandler<Outcome> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if (self.setup) {
            let window_attributes = Window::default_attributes()
                    .with_title("📦")
                    .with_blur(true);
            self.window = Some(event_loop.create_window(window_attributes).expect("\x1b[1;31mError creating window\x1b[0m\n"));
        // neither of these worked    let inst_descriptor = InstanceDescriptor::from_env_or_default().with_env();
            //let inst_descriptor = InstanceDescriptor::with_env(self);
            let inst_descriptor = InstanceDescriptor { backends: (wgpu::Backends::METAL), ..Default::default()};
            let graphics_instance = Instance::new(&inst_descriptor);

            let surf = graphics_instance.create_surface(self.window.as_ref().unwrap()).expect("\x1b[1;32mError creating surface\x1b[0m\n"); // used to wrap window
            let request_adapter_options = RequestAdapterOptions { // see here for an explanation of each option :) https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions
                power_preference: PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surf)
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
            let adapter_request = graphics_instance.request_adapter(&request_adapter_options);
            
            //  cannot call await in App due to ApplicationHandler constraint 
            // for this reason I will spawn a task using a tokio runtime, use a user event proxy to get dev and queue into app
            let prx = self.proxy.take().unwrap();
            spawn(async move {
                // now await adapter
                println!("\x1b[0;33mAwaiting adapter...\n\x1b[0m"); // in Rust, escape code for CSI isn't \033, it's \x1b. reset with [0m in case errors 
                let adapter_wan = adapter_request.await.expect("\x1b[1;31mCouldn't fetch adapter\x1b[0m\n");
                println!("\x1b[0;33mAwaiting device...\x1b[0m\n");
                let dev_req = adapter_wan.request_device(&desc).await.expect("\x1b[1;31mDevice request failed\x1b[0m\n"); // logical *handle* for physical GPU!
                let _ = prx.send_event(Outcome::ADAPTER(dev_req));
            });
            self.setup = false;
        } // end of setup
    
        // Device is an open connection to your GPU. Adapter can now die, it's okay :')
        // Dev's gonna be responsible for all the really cool stuff :p
        // TODO: enclose this backend setup within a conditional, use a bool flag in app to prevent redoing all of it on successive resumptions


    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: Outcome) {
            match event {
                Outcome::ADAPTER(adapter_wan) => {
                    self.device = Some(adapter_wan.0);
                    self.queue = Some(adapter_wan.1);
                    println!("\033[1;33mDevice and Queue ready! \033[0m\n")
                }
            }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Slipping through my fingers all the time\nI try to capture every minute... \nThe feeling in it, slipping through my fingers all the time... \nUntil next time!");
                event_loop.exit();
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
            }
            _ => (),
        }
    }
}
