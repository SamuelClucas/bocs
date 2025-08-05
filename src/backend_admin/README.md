# Backend Admin  
  
This directory contains the beating heart of this simulation engine—**its graphical backend setup** and **its event handler**.  
Nobody likes boilerplate code... but I have to! This is most of that lives.   
*Ignore if unbothered by configuration of the graphical backend or application design itself.* 

- See the [state handler](./state.rs) for async request handling during intial pipeline setup, and for the configuration of the compute and render pipelines themselves.  
- See the [app dispatcher](./app_dispatcher.rs) for window setup and event dispatch configuration (the nervous system of the app).  

> [!Note] 
> Currently, the app should be ran on one monitor only (no switching). This is due to a lack of perceived clarity and consistency across platforms using winit's dpi crate. See [here](../../docs/lights%20camera%20action/The%20Near%20Plane.md) for notes my implementation of a camera frustum and why the aspect ratio of the window is integral to the app's functionality. I plan to address DPI-awareness in future updates.  



