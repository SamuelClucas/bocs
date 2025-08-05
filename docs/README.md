# Docs
  
This project demands lots of learning:  
- learn Rust
    - design patterns
    - API abstraction
- learn wgpu
    - graphical pipelines (2D texture creation stage)
    - compute pipelines (workgroups, dispatch)
    - GPU programming ([WGSL](https://www.w3.org/TR/WGSL/) shaders)
    - memory management (e.g., alignment, buffer types, indexing) 
- learn winit
    - interfacing with OS kernel
    - application architecture
    - async programming (multi-threading)

I needed a place to keep track of it all. *This is that place*.  

I've written notes that feel approachable, explicit, and precise. Please read them if you want to better understand any of the concepts or code involved in the creation of such a program.  
> [!NOTE]
> I avoid writing here until I have a secure understanding of my implementation details. Sometimes, I need to write preliminary documentation to formalise my mental model before implementation. These notes are less-so learning resources, more-so sanity checks. I will mark them as such (using a note like this) until they are concrete.  

### Contents:
1. [Winit](https://docs.rs/winit/latest/winit/index.html)  
    ...[What Is a Context?](./winit/WTF%20Is%20a%20Context.md)  
    ...[Threads](./winit/Threads.md)  
    ...[Time to Win-it](./winit/Time%20to%20Win-it.md)  
2. [wgpu Setup](https://docs.rs/wgpu/latest/wgpu/index.html)  
    ...[A Brief Introduction](./wgpu%20setup/A%20Brief%20Introduction.md)  
    ...[Asynchronous Programming](./wgpu%20setup/Asynchronous%20Programming.md)  
3. lights, camera, action  
    ...[The Near Plane](./lights%20camera%20action/The%20Near%20Plane.md)  **preliminary**  
Appendix. [Current Plan](./Current%20Plan.md)
