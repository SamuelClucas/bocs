
use crate::{
    backend_admin::gpu::gfx_context::GraphicsContext,
    world::{voxel_grid::VoxelGrid, world::BoundingBox}
};
use rand::Rng;

pub type DispatchDims = [u32; 3];
pub type GroupDims3 = [u32; 3]; // IDENTICAL TO DISPATCHDIMS ONLY IN FORM, one is WGSL-side group dimensions, the other is dispatch dimensions
pub type GroupDims2 = [u32; 2];

const RAYMARCH_GROUPS: GroupDims2 = [16, 16]; 
const LAPLACIAN_GROUPS: GroupDims3 = [8, 4, 8]; // 256 is max x * y * z

#[derive(Debug)]
pub struct Bridge {
    pub raymarch_dispatch: DispatchDims,

    pub laplacian_dispatch: DispatchDims,
    pub init_dispatch: DispatchDims, // THIS IS THE SAME AS LAPLACIAN, but added as a separate field for clarity in state::render()

    pub rand_seed: u32
}

impl Bridge {
    pub fn new(voxel_grid: &VoxelGrid, gfx_context: &GraphicsContext) -> Self {
        let (w, h) = (gfx_context.surface_config.width, gfx_context.surface_config.height);

        // Raymarch dispatch config is essentially 2D 
        assert!(RAYMARCH_GROUPS[0] > 0 && RAYMARCH_GROUPS[1] > 0);
        let raymarch_dispatch: DispatchDims = [ // TODO: COMPUTE THIS ONLY AFTER BOUNDING BOX HAS BEEN GENERATED FOR FIRST PASS
            w.div_ceil(RAYMARCH_GROUPS[0]),
            h.div_ceil(RAYMARCH_GROUPS[1]),
            1
        ];

        let laplacian_dispatch: DispatchDims = [
            voxel_grid.dims[0].div_ceil(LAPLACIAN_GROUPS[0]),
            voxel_grid.dims[1].div_ceil(LAPLACIAN_GROUPS[1]),
            voxel_grid.dims[2].div_ceil(LAPLACIAN_GROUPS[2])
        ];

        let seed = rand::rng().random::<u32>();

        Bridge {
            raymarch_dispatch: raymarch_dispatch,

            laplacian_dispatch: laplacian_dispatch,
            init_dispatch: laplacian_dispatch,

            rand_seed: seed
        }
    }

    /// Determines dispatch dims on each render() 
    pub fn update_raymarch_dispatch(&mut self, bounding_box: BoundingBox){
        let (w, h) = (
            (bounding_box[1][0] - bounding_box[0][0]) as u32, 
            (bounding_box[1][1] - bounding_box[0][1]) as u32
        );  

        assert!(w > 0 && h > 0); // OrbitalCamera implementation should always have voxel grid in view, so should never be 0
        
        self.raymarch_dispatch = [
            w.div_ceil(RAYMARCH_GROUPS[0]),
            h.div_ceil(RAYMARCH_GROUPS[1]),
            1
        ];
    }
}