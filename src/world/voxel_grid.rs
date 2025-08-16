
pub struct VoxelDims {
    pub i: u32,
    pub j: u32,
    pub k: u32,
}

pub struct VoxelVertices {
    pub world_vertices: [f32; 3 * 8],
    pub ruf_vertices: [f32; 3 * 8],
    pub onto_plane: [f32; 2 * 8]
}
#[derive(Copy, Clone)]
pub enum SystemSet{
    WORLD([f32;3]),
    RUF([f32;3]),
    PLANE([f32;2]),
}
#[derive(Copy, Clone)]
pub enum SystemGet{
    WORLD,
    RUF,
    PLANE,
}

impl VoxelVertices {
    pub fn  centre_at_origin(dims: &VoxelDims) -> Self {
        let i = (dims.i as f32) / 2.0;
        let j = (dims.j as f32) / 2.0;
        let k = (dims.k as f32) / 2.0;

        // rh coordinates looking down k,-ijk first (i major, k minor), bottom left, counterclockwise
        Self {
            world_vertices: [
            -i, -j, -k, 
            -i, j, -k, 
            i, j, -k,
            i, -j, -k, // face in -k ij plane 

            -i, -j, k,
            -i, j, k,
            i, j, k,
            i, -j, k],    // face in k ij plane
            ruf_vertices: [0.0; 3*8],
            onto_plane: [0.0; 2*8]
        }
    }    
    /// 0-indexed vertices 0-7
    pub fn get_point(&self, point: usize, specifier: SystemGet) -> SystemSet {
        match specifier {
            SystemGet::WORLD => {
                assert!(point <= 7);
                SystemSet::WORLD([self.world_vertices[point * 3], 
                self.world_vertices[(point * 3) as usize + 1], 
                self.world_vertices[(point * 3) as usize + 2]
                ])
            },
            SystemGet::RUF => {
                assert!(point <= 7);
                SystemSet::RUF([self.ruf_vertices[point * 3], 
                self.ruf_vertices[(point * 3) + 1], 
                self.ruf_vertices[(point * 3) + 2]
                ])
            },
            SystemGet::PLANE => {
                assert!(point <= 7);
                SystemSet::PLANE([self.onto_plane[point * 2], 
                self.onto_plane[(point * 2) + 1] 
                ])
            },
        }   
    }

    pub fn set_point(&mut self, point: usize, specifier: SystemSet){
        match specifier {
            SystemSet::WORLD(slice) => {
                assert!(point <= 7);
                self.world_vertices[point * 3] = slice[0];
                self.world_vertices[(point * 3) + 1] = slice[1];
                self.world_vertices[(point * 3) + 2] = slice[2];
                
            },
            SystemSet::RUF(slice) => {
                assert!(point <= 7);
                self.ruf_vertices[point * 3] = slice[0];
                self.ruf_vertices[(point * 3) + 1] = slice[1];
                self.ruf_vertices[(point * 3) + 2] = slice[2];
            },
            SystemSet::PLANE(slice) => {
                assert!(point <= 7);
                self.onto_plane[point * 2] = slice[0];
                self.onto_plane[(point * 2) + 1] = slice[1];
            },
        }
    }
}