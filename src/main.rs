use winit::{
    event_loop::{
        ControlFlow, 
        EventLoop,
    }, 
};
mod backend_admin;
use crate::backend_admin::{state::State, app_with_event_handler::App} ;


/// Entry into app \n
/// See winit and wgpu docs for more information \n
#[tokio::main] // this is for async as in backend_admin/app_with_event_handler! see here: https://rust-lang.github.io/async-book/part-guide/async-await.html
async fn main() { // see async  
    // The EventLoop interfaces with the OS 
    // Tracking WindowEvent and DeviceEvent events...
    let event_loop = EventLoop::<State>::with_user_event().build().unwrap(); // not an active event loop
    let proxy = event_loop.create_proxy(); // used to inject awaited requests back into App

    // ControlFlow::Poll continuously runs the event loop (through Application Handler in App), even if the OS hasn't dispatched any events. 
    event_loop.set_control_flow(ControlFlow::Poll);

    let _x = event_loop.run_app(&mut App::new(move || proxy)); // ! APP ENTRY HERE ! //

}