use cgmath::{Vector2, Vector3, Vector4};
use winit::dpi::PhysicalSize;


pub struct OrbitalCamera {
    c: Vector3<f32>, // where c is camera pos in world space, 
    f: Vector3<f32>, // where f is unit vector from c to world space origin, orthogonal to u and r (u X r)
    u: Vector3<f32>, // where u is unit vector up from c, orthogonal to f and r (f x r)
    r: Vector3<f32>, // where r is unit vector right from c, orthogonal to f and u (f X u)
    
    
}

impl OrbitalCamera {   
    // returns right-handed, orthogonal vector to a, b
    pub fn cross(a: &Vector3<f32>, b: &Vector3<f32>) -> Vector3<f32>{
        Vector3::new(
            (a.y * b.z) - (a.z * b.y), // x
            (a.z * b.x) - (a.x * b.z), // y
            (a.x * b.y) - (a.y * b.x) // z
        )
    }
    // returns scalar sum of component-wise products of a and b
    pub fn dot(a: &Vector3<f32>, b: &Vector3<f32>) -> f32{
        (a.x*b.x)+(a.y*b.y)+(a.z*b.z)
    }

    pub fn magnitude(input: &Vector3<f32>) -> f32 {
        let square = Self::dot(input, input);
        square.sqrt()
    }

    pub fn new(window_size: PhysicalSize<u32>, i: f32, j: f32, k: f32) -> Self {
        let pos = Vector3::new(i,j,k);
        let mag = Self::magnitude(&pos);
        // forward is negative camera pos, normalised by its magnitude
        let forward = -pos/mag;

        // at first, let up be 1 unit in j direction
        let up = Vector3::new(pos.x, pos.y + 1.0, pos.z);

        // get right by u cross f, then normalise by its magnitude
        let right = Self::cross(&up, &forward);
        let right = right/Self::magnitude(&right); // norm

        // recompute up to ensure orthogonality with forward and right in new basis
        // this is done by forward cross right, then normalise (just to be safe)
        let up = Self::cross(&forward, &right); // now truly orthogonal to f and r
        let up = up/Self::magnitude(&up); // norm

       OrbitalCamera { 
        c: pos,
        f: forward, 
        u: up, 
        r: right 
    }

    }
    
}
