use winit::dpi::PhysicalSize;
use crate::world::voxel_grid::{P2i, P3};

const DY_SENS: f32 = 0.0001;
const DX_SENS: f32 = 0.00005; // half as sensitive x given typical display aspect ratio

pub struct OrbitalCamera {
    pub c: P3, // where c is camera pos in world space, 
    pub f: P3, // where f is unit vector from c to world space origin, orthogonal to u and r (u X r)
    pub u: P3, // where u is unit vector up from c, orthogonal to f and r (f x r)
    pub r: P3, // where r is unit vector right from c, orthogonal to f and u (f X u)  
    pub centre: P3, // ASSUMES WINDOW EXISTS ON CAMERA INIT (check state.rs new())
    scroll_coeff: f32,
    dx_sens: f32,
    dy_sens: f32
}

impl OrbitalCamera {   
    // BASIC UTILITY FUNCTIONS
    pub fn normalise(a: &P3, mag: &f32) -> P3{
        [a[0] / mag,
        a[1] / mag,
        a[2] / mag]
    }
    pub fn scale(a: &P3, k: &f32) -> P3{
        [a[0] * k, a[1] * k, a[2] * k]
    }
    pub fn negate(a: &P3) -> P3{
        [-a[0], -a[1], -a[2]]
    }
    pub fn add(a: &P3, b: &P3) -> P3{
        [a[0] + b[0],
        a[1] + b[1],
        a[2] + b[2]]
    }
    // returns right-handed, orthogonal vector to a, b
    pub fn cross(a: &P3, b: &P3) -> P3 {
        [
            (a[1] * b[2]) - (a[2] * b[1]), // x i r 
            (a[2] * b[0]) - (a[0] * b[2]), // y j u
            (a[0] * b[1]) - (a[1] * b[0]) // z k f
        ]
    }
    // returns scalar sum of component-wise products of a and b
    pub fn dot(a: &P3, b: &P3) -> f32{
        (a[0]*b[0])+(a[1]*b[1])+(a[2]*b[2])
    }

    pub fn magnitude(input: &P3) -> f32 {
        let square = Self::dot(input, input);
        square.sqrt()
    }
    // this is moving to the compute shader
    pub fn world_to_ruf(&self, input: &P3) -> P3 { // right is x, up is y, forward is z
        let offset = [input[0]-self.c[0], input[1]-self.c[1], input[2]-self.c[2]];
        [
                Self::dot(&offset, &self.r), // right
                Self::dot(&offset, &self.u), // up
                Self::dot(&offset, &self.f) // forward
        ]
    }
    pub fn ruf_to_ru_plane(&self, input: &P3, r_scale: &f32) -> P2i {
        let normalised = OrbitalCamera::normalise(input, &OrbitalCamera::magnitude(input));
        let centre_mag = OrbitalCamera::magnitude(&self.centre); // scale factor for F and U

        let up_multiplier = centre_mag/normalised[2];

        // for 90 eg vertical fov, F and U are 1:1
        // scale u by f coefficient to centre
        let up_pixels = normalised[1] * up_multiplier; 
        let right_pixels =  normalised[0] * up_multiplier * r_scale;

        [ right_pixels as i32, up_pixels as i32]
    }
    pub fn sin(a: P3) -> P3 {
        [a[0].sin(),
        a[1].sin(),
        a[2].sin()]
        
    }
    pub fn cosine(a: P3) -> P3 {
        [a[0].cos(),
        a[1].cos(),
        a[2].cos()]
        
    }
    /// Order of rotation matters, so only this function is exposed externally to handle camera rotations
    /// to ensure order is always preserved
    pub fn handle_rotate(&mut self, dx: f32, dy: f32){
        self.rotate_up(dx); 
        self.rotate_right(dy);
        self.orthonormalise();
    }

    /// Rotate about Up vector (i.e. yaw) takes mouse x deltas
    fn rotate_up(&mut self, dx: f32) {
        // Up == Up
        let coef_sin = (dx * self.dx_sens).sin();
        let coef_cos = (dx * self.dx_sens).cos();
        let old_r = self.r.clone();
        let old_f = self.f.clone();
        self.r = OrbitalCamera::add(&OrbitalCamera::scale(&old_f, &coef_sin), &OrbitalCamera::scale(&old_r, &coef_cos));
        self.f = OrbitalCamera::add(&OrbitalCamera::scale(&old_r, &-coef_sin), &OrbitalCamera::scale(&old_f, &coef_cos));
    }

    /// Rotate about Right vector (i.e. pitch) takes mouse y deltas
    pub fn rotate_right(&mut self, dy: f32) { // CHECK THESE FOR INSITU MESS
        // Right == Right
        let coef_sin = (dy * self.dy_sens).sin();
        let coef_cos = (dy * self.dy_sens).cos();
        let old_u = self.u.clone();
        let old_f = self.f.clone();
        self.u = OrbitalCamera::add(&OrbitalCamera::scale(&old_u, &coef_cos), &OrbitalCamera::scale(&old_f, &-coef_sin));
        self.f = OrbitalCamera::add(&OrbitalCamera::scale(&old_f, &coef_cos), &OrbitalCamera::scale(&old_u, &coef_sin));
    }

