## The Deep Mental Model

Here’s the **order of operations** that Winit sets up:

1. In `main()`, you:
    
    - create the `EventLoop<T>` (interfaces with system kernel)
    - configure it (e.g. `set_control_flow`) - I used poll for continuous looping even in the absence of event signals 
    - optionally store the event proxy
	
2. You pass your `App` (implementing `ApplicationHandler`) into `event_loop.run_app(&mut app)`— logic in App's implementation of ApplicationHandler is used for dispatch! (UserEvents, WindowEvents, DeviceEvents)
    
3. Behind the scenes, Winit:
    
    - starts the actual **runloop**
    - converts the `EventLoop` into an `ActiveEventLoop`
    - calls your `App::resumed(&ActiveEventLoop)` method
    - that’s **your moment** to call `create_window()` (because you _now_ have the `ActiveEventLoop`)


You don’t ever construct `ActiveEventLoop`. You **react to being handed it**.