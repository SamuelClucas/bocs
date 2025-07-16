use std::default;

use wgpu::wgc::present::SurfaceOutput;
use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop};
use winit::window::{Window, WindowId};
use wgpu::{Instance, InstanceDescriptor, RequestAdapterOptionsBase};
use wgpu::{RequestAdapterOptions, PowerPreference, Surface} ;

/// Setup for logical App struct, with logical Window \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n

// Logical abstraction for the application
#[derive(Default)]
pub struct App {
    window: Option<Window>,
}

/// implements ApplicationHandler for logical App
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
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
        
        let adapter = graphics_instance.request_adapter(&request_adapter_options);
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
