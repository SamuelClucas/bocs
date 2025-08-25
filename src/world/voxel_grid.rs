/// Rust-side representation of Voxel Grid
/// naming convention agnostic -
/// not ijk, xyz, can be anything -
/// hence d1, d2, d3_size represents size of voxelgrid
/// in that dimension (i.e., [0], [1], [2])
/// The invariant: always 3D - 'voxel' as 'volumetric pixel'
const DIMS: usize = 3; 
const PROJ_DIMS: usize = 2; // projection onto 2D surface

pub type P3 = [f32; DIMS]; // 3D point
pub type Dims3 = [u32; DIMS];
pub type P2f = [f32; PROJ_DIMS]; // 2D Point
pub type P2i = [i32; PROJ_DIMS]; 

/// Enum for each coordinate system,
/// Carries point as P3 or P2 and is 
/// returned from VoxelGrid::get_vertex_at() but is 
/// passed as an arg to VoxelGrid::set_vertex_at()
/// ensuring basis transforms are always explicit and visible
#[derive(Debug, Copy, Clone)]
pub enum SystemSet{
    WORLD(P3),
    RUF(P3),
    SQUARE(P2i),
}
/// Enum for each coordinate system,
/// same as SystemSet but with only an index payload for getting vertices
/// useful to keep code explicit
#[derive(Debug, Copy, Clone)]
pub enum SystemGet{
    WORLD(usize),
    RUF(usize),
    SQUARE(usize),
}

/// All world geometries implement the Access trait
/// This includes Square, Cuboid, and VoxelGrid itself
/// I, O generics represent inputs or outputs (either P2s or P3s, or usize) for 'setting' or 'getting' 
pub trait Access<I, O> { 
    fn set_vertex_at(&mut self, a: I, b: O);
    fn get_vertex_at(& self, a: I) -> O; 
}

/// Recommend sticking to CCW or CW when ordering vertices for a face
/// For example, 
/// p1: bottom left
/// p2: top left
/// p3: top right
/// p4: bottom right
/// when looking down the first basis vector of your coordinate system (e.g., ijk, i going into the screen)
#[derive(Debug, Copy, Clone)]
pub struct Square<T> {
    pub p1: T,
    pub p2: T,
    pub p3: T,
    pub p4: T,
}
/// Purely geometric Square
/// Implements Access trait
impl Default for Square<[i32; PROJ_DIMS]> {
    fn default() -> Self {
        Square {
            p1: [0 as i32, 0 as i32],
            p2: [0 as i32, 0 as i32],
            p3: [0 as i32, 0 as i32],
            p4: [0 as i32, 0 as i32],
        }
    }
}
impl Access<usize, P2i> for Square<[i32; PROJ_DIMS]> {
    /// Getter for purely geometric Square vertices (P2s)
    fn get_vertex_at(&self, idx: usize) -> P2i {
        if idx > 4 || idx < 0 { panic!("Out of bounds Square vertex access attempt. Vertex index should be <= 3 and >= 0.\n") } // OOB attempts are unrecoverable errors
         else {
            match idx {
            0 => self.p1,
            1 => self.p2,
            2 => self.p3,
            3 => self.p4,
            _ => panic!("Strange index for Square::at_vertex() call.\n")
            }
        }
    }
    /// Setter for purely geometric Square vertices (P2s)
    fn set_vertex_at(&mut self, idx: usize, point: P2i) {
        if idx > 4 || idx < 0 { panic!("Out of bounds Square vertex access attempt. Vertex index should be <= 3 and >= 0.\n") } // OOB attempts are unrecoverable errors
        else {
            match idx {
            0 => self.p1 = point,
            1 => self.p2 = point,
            2 => self.p3 = point,
            3 => self.p4 = point,
            _ => panic!("Strange index for Square::get_vertex_at() call.\n")
        }
    }}
}

