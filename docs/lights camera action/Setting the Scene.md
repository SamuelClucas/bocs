# Setting the Scene  
I wanted to visually represent the app's design choices. These are abstract, high-level diagrams that should give some insight into the engine's functionality.  

### App window *as* the Near Plane  
This is why I opted to fix the aspect ratio of the app's window on launch relative to the display in use. The window is the literal intersection plane into the simulation space, and its width and height will influence the camera's horizontal and vertical field-of-view. With this design, no matter how the window is resized (with tight aspect-ratio control), the 2D texture presented at each time step should be crisp and representative of the space.  
[](../../assets/Fig_1.png)

### Orbital Camera  
[](../../assets/Fig_2.png)

### Voxel Grid as Flat, Contiguous Memory
[](../../assets/Fig_3.png)

### Raymarching through Near Plane Pixel Grid
[](../../assets/Fig_4.png)

### App Architecture Overview
[](../../assets/Fig_5.png)