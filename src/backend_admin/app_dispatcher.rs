use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, InnerSizeWriter, MouseScrollDelta, WindowEvent};
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
    size: Option<PhysicalSize<u32>>,
    scale_factor: Option<f64>,
}

impl App  {
    pub fn new (fun: impl FnOnce()-> EventLoopProxy<State>) -> App {
        App {
            scale_factor: None,
            state: None,
            proxy:  Some(fun()), // smuggle proxy into app using move closure for downstream requests
            aspect_ratio: None,
            size: None
        }
    }}


/// implements ApplicationHandler for logical App
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) { // ran once on init
        let primary_monitor = event_loop.primary_monitor().expect("No primary monitor\n");

        let mut physical_dims = primary_monitor.size();

        self.aspect_ratio = Some(physical_dims.width as f32 / physical_dims.height as f32);
        let physical_width = physical_dims.width as f32;
        let physical_height = physical_width as f32 / self.aspect_ratio.as_ref().expect("No aspect ratio in App on resumed()\n");

        let scaled_size = winit::dpi::PhysicalSize::new(
            physical_width as u32, 
            physical_height as u32
        );

        self.size = Some(scaled_size);
        
        let time_step_width = primary_monitor.refresh_rate_millihertz()
        .map(|r| 1.0 / (r as f32 / 1000.0))
        .unwrap_or_else(|| {
            println!("Could not determine refresh rate, defaulting to 16.67ms timestep\n");
            1.0 / 60.0
        });

        // window: logical representation of gpu output, managed by system os
        let window_attributes = Window::default_attributes()
                .with_title("📦")
                .with_blur(true)
                .with_inner_size(self.size.clone().expect("No size on window attributes creation\n"));

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
                event.window.inner_size().height,
            );

            self.state = Some(event);
    }
    
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: winit::window::WindowId, event: WindowEvent) {
        let state = match &mut self.state {
            Some(s) => s,
            None => return
        };

        match event {
            WindowEvent::MouseInput { device_id, state, button } => {
                match state {
                    ElementState::Pressed => {
                        self.state.as_mut().unwrap().mouse_is_pressed = true;
                    },
                    ElementState::Released => {
                        if let Some(state) = self.state.as_mut(){
                            state.mouse_is_pressed = false;
                            state.mouse_down = None;
                        }
                        else {println!("No state at mouse input\n");}
                    }
                }
            },
            WindowEvent::CursorMoved { device_id: _device_id, position } => {
                if let Some(state) = self.state.as_mut(){
                    if state.mouse_is_pressed == true {
                        if let Some(mouse_down) = state.mouse_down{
                            let delta = PhysicalPosition {
                                x: position.x - mouse_down.x,
                                y: position.y - mouse_down.y
                            };
                            // handle cursor move here
                            state.camera.update(Some(delta.x as f32), Some(delta.y as f32), None, None);
                            state.mouse_down = Some(position);
                        }
                        else {
                            state.mouse_down = Some(position);
                        }
                    }
                }
                else {println!("No state at cursor moved\n");}
            },
            WindowEvent::MouseWheel { device_id: _device_id, delta, phase: _phase } => {
                if let Some(state) = self.state.as_mut() {
                    match delta {
                        MouseScrollDelta::PixelDelta(pos) => {
                            state.camera.update(None, None, Some(pos.y as f32), None);
                        },
                        MouseScrollDelta::LineDelta(x, y) => {
                            state.camera.update(None, None, Some(y as f32), None);
                        }
                    }
                }
            },
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
            WindowEvent::Resized(size) => {
                if let Some(ar) = self.aspect_ratio {
                    if size.height != self.size.as_ref().unwrap().height {
                        let new_width= (size.height as f32 * ar)as u32;
                        self.size = Some(PhysicalSize::new(new_width, size.height));
                        state.is_surface_configured = false;
                        state.window.request_redraw();
                    }
                    else { // catches all width adjustments
                        self.size = Some(PhysicalSize::new(size.width, (size.width as f32 / ar) as u32));
                        state.is_surface_configured = false;
                        state.window.request_redraw();
                    }
                }
                else { println!("No aspect ratio in resized yet\n"); }
            },       
            WindowEvent::RedrawRequested => {
                match state.render(self.size ) {
                    Ok(_) => {state.window.request_redraw()},
                    Err(e) => {
                        println!("Unable to render {}", e);
                    }
                }
            },
            _ => {()},
        }
        }
    }
