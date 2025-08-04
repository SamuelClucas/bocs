use cgmath::{InnerSpace, Vector2, Vector3, Vector4};
use winit::dpi::PhysicalSize;
use approx::abs_diff_eq;


pub struct OrbitalCamera {
    c: Vector3<f64>, // where c is camera pos in world space, 
    f: Vector3<f64>, // where f is unit vector from c to world space origin, orthogonal to u and r (u X r)
    u: Vector3<f64>, // where u is unit vector up from c, orthogonal to f and r (f x r)
    r: Vector3<f64>, // where r is unit vector right from c, orthogonal to f and u (f X u)  
    scroll_coeff: f64
}

impl OrbitalCamera {   
    // returns right-handed, orthogonal vector to a, b
    pub fn cross(a: &Vector3<f64>, b: &Vector3<f64>) -> Vector3<f64>{
        Vector3::new(
            (a.y * b.z) - (a.z * b.y), // x
            (a.z * b.x) - (a.x * b.z), // y
            (a.x * b.y) - (a.y * b.x) // z
        )
    }
    // returns scalar sum of component-wise products of a and b
    pub fn dot(a: &Vector3<f64>, b: &Vector3<f64>) -> f64{
        (a.x*b.x)+(a.y*b.y)+(a.z*b.z)
    }

    pub fn magnitude(input: &Vector3<f64>) -> f64 {
        let square = Self::dot(input, input);
        square.sqrt()
    }
    // this is moving to the compute shader
    pub fn world_to_ruf_coeffcients(&self, input: Vector3<f64>) -> Vector3<f64> { // right is x, up is y, forward is z
        Vector3::new(
                Self::dot(&input, &self.r), // right
                Self::dot(&input, &self.u), // up
                Self::dot(&input, &self.f) // forward
        )
    }
    /// recompute ruf basis vectors on camera movement
    /// TODO: implement angle-based mapping of dx and dy into world deltas to avoid normalisation error drift
    pub fn update(&mut self, dx: Option<f64>, dy: Option<f64>, dscroll: Option<f64>) {
        let multiplier_to_surface = if let Some(d_scroll) = dscroll{
            let old_mag = Self::magnitude(&self.c);
            self.c /= old_mag; // normalise
            let new_mag = (old_mag + (d_scroll * self.scroll_coeff)).clamp(1.0, 500.0);
            self.c *= new_mag ; // new scaled vector
            new_mag
        }
        else { Self::magnitude(&self.c)};
        
        // distribute over components, normalise and scale back to surface
        let new_mag = if let (Some(dx), Some(dy)) = (dx, dy) {
            self.c.x -= dx;
            self.c.y -= dy;
            self.c.z += dx;
            Some(Self::magnitude(&self.c)) 
        }
        else { None };

        if let Some(new_mag) = new_mag {
            self.c /= new_mag;
            self.c *= multiplier_to_surface;
            self.f = -self.c/new_mag; // new forward direction, normalised
        }
        else { self.f = -self.c/multiplier_to_surface; }; // recompute f regardless given update function has been called
        let multiplier_to_surface = Self::magnitude(&self.c);  // HACKY FIX!!!
        
        assert!(
            abs_diff_eq!(
                Self::magnitude(&self.c), 
                multiplier_to_surface)
            );

        let up = Vector3::new(self.c.x, self.c.y + 0.9, self.c.z);

        self.r = Self::cross(&up, &self.f);
        self.r = self.r/Self::magnitude(&self.r); // new right, normalised

        self.u = Self::cross(&self.f, &self.r);
        self.u = self.u/Self::magnitude(&self.u); // new up, normalised
        println!("Magnitude:{}", multiplier_to_surface);
    }

    pub fn new(i: f64, j: f64, k: f64) -> Self {
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
        scroll_coeff: 0.3,
        c: pos,
        f: forward, 
        u: up, 
        r: right 
    }

    }
    
}
