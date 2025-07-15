use winit::{
    event_loop::{
        ControlFlow, 
        EventLoop,
    }, 
};
mod backend_admin;
use crate::backend_admin::app_and_event_handler::App;
// TODO: put comments in docs/? feels messy right now
/// This spins up the the simulation engine
/// See winit and wgpu docs for more information
fn main() {
    // The EventLoop interfaces with the OS 
    // Tracking WindowEvent and DeviceEvent events...
    let event_loop = EventLoop::new().unwrap(); // not an active event loop, need proxy for custom window config

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. 
    event_loop.set_control_flow(ControlFlow::Poll);

    // ... dispatching them through App's implementation
    // of EventHandler! (see backend_admin/app_and_event_handler.rs
    let _x = event_loop.run_app(&mut App::default()); // ! APP ENTRY HERE ! //

}