use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::event::{InnerSizeWriter, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Fullscreen, Window};
use std::sync::Arc;

use crate::backend_admin::state::State;

/// Setup for logical App struct \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n
#[derive(Default)]
pub struct App {
    state: Option<State>,
    proxy: Option<EventLoopProxy<State>>,
    aspect_ratio: Option<f32>,
    size: Option<PhysicalSize<u32>>
}

impl App  {
    pub fn new (fun: impl FnOnce()-> EventLoopProxy<State>) -> App {
        App {
            state: None,
            proxy:  Some(fun()), // smuggle proxy into app using move closure for downstream requests
            aspect_ratio: None,
            size: None
        }
    }
}

/// implements ApplicationHandler for logical App
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // ran once on init
        let primary_monitor = event_loop.primary_monitor().expect("No primary monitor\n");

        let dims = primary_monitor.size();
        self.aspect_ratio = Some(dims.width as f32 / dims.height as f32);
        self.size = Some(winit::dpi::PhysicalSize::new((dims.width as f32 / 2.0) as u32, (dims.width as f32 / self.aspect_ratio.as_ref().unwrap()) as u32));

        let time_step_width = 1.0 / (primary_monitor.refresh_rate_millihertz().unwrap() as f32 / 1000.0); // TODO: update dims and timestepwidth on monitor change

        // window: logical representation of gpu output, managed by system os
        let window_attributes = Window::default_attributes()
                .with_title("📦")
                .with_blur(true)
                .with_inner_size(self.size.clone().unwrap());

        let window = Arc::new(event_loop.create_window(window_attributes).expect("\x1b[1;31mError creating window\x1b[0m\n"));
    
        // Need async context for requests
        // hence using tokio::spawn task, use a user event proxy to inject awaits back into app
        if let Some(prx) = self.proxy.take() {
            let state = pollster::block_on(State::new(window.clone())).expect("Couldn't get state");
            self.user_event(&event_loop,state);
        } // end of setup: go to App::user_event() :)
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, mut event: State) {

            event.resize(
                event.window.inner_size().width,
                event.window.inner_size().height
            );

            self.state = Some(event);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return
        };

        match event {
            WindowEvent::KeyboardInput {
                event: winit::event::KeyEvent {
                        physical_key: winit::keyboard::PhysicalKey::Code(code),
                        state: key_state, 
                        ..},
                ..} => {
                    state.handle_key(&event_loop, code, key_state.is_pressed()) // self.state, not KeyEvent::state
            },
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            // scale new size by aspect ratio constraint as in resume()
            WindowEvent::Resized(size) =>
            {
                self.size = Some(PhysicalSize::new(size.width, (size.width as f32 * self.aspect_ratio.as_ref().unwrap()) as u32));
                state.window.request_inner_size(self.size.clone().unwrap());
                state.resize(self.size.as_ref().unwrap().width, self.size.as_ref().unwrap().height); // reconfigs surface to match new size dims
               
            },
            // Store scale factor for dynamic DPI-aware resizing for conversion into virtual size (i.e., not physical pixels)
            // Keep up with monitor resolution changes, as well as monitor switch
            WindowEvent::ScaleFactorChanged { scale_factor, mut inner_size_writer } => {
                state.scale_factor = Some(scale_factor);
            
                if let Some(size) = self.size.as_mut() {
                    size.width = state.window.as_ref().inner_size().width;
                    size.height = (size.width as f32 * self.aspect_ratio.unwrap()) as u32;
                    inner_size_writer.request_inner_size(size.clone());
                    state.resize(size.width, size.height);
                }
            },            
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(_) => {},
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    },
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }
            },
            _ => {()},
        }
    }
    }
