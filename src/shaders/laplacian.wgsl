struct Uniforms{
    mid_window: vec4<u32>,
    dims: vec4<u32>, // i, j, k, k stride
    bounding_box: vec4<i32>,
    cam_pos: vec4<f32>,
    forward: vec4<f32>,
    centre: vec4<f32>, // some k*forward
    up: vec4<f32>,
    right: vec4<f32>, // [3] horizontal scaling factor (not needed for up, 1:1)
    timestep: vec4<f32>, // [0] time in seconds
    seed: vec4<f32>,
    flags: vec4<u32> // [0] reada flag 1 true, 0 false
}
// BINDINGS

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read_write> grid_a: array<f32>;

@group(0) @binding(2)
var<storage, read_write> grid_b: array<f32>;

@group(0) @binding(3)
var output_tex: texture_storage_2d<rgba8unorm, write>; 

// CONSTS AND SHARED MEMORY
const group_x: u32 = 8;
const group_y: u32 = 4;
const group_z: u32 = 8;

const shared_x: u32 = group_x + 2;
const shared_y: u32 = group_y + 2;
const shared_z: u32 = group_z + 2;

var<workgroup> shared_cells: array<f32, shared_x * shared_y * shared_z>;


// COLLABORATIVE LOADING AND LAPLACIAN STENCIL
// WORKGROUP DIMS + 2 = SHARED MEMORY CUBOID WITH HALO
// TODO: ADD PERIODIC SWAP FOR NEUMANN ON KEYPRESS RUST-SIDE, LOGIC DIVERGENCE HERE
@compute @workgroup_size(group_x, group_y, group_z)
fn laplacian(@builtin(global_invocation_id) gid: vec3<u32>, @builtin(local_invocation_id) loc: vec3<u32>, @builtin(workgroup_id) gro: vec3<u32>){
    // ALL THREADS IN DOMAIN TO FETCH INNER CELLS
    let global_y_stride = gid.y * uniforms.dims[0];
    let global_z_stride = gid.z * uniforms.dims[3];
    let idx: u32 = gid.x + global_y_stride + global_z_stride; // index for inner cells
    if gid.x < uniforms.dims[0] && gid.y < uniforms.dims[1] && gid.z < uniforms.dims[2] {
        if uniforms.flags[0] == 1{
            let middle_voxel = grid_a[idx];
            
            // insert inner cell float
            shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = middle_voxel; // +1 for halo offset, using xyz + 2 for stride calcs
            
            // FACE THREADS TO FETCH HALOS
            // X HALOS
            if gid.x == uniforms.dims[0] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch x in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 2 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x + 2 in shared for Neumann bound
            
            }
            else if loc.x == group_x - 1  && gid.x + 1 < uniforms.dims[0] { // catches clean tiles before final x bound tile which still needs halo
                let idx: u32 = gid.x + 1 + global_y_stride + global_z_stride; // fetch x + 1 in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 2 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x + 2 in shared
            
            }
            else if loc.x == 0 && gid.x > 0  { // && gid.x < uniforms.dims[0] already assured
                let idx: u32 = gid.x - 1 + global_y_stride + global_z_stride; // fetch x - 1  in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x in shared
                
            }
            else if gid.x == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch x  in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared for Neumann bound
                }

                // Y HALOS
            if gid.y == uniforms.dims[1] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 2) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y + 2 in shared for Neumann bound
            
            }
            else if loc.y == group_y - 1  && gid.y + 1 < uniforms.dims[1] { // catches clean tiles before final y bound tile which still needs halo
                let global_y_stride = (gid.y + 1) * uniforms.dims[0]; // recompute y + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y + 1 in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 2) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y + 2 in shared
            
            }
            else if loc.y == 0 && gid.y > 0  { // && gid.y < uniforms.dims[1] already assured
                let global_y_stride = (gid.y -1) * uniforms.dims[0]; // recompute y - 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y - 1  in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + (loc.y * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared
                
            }
            else if gid.y == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y  in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + (loc.y * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared for Neumann bound
                }
                // Z HALOS
            if gid.z == uniforms.dims[2] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 2) * shared_x * shared_y)] = halo_cell; // write z + 2 in shared for Neumann bound
            
            }
            else if loc.z == group_z - 1  && gid.z + 1 < uniforms.dims[2] { // catches clean tiles before final y bound tile which still needs halo
                let global_z_stride = (gid.z + 1) * uniforms.dims[3]; // recompute z + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z + 1 in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 2) * shared_x * shared_y)] = halo_cell; // write z + 2 in shared
            
            }
            else if loc.z == 0 && gid.z > 0 { // && gid.z < uniforms.dims[2] already assured
                let global_z_stride = (gid.z - 1) * uniforms.dims[3]; // recompute z + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z - 1  in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + (loc.z * shared_x * shared_y)] = halo_cell; // write z in shared
                
            }
            else if gid.z == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z in global

                let halo_cell = grid_a[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) *shared_x) + (loc.z * shared_x * shared_y)] = halo_cell; // write z in shared for Neumann bound
                }
        } // READ GRID A
        else {
            let middle_voxel = grid_b[idx];
            
            // insert inner cell float
            shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = middle_voxel; // +1 for halo offset, using xyz + 2 for stride calcs
            
            // FACE THREADS TO FETCH HALOS
            // X HALOS
            if gid.x == uniforms.dims[0] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch x in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 2 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x + 2 in shared for Neumann bound
            
            }
            else if loc.x == group_x - 1  && gid.x + 1 < uniforms.dims[0] { // catches clean tiles before final x bound tile which still needs halo
                let idx: u32 = gid.x + 1 + global_y_stride + global_z_stride; // fetch x + 1 in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 2 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x + 2 in shared
            
            }
            else if loc.x == 0 && gid.x > 0  { // && gid.x < uniforms.dims[0] already assured
                let idx: u32 = gid.x - 1 + global_y_stride + global_z_stride; // fetch x - 1  in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write x in shared
                
            }
            else if gid.x == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch x  in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared for Neumann bound
                }


                // Y HALOS
            if gid.y == uniforms.dims[1] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 2) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y + 2 in shared for Neumann bound
            
            }
            else if loc.y == group_y - 1  && gid.y + 1 < uniforms.dims[1] { // catches clean tiles before final y bound tile which still needs halo
                let global_y_stride = (gid.y + 1) * uniforms.dims[0]; // recompute y + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y + 1 in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 2) * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y + 2 in shared
            
            }
            else if loc.y == 0 && gid.y > 0  { // && gid.y < uniforms.dims[1] already assured
                let global_y_stride = (gid.y -1) * uniforms.dims[0]; // recompute y - 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y - 1  in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + (loc.y * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared
                
            }
            else if gid.y == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch y  in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + (loc.y * shared_x) + ((loc.z + 1) * shared_x * shared_y)] = halo_cell; // write y in shared for Neumann bound
                }

                // Z HALOS
            if gid.z == uniforms.dims[2] - 1 { // catches both short tiles and clean tiles
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 2) * shared_x * shared_y)] = halo_cell; // write z + 2 in shared for Neumann bound
            
            }
            else if loc.z == group_z - 1  && gid.z + 1 < uniforms.dims[2] { // catches clean tiles before final y bound tile which still needs halo
                let global_z_stride = (gid.z + 1) * uniforms.dims[3]; // recompute z + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z + 1 in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 2) * shared_x * shared_y)] = halo_cell; // write z + 2 in shared
            
            }
            else if loc.z == 0 && gid.z > 0 { // && gid.z < uniforms.dims[2] already assured
                let global_z_stride = (gid.z - 1) * uniforms.dims[3]; // recompute z + 1 stride
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z - 1  in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) * shared_x) + (loc.z * shared_x * shared_y)] = halo_cell; // write z in shared
                
            }
            else if gid.z == 0  {
                let idx: u32 = gid.x + global_y_stride + global_z_stride; // fetch z in global

                let halo_cell = grid_b[idx];
                shared_cells[loc.x + 1 + ((loc.y + 1) *shared_x) + (loc.z * shared_x * shared_y)] = halo_cell; // write z in shared for Neumann bound
                }
        } // READ GRID_B
    }
        // all halos and inner cells loaded, OOB still arrive here
        workgroupBarrier();
    if gid.x < uniforms.dims[0] && gid.y < uniforms.dims[1] && gid.z < uniforms.dims[2] {

        let idx_x = loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 1) * shared_x * shared_y);

        let idx_ymin = loc.x + 1 + (loc.y * shared_x) + ((loc.z + 1) * shared_x * shared_y);
        let idx_yplus = loc.x + 1 + ((loc.y + 2) * shared_x) + ((loc.z + 1) * shared_x * shared_y);

        let idx_zmin = loc.x + 1 + ((loc.y + 1) * shared_x) + (loc.z * shared_x * shared_y);
        let idx_zplus = loc.x + 1 + ((loc.y + 1) * shared_x) + ((loc.z + 2) * shared_x * shared_y);

        let c_i = shared_cells[ idx_x ];
        let c_i_xmin = shared_cells[ idx_x - 1 ];
        let c_i_xplus = shared_cells[ idx_x + 1 ];
        let c_i_ymin = shared_cells[ idx_ymin ];
        let c_i_yplus = shared_cells[ idx_yplus ];
        let c_i_zmin = shared_cells[ idx_zmin ];
        let c_i_zplus = shared_cells[ idx_zplus ];

        // LAPLACIAN x^2 == 1.0, D = 1.0
        let next_c_i = c_i + ((1.0 * uniforms.timestep[0] / 1.0) * ((c_i_xmin + c_i_xplus + c_i_ymin + c_i_yplus + c_i_zmin + c_i_zplus) - (6.0 * c_i)));
        if uniforms.flags[0] == 1 {
            grid_b[idx] = next_c_i;
        }
        else {
            grid_a[idx] = next_c_i;
        }
    }
}

