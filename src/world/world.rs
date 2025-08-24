use crate::{backend_admin::gpu::gfx_context::GraphicsContext, world::{camera::OrbitalCamera, voxel_grid::{Cuboid, VoxelGrid, Dims, P3}}};

/// Manages all World entities
pub struct World {
    voxel_grid: VoxelGrid,
    camera: OrbitalCamera

}

impl World {
    fn new(d: Dims, gfx_ctx: &GraphicsContext) -> Self {
        assert!(d[0] > 0 && d[1] > 0 && d[2] > 0);
        let cam_init: P3 = [d[0] * 2.0, 0.0, 0.0];
        World {
            voxel_grid: VoxelGrid::new_centered_at_origin(d),
            camera: OrbitalCamera::new(cam_init, &gfx_ctx.size)
        }
    }
}