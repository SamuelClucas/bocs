use cgmath::{Vector2, Vector3, Vector4};


pub struct Camera {
    pos_from_origin: Vector3<f32>, // negate for from camera
    origin_from_pos: Vector3<f32>,
    frustum: [Vector3<f32>; 4],
    fov: f32, // could be int but simpler to implement with same type
    radius: f32
    
    
}

impl Camera {   
    fn frustum_and_at(&self) -> [Vector3<f32>; 4] {}

    fn is_in_view(&self) -> bool {}

    fn world_to_frustum(&self) -> Vector2<f32> {} // inverse scaling

    pub fn pan(&self){} // includes left, right, forward, back

    pub fn look_at(&self){} // bound to unit sphere 

    pub fn new(fov: f32, radius: f32) -> Self {
        let mut pitch: f32 = 0.0;
        let mut yaw: f32 = 0.0;
        
        let mut pos: Vector3<f32> = Vector3::<f32>{
            x: pitch.cos() * yaw.sin() * radius, 
            y: pitch.sin() * radius, 
            z: pitch.cos() * yaw.cos() * radius};

        let mut inverse: Vector3<f32> = Vector3::<f32>{
            x: -(pitch.cos() * yaw.sin()), 
            y: -(pitch.sin()), 
            z: -(pitch.cos() * yaw.cos())
        };

        let delta_in_radians = fov.to_radians() / 2.0;

        let frustum = [ // bottom left to bottom right, ccw
            Vector3::<f32> {
                
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