/// This is distinct from Square, where each vertex is P3 type
/// Still, CuboidFace is purely geometric and is not concerned with 
/// coordinate systems
/// Implements Access
#[derive(Debug, Copy, Clone)]
pub struct CuboidFace {
    p1: P3,
    p2: P3,
    p3: P3,
    p4: P3
}
impl Default for CuboidFace {
    fn default() -> Self {
        CuboidFace {
            p1: [0.0, 0.0, 0.0],
            p2: [0.0, 0.0, 0.0],
            p3: [0.0, 0.0, 0.0],
            p4: [0.0, 0.0, 0.0]
        }
    }
}

impl Access<usize, P3> for CuboidFace {
    fn get_vertex_at(& self, idx: usize) -> P3 {
        assert!(idx < 4 && idx >= 0);
         match idx {
            0 => self.p1,
            1 => self.p2,
            2 => self.p3,
            3 => self.p4,
            _ => panic!("Strange index for CuboidFace::get_vertex_at() call.\n")
            }
    }
    fn set_vertex_at(&mut self, idx: usize, point: P3) {
        assert!(idx < 4 && idx >= 0);
        match idx {
            0 => self.p1 = point,
            1 => self.p2 = point,
            2 => self.p3 = point,
            3 => self.p4 = point,
            _ => panic!("Strange index for CuboidFace::get_vertex_at() call.\n")
            }
    }
}
/// Cuboid is a purely geometric object with useful helpers
/// Not to be confused with the VoxelGrid, which is indeed a Cuboid
/// but is sufficiently distinct in its role as a container
/// for simulation, hence it is its own struct with additional data and methods and uses System enums
/// See their shared trait "Access"
#[derive(Debug, Copy, Clone)]
pub struct Cuboid {
    f1: CuboidFace,
    f2: CuboidFace
}

impl Default for Cuboid {
    fn default()-> Self {
        Cuboid {
            f1: CuboidFace::default(),
            f2: CuboidFace::default()
        }
    }
}

impl Access<usize, P3> for Cuboid {
    fn get_vertex_at(&self, idx: usize) -> P3 {
        if idx > 7 || idx < 0{ panic!("Out of bounds cuboid vertex access attempt. Vertex index should be <= 7.\n") } // OOB attempts are unrecoverable errors
        else {
            match idx {
            0 => self.f1.p1,
            1 => self.f1.p2,
            2 => self.f1.p3,
            3 => self.f1.p4,
            4 => self.f2.p1,
            5 => self.f2.p2,
            6 => self.f2.p3,
            7 => self.f2.p4,
            _ => panic!("Strange index for Cuboid::get_vertex_at() call.\n")
            }
        }
    }
    fn set_vertex_at(&mut self, idx: usize, point: P3) {
        assert!(idx < 8 && idx >= 0);
        match idx {
            0 => self.f1.p1 = point,
            1 => self.f1.p2 = point,
            2 => self.f1.p3 = point,
            3 => self.f1.p4 = point,
            4 => self.f2.p1 = point,
            5 => self.f2.p2 = point,
            6 => self.f2.p3 = point,
            7 => self.f2.p4 = point,
            _ => panic!("Strange index for Cuboid::set_vertex_at() call.\n")
        };
    }
}

/// I chose to keep Camera and VoxelGrid totally separately
/// in future I may implement a builder that takes a Camera instance 
/// this way ruf_cuboid can be populated and ruf_is_stale is false
#[derive(Debug, Copy, Clone)]
pub struct VoxelGrid {
    pub dims: Dims3,

    pub ruf_is_stale: bool, // triggered on any change to VoxelGrid or Camera

    pub world_cuboid: Cuboid, // vertices of voxel grid in worldspace

    pub ruf_cuboid: Cuboid, // vertices of voxel grid in camera basis vectors Right, Up, Forward

    pub onto_plane: [Square<[i32; 2]>; 2] // 8 3D vertices -> 8 2D vertices is 2 Squares, distinct from CuboidFace (which uses P3s)
}

