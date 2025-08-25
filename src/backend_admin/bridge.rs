
use crate::{
    backend_admin::gpu::gfx_context::GraphicsContext,
    world::voxel_grid::VoxelGrid
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

            rand_seed: seed
        }
    }

    /// Determines dispatch dims on each render() 
    pub fn update_raygroup_ceil(&mut self, bounding_box: [i32; 4]) -> (u32, u32) {
        let width = bounding_box[2] - bounding_box[0];  
        let height = bounding_box[3] - bounding_box[1];
        self.w_ceil = if width % self.raymarch_group == 0 { 0 }
            else { 1 };
        self.h_ceil = if height % self.raymarch_group == 0 { 0 } 
            else { 1 };
        
        (
            ((width / self.raymarch_group) + self.w_ceil) as u32, 
            ((height / self.raymarch_group) + self.h_ceil) as u32
        )
    }
}