    pub fn orthonormalise(&mut self) {
        self.r = OrbitalCamera::normalise(&self.r, &OrbitalCamera::magnitude(&self.r));
        self.u = OrbitalCamera::normalise(&self.u, &OrbitalCamera::magnitude(&self.u));

        self.f = OrbitalCamera::cross(&self.r, &self.u);
        self.f = OrbitalCamera::normalise(&self.f, &OrbitalCamera::magnitude(&self.f)); // norm in case r and u not at 90 deg

        self.r = OrbitalCamera::cross(&self.u, &self.f);
        // normalising just for sureness
        self.r = OrbitalCamera::normalise(&self.r, &OrbitalCamera::magnitude(&self.r));
    }

    /// recompute ruf basis vectors on camera movement
    /// TODO: implement angle-based mapping of dx and dy into world deltas to avoid normalisation error drift
    pub fn update(&mut self, dx: Option<f32>, dy: Option<f32>, dscroll: Option<f32>, size: Option<&PhysicalSize<u32>>) { // kitchen sink vibes...
        // HANDLE ZOOM
        let multiplier_to_surface = if let Some(d_scroll) = dscroll{
            let old_mag = Self::magnitude(&self.c);
            self.c = OrbitalCamera::normalise(self.c, old_mag); // normalise

            let new_mag = (old_mag + (d_scroll * self.scroll_coeff)).clamp(1.0, 500.0);
            self.c = OrbitalCamera::scale(self.c, new_mag);  // new scaled vector
            new_mag
        }
        else { Self::magnitude(&self.c)}; // no zoom, multiplier is 1.0
        
        // HANDLE PAN
        // distribute pan deltas over components, normalise and scale back to surface
        let new_mag = if let (Some(dx), Some(dy)) = (dx, dy) {
            self.c[0] -= dx;
            self.c[1] -= dy;
            self.c[2] += dx;
            Some(Self::magnitude(&self.c)) 
        }
        else { None };

        if let Some(new_mag) = new_mag {
            self.c = OrbitalCamera::scale(
                    OrbitalCamera::normalise(self.c, new_mag),
                    multiplier_to_surface);
            self.f = OrbitalCamera::normalise(
                    OrbitalCamera::negate(self.c.clone()), // WATCH OUT FOR OWNERSHIP
                    new_mag); // new forward direction, normalised 
        } // no pan happens, use multiplier to surface to recompute f (c has already been updated)
        else { self.f = OrbitalCamera::normalise(
                        OrbitalCamera::negate(self.c.clone()), 
                        multiplier_to_surface)
                    }; // recompute f regardless given update function has been called

        let multiplier_to_surface = Self::magnitude(&self.c);  // HACKY FIX - float division error build up!!!

        // HANDLE UP NEAR POLES
        let up: P3 = if OrbitalCamera::dot(&self.c, &[0.0, 1.0, 0.0]) > 0.9 {
            [self.c[0] + 1.0, self.c[1], self.c[2]]
        }
        else {[self.c[0], self.c[1] + 0.9, self.c[2]]}; 

        self.r = Self::cross(&up, &self.f);
        self.r = OrbitalCamera::normalise(self.r, Self::magnitude(&self.r)); // new right, normalised

        self.u = Self::cross(&self.f, &self.r);
        self.u = OrbitalCamera::normalise(self.u, Self::magnitude(&self.u)); // new up, normalised
        println!("Magnitude:{}", multiplier_to_surface);
        if let Some(size)= size {
            let kf = (size.width.max(size.height)) as f32 / 2.0; // fixes 90 FOV in larger dimension, given tan(pi/2) = 1
            self.centre = OrbitalCamera::add(self.c.clone(), OrbitalCamera::scale(self.f.clone(), kf));
        }
    }

    pub fn new(p: P3, size: &PhysicalSize<u32>) -> Self {

        let mag = Self::magnitude(&p);
        // forward is negative camera p, normalised by its magnitude
        let forward = OrbitalCamera::normalise(
            OrbitalCamera::negate(p.clone()), 
            mag);

        // f* kf = centre of near plane
        // near-plane edges = (k * kf) +- (max(width, height) * r or u) (r for horizontal, u for vertical)
        // this gives directions for any x, y pixel
        let kf = (size.width.min(size.height)) as f32 / 2.0; // fixes 90 FOV in smaller dimension, given tan(pi/2) = 1
        let centre = OrbitalCamera::scale(forward.clone(), kf);

        // at first, let up be 1 unit in j direction
        let up = [p[0], p[1] + 1.0, p[2]];

        // get right by u cross f, then normalise by its magnitude
        let right = Self::cross(&up, &forward);
        let right = OrbitalCamera::normalise(right, Self::magnitude(&right)); // norm

        // recompute up to ensure orthogonality with forward and right in new basis
        // this is done by forward cross right, then normalise (just to be safe)
        let up = Self::cross(&forward, &right); // now truly orthogonal to f and r
        let up = OrbitalCamera::normalise(up, Self::magnitude(&up)); // norm

       OrbitalCamera { 
        scroll_coeff: 0.3,
        c: p,
        f: forward, 
        u: up, 
        r: right,
        centre: centre,
        dx_sens: DX_SENS,
        dy_sens: DY_SENS
    }

    }
    
}
