use crate::{backend_admin::gpu::gfx_context::GraphicsContext, world::{camera::OrbitalCamera, voxel_grid::{P2i, Square, Access, SystemGet, SystemSet, VoxelGrid, Dims3, P3}}};

/// Manages all World entities
pub struct World {
    pub voxel_grid: VoxelGrid,
    pub camera: OrbitalCamera
}

pub type BoundingBox = [P2i; 2];

impl World {
    pub fn new(d: Dims3, gfx_ctx: &GraphicsContext) -> Self {
        assert!(d[0] > 0 && d[1] > 0 && d[2] > 0);
        let cam_init: P3 = [d[0] as f32 * 2.0, 0.0, 0.0];
        World {
            voxel_grid: VoxelGrid::new_centered_at_origin(d),
            camera: OrbitalCamera::new(cam_init, &gfx_ctx.size)
        }
    }

    /// Projects 8 P3 vertices of VoxelGrid onto camera's near plane as 4 P2s
    /// This is the minimum enclosing square for the voxel_grid (bounding box)
    pub fn generate_bb_projection(&self, gfx_ctx: &GraphicsContext) -> BoundingBox {
        // First: convert from pixels into world units
        let (w, h) = (gfx_ctx.size.width as i32, gfx_ctx.size.height as i32);

        let centre_top = h / 2; // 1:1 vertical pixels and up vector
        let r_sf = (w / 2) / centre_top; // Right vector's scaling factor from pixels to world garantees FOV 90 in vertical

        println!("Right scale: {}", r_sf);
        let centre_right = centre_top * r_sf; // Opp = tan(theta) * adj
        
        let bounding_box = { 
            let mut max_r = f32::NEG_INFINITY;
            let mut max_u = f32::NEG_INFINITY;
            let mut min_r = f32::INFINITY;
            let mut min_u = f32::INFINITY;

            for i in 0..8 {
                // VOXEL VERTICES INTO RUF
                match self.voxel_grid.get_vertex_at(SystemGet::WORLD(i)) {
                    SystemSet::WORLD(point) => {
                        let ruf_point = self.camera.world_to_ruf(&point);
                        self.voxel_grid.set_vertex_at(SystemGet::RUF(i), SystemSet::RUF(ruf_point))
                    },
                    _ => { println!("Couldn't get voxelgrid WORLD vertex.\n"); } }
                // PROJECT ONTO NEAR PLANE
                match self.voxel_grid.get_vertex_at(SystemGet::RUF(i)) {
                    SystemSet::RUF(point) => {
                        let projection = self.camera.ruf_to_ru_plane(&point, &(r_sf as f32));
                        self.voxel_grid.set_vertex_at(SystemGet::SQUARE(i), SystemSet::SQUARE(projection));
                    },
                    _ => { println!("Couldn't get voxelgrid RUF vertex.\n"); }
                }
                // COMPUTE BOUNDING BOX
                match self.voxel_grid.get_vertex_at(SystemGet::SQUARE(i)) {
                    SystemSet::SQUARE(point) => {
                        max_r = (point[0] as f32).max(max_r);
                        max_u = (point[1] as f32).max(max_u);
                        min_r = (point[0] as f32).min(min_r);
                        min_u = (point[1] as f32).min(min_u);
                    },
                    _ => { println!("Couldn't get voxelgrid SQUARE vertex.\n"); }
                }
            }

            [
                [(min_r - 1.0).max(-centre_right as f32) as i32, (min_u - 1.0).max(-centre_top as f32) as i32], 
                [(max_r + 1.0).min(centre_right as f32) as i32, (max_u + 1.0).min(centre_top as f32) as i32]
            ]
        };

        bounding_box

    }
}