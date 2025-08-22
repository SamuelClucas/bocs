# World

I come here to escape the boilerplate nightmare that is low-level app/pipeline setup — essentially any programming ideas that I can implement without use of `winit` or `wgpu`.
World: backend-agnostic, pipeline agnostic Rust-side representations of 'world' entities, responsible for managing their state and providing meta-data for the Render/Compute passes. 
 This includes:  
- [camera](./camera.rs)  
- [voxel_grid](./voxel_grid.rs) 
- [brownian_motion](./brownian_motion.rs) **purely experimental**  

### Camera Design
The interactive and visual elements of this app depend on the implementation design choices in [camera](./camera.rs). This was a really exciting learning opportunity for me, as I have always wondered how cameras really work when using other visualisation libraries (like [here](https://github.com/SamuelClucas/Morpheus) in my undergraduate research project).  

I came up with an approach that makes sense to me - at present, I don't know if it will prove feasibly, but I am excited by the idea. I created a section in the [docs](../../docs/README.md) that discusses all details surrounding lighting and camera behaviour in this engine. If you're interested, check it out! ([This](../../docs/lights%20camera%20action/The%20Near%20Plane.md) is a good starting point)

##### Prerequisites for Understanding  
I had to really sit with a lot of new ideas to be able to write this from scratch. I will circle back to these (bear with), exploring my mental model for each. I hope it helps if you are struggling with them!   
- The dot product  
- The cross product  
- The determinant (helpful)  


### Legacy and Experimental Code  
Voxel grid code will be handled by [shaders]() wherever possible, given the inefficiency of computation of 200 * 200 * 200 voxels on a CPU. 

Similarly, the brownian motion code is experimental/legacy code. When writing this file, I had just begun learning Rust. I have kept it around for future reference should it serve me somehow.