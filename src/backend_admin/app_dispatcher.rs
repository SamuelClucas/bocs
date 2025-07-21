use winit::application::ApplicationHandler;
use winit::event::{WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::window::{Window};
use std::sync::Arc;

use crate::backend_admin::state::State;

/// Setup for logical App struct \n
/// App implements ApplicationHandler for resuming of app, WindowEvent handling \n
#[derive(Default)]
pub struct App {
    state: Option<State>,
    proxy: Option<EventLoopProxy<State>>,
}

impl App  {
    pub fn new (fun: impl FnOnce()-> EventLoopProxy<State>) -> App {
        App {
            state: None,
            proxy:  Some(fun()), // smuggle proxy into app using move closure for downstream requests
        }
    }
}

/// implements ApplicationHandler for logical App
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // ran once on init
        // window: logical representation of gpu output, managed by system os
        let window_attributes = Window::default_attributes()
                .with_title("📦")
                .with_blur(true);
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
            WindowEvent::ScaleFactorChanged { .. } => {
                // Store scale factor for dynamic DPI-aware resizing for conversion into virtual size (i.e., not physical pixels)
                state.scale_factor = Some(state.window.as_ref().scale_factor()); 
                let size = state.window.inner_size();
                state.resize(size.width, size.height);
            },
            WindowEvent::RedrawRequested => {
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }
            },
            _ => {()},
        }
    }
    }
