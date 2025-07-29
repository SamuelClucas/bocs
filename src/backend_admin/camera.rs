use cgmath::{Vector2, Vector3, Vector4};
use winit::dpi::PhysicalSize;


pub struct Orbital_Camera {
    pos_from_origin: Vector3<f32>, // negate for from camera
    origin_from_pos: Vector3<f32>,
    frustum: [Vector3<f32>; 4],
    fov: f32, // could be int but simpler to implement with same type
    radius: f32
    
    
}

impl Orbital_Camera {   
    pub fn new(fov: f32, radius: f32, window_size: PhysicalSize<u32>) -> Self {
        let mut pitch: f32 = 0.0;
        let mut yaw: f32 = 0.0;
        // vector f rom origin to camera
        let mut pos: Vector3<f32> = Vector3::<f32>{
            x: pitch.cos() * yaw.sin() * radius, 
            y: pitch.sin() * radius, 
            z: pitch.cos() * yaw.cos() * radius};

        // vector from camera to origin 
        let mut inverse: Vector3<f32> = Vector3::<f32>{
            x: -(pitch.cos() * yaw.sin()), // pitch increase, x decrease; yaw increase, x increase
            y: -(pitch.sin()), 
            z: -(pitch.cos() * yaw.cos())
        };
        // horizontal fov delta
        let delta_in_radians = fov.to_radians() / 2.0;

        let frustum = [ // bottom left to bottom right, ccw
            Vector3::<f32> {
                x: -delta_in_radians.cos(),
                y: -delta_in_radian.cos(),

            },
            Vector3::<f32> {

            },
            Vector3::<f32> {

            },
            Vector3::<f32> {

            }
        ];

        
        Camera {
            pos_from_origin: pos,
            origin_from_pos: inverse,
            radius: radius,
        }

    }
    
}
