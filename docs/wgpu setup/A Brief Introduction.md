Firstly, create an `Instance`, which is the entry point into wgpu, used to create an Adapter, and a Surface.

An adapter is a handle to a physical device like a GPU.  A surface is a handle to a presentable surface, like the window created in winit.

*For clarity:* 
	A 'handle' is an abstract or 'logical' reference to a resource.

The adapter, following its successful request, can be used to open a connection to the host system's corresponding `Device`,  paired with a `Queue` used to submit draw calls. 

This is all fairly straight forward so far, but with each request comes the added complexity of asynchronous programming. It takes some unknown during for a request to be completed. In Rust, they are represented through the [`Future`](https://doc.rust-lang.org/nightly/core/future/trait.Future.html) trait. To cut a long story short, you cannot await a future in a non-async programming context, i.e., you must make it clear to the compiler which code isn't sequential. You must then also handle the evaluation of that code. I discuss this in-depth in [Asynchronous Programming](./Asynchronous\ Programming), but if your interest lies exclusively in graphical rendering, feel free to skip ahead to that section.