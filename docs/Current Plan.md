### **The Plan**  

#### **Stage 1: Build Mathematical Foundations**  ✔️

- **Understand the 3D determinant** geometrically and algebraically  ✔️
- Use that to **intuitively derive the cross product**, so it’s not just a rule but a spatial operation I trust  ✔️
- Apply the cross product to build a **view matrix**: use `right = cross(up, forward)` and so on to derive camera orientation → this unlocks orbiting cameras   ✔️
    
---

#### **Stage 2: Implement Camera & Raymarching**  

- Create a **camera system** with:  
    - Position (`eye`), target (`center`), up vector  ✔️
    - **Frustum definition** using near plane, FOV, and aspect ratio  ✔️
    - **Ray direction generation** per pixel (based on view matrix & projection maths)  ✔️
        
- Implement a **raymarching renderer** that:  
    - Shoots rays through a 3D scalar field (storage buffer)  ✔️
    - Samples values along the ray (e.g. with trilinear interpolation or stepping)  ✔️
    - Produces a **2D output texture** (scalar → grayscale or colour-mapped pixel)  ✔️
        
---

#### **Stage 3: Scalar Field Compute Pipeline** 

- Configure a `wgpu` **compute pipeline**:  
    - Storage buffer representing a 3D voxel grid (e.g., `Vec<f32>`, size `NxNxN`)  ✔️
    - Apply **Laplacian diffusion** using a compute shader across timesteps  ✔️
    - Optional: inject random initial conditions (e.g., lipid seed clusters)  
        
- On each frame:  
    - Run the compute shader → update the scalar field  ✔️
    - Use the camera’s raymarcher to visualise it into a **presentable 2D texture**  ✔️
        
---
#### **Outcome**  
- I should now have a **physically motivated, GPU-accelerated 3D diffusion visualiser**, rendered in Rust, with a mathematically sound camera system — ready to be extended into: 
     
    - Vesicle formation 
    - Thermodynamic modeling 
    - Membrane-bound protein influence 
    - Real cell-like behaviour
 
> [!NOTE]
> If only it were that simple... see issues 
