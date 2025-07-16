use std::default;
use tokio::*;
use wgpu::wgc::present::SurfaceOutput;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::window::{Window, WindowId};
use wgpu::{Instance, InstanceDescriptor, RequestAdapterOptionsBase};
use wgpu::{RequestAdapterOptions, PowerPreference, Surface, DeviceDescriptor, Device, Queue, Features, FeaturesWGPU, FeaturesWebGPU, Limits, MemoryHints, Trace} ;

/// Setup for logical App struct, with logical Window \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n

// Logical abstraction for the application
#[derive(Default)]
pub struct App {
    window: Option<Window>,
   device: Option<Device>,
    queue: Option<Queue>,
}

/// implements ApplicationHandler for logical App
impl ApplicationHandler for App {
    #[tokio::main] // this is for async! see here: https://rust-lang.github.io/async-book/part-guide/async-await.html
    async fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
                .with_title("📦")
                .with_blur(true);
        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        let inst_descriptor = InstanceDescriptor::from_env_or_default();
        let graphics_instance = Instance::new(&inst_descriptor);

        let surface = graphics_instance.create_surface(self.window.as_ref().unwrap()).ok(); // used to wrap window
        
        let request_adapter_options = RequestAdapterOptions { // see here for an explanation of each option :) https://gpuweb.github.io/gpuweb/#dictdef-gpurequestadapteroptions
            power_preference: PowerPreference::None,
            force_fallback_adapter: true,
            compatible_surface: surface.as_ref()
        };
        let adapter_wan = (graphics_instance.request_adapter(&request_adapter_options)).await.unwrap(); // logical *handle* for physical GPU!
        
        // device descriptor and request (device, queue)
        let desc = DeviceDescriptor {
            required_features: Features {features_wgpu: {FeaturesWGPU::POLYGON_MODE_POINT; 
                                                        FeaturesWGPU::STORAGE_RESOURCE_BINDING_ARRAY; 
                                                        FeaturesWGPU::BUFFER_BINDING_ARRAY;
                                                        FeaturesWGPU::MULTI_DRAW_INDIRECT;
                                                        FeaturesWGPU::PUSH_CONSTANTS;
                                                        FeaturesWGPU::POLYGON_MODE_LINE},
                                                        features_webgpu: {FeaturesWebGPU::DEPTH_CLIP_CONTROL}}, // only request what you require (see about OptionalCapabilities - limits and features - here: https://gpuweb.github.io/gpuweb/#feature also https://gpuweb.github.io/gpuweb/#feature-index)
            required_limits: Limits::downlevel_defaults(),
            trace: Trace::Off,
            memory_hints: MemoryHints::Performance,
            label: Some("wan adapter")

        }; // see here for dev desc debrief: https://gpuweb.github.io/gpuweb/#dictdef-gpudevicedescriptor
        let (device, queue) = adapter_wan.request_device(&desc).await.unwrap(); 
        self.device = Some(device);
        self.queue = Some(queue);
        // Device is an open connection to your GPU. Adapter can now die, it's okay :')
        // Dev's gonna be responsible for all the really cool stuff :p
        // TODO: enclose this backend setup within a conditional, use a bool flag in app to prevent redoing all of it on successive resumptions


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