/// World coordinates are always P3, but...
/// may be expressed in terms of ijk or
/// RUF... or any coord system
/// I enforce enum usage for clarity later on
/// VoxelGrid is not purely geometric - the coordinate system matters for simulation
/// and visualisation logic
impl VoxelGrid {
    pub fn new_centered_at_origin(dims: Dims3) -> Self {
        let offset_dims = [(dims[0]/2) as f32, (dims[1]/2) as f32, (dims[2]/2) as f32];

        let near_face: CuboidFace = CuboidFace {
            p1: [-offset_dims[0], -offset_dims[1], -offset_dims[2]],
            p2: [-offset_dims[0], offset_dims[1], -offset_dims[2]],
            p3: [offset_dims[0], offset_dims[1], -offset_dims[2]],
            p4: [offset_dims[0], -offset_dims[1], -offset_dims[2]]
        };

        let far_face: CuboidFace = CuboidFace {
            p1: [-offset_dims[0], -offset_dims[1], offset_dims[2]],
            p2: [-offset_dims[0], offset_dims[1], offset_dims[2]],
            p3:  [offset_dims[0], offset_dims[1], offset_dims[2]],
            p4:  [offset_dims[0], -offset_dims[1], offset_dims[2]]
        };
      
        // FOR EXAMPLE - rh coordinates looking down k,-ijk first (i major, k minor), bottom left, counterclockwise 
        Self {
            dims: dims,

            world_cuboid: Cuboid {
                f1: near_face, // centered at origin using offsets
                f2: far_face 
            },

            ruf_is_stale: true, // always true on creation

            // populated later 
            ruf_cuboid: Cuboid::default(),

            onto_plane: [Square::<[i32; 2]>::default(); 2] // 2D PROJECTION
        }
    }    
}

impl Access<SystemGet, SystemSet> for VoxelGrid {
    /// 0-indexed vertices 0-7
     fn get_vertex_at(&self, specifier: SystemGet) -> SystemSet { // can return P3 or P2, so using SystemSet variants
        match specifier {
            SystemGet::WORLD(vertex) => {
                assert!(vertex < 8 && vertex >= 0);
                SystemSet::WORLD(self.world_cuboid.get_vertex_at(vertex))
            },
            SystemGet::RUF(vertex) => {
                assert!(vertex < 8 && vertex >= 0);
                SystemSet::RUF(self.ruf_cuboid.get_vertex_at(vertex))
            },
            SystemGet::SQUARE(vertex) => {
                assert!(vertex < 8 && vertex >= 0); //  still bound at 7
                let face = vertex / 4; // always floored, so 0 or 1
                let rem = vertex % 4; // always 0, 1, 2 or 3
                SystemSet::SQUARE(self.onto_plane[face].get_vertex_at(rem))
            },
        }   
    }
    /// Handy to enforce that SystemGet and SystemSet are the same for a given call to VoxelGrid::set_vertex_at
    fn set_vertex_at(&mut self, idx: SystemGet, specifier: SystemSet) {

        match (idx, specifier) {
            (SystemGet::WORLD(idx),SystemSet::WORLD(point)) => {
                assert!(idx < 8 && idx >= 0);
                self.world_cuboid.set_vertex_at(idx, point);
            },
            (SystemGet::RUF(idx),SystemSet::RUF(point)) => {
                assert!(idx < 8 && idx >= 0);
                self.ruf_cuboid.set_vertex_at(idx, point);
            },
            (SystemGet::SQUARE(idx),SystemSet::SQUARE(point)) => {
                assert!(idx < 8 && idx >= 0);
                let face = idx / 4; // always floored, so 0 or 1
                let rem = idx % 4; // always 0, 1, 2 or 3
                self.onto_plane[face].set_vertex_at(rem, point);
            },
            _ => panic!("SystemGet and SystemSet are for different coordinate systems. Is this a mistake?\n")
        }
    }

}