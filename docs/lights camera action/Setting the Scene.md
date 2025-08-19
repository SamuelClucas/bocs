# Setting the Scene  
I wanted to visually represent the app's design choices. These are abstract, high-level diagrams that should give some insight into the engine's functionality.  

### App window *as* the Near Plane  
The window is the literal intersecting plane into the simulation space.  
* Focal distance from camera to centre is height/2 * tan(90) = height/2.  
* Horizontal scaling that preserves 90 degree vertical viewing angle is (width/2) / (height/2).  
<img src="../../assets/Fig_1.png" alt="Window as near plane" width="400">

### Orbital Camera  
<img src="../../assets/Fig_2.png" alt="Orbital Camera" width="400">

### Voxel Grid as Flat, Contiguous Memory
<img src="../../assets/Fig_3.png" alt="Voxel Grid as Flat, Contiguous Memory" width="400">

### Raymarching through Near Plane Pixel Grid
<img src="../../assets/Fig_4.png" alt="Raymarching through Near Plane Pixel Grid" width="400">

### Engine Architecture Overview
<img src="../../assets/Fig_5.png" alt="Engine Architecture Overview" width="400">
