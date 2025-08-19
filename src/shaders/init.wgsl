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

const ray_group: u32 = 16; // for raymarch only 

var<workgroup> shared_cells: array<f32, shared_x * shared_y * shared_z>;

// RANDOM INIT OF GRID_X
@compute @workgroup_size(group_x, group_y, group_z)
fn init(@builtin(global_invocation_id) gid: vec3<u32>) {
    // OOB check for when grid_n % group_n != 0 (ceiling to access all cells)
    if gid.x >= uniforms.dims[0] || gid.y >= uniforms.dims[1] || gid.z >= uniforms.dims[2] {return;}

    let prn: f32 = fract(sin(uniforms.seed[0] + f32(gid.x) + f32(gid.y) + f32(gid.z)) * 523969.3496);

    let idx: u32 = gid.x + (gid.y * uniforms.dims[0]) + (gid.z * uniforms.dims[3]);

    grid_a[idx] = prn;
}

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

@compute @workgroup_size(ray_group, ray_group)
fn raymarch(@builtin(global_invocation_id) gid: vec3<u32>) {
    // no bounds check here beucase dispatch only launched for threads in bounding box
    // first undo horizontal scaling of bounding box (later, scaled version is still used to write to texture)
    if (gid.x == 0u && gid.y == 0u) {
    textureStore(output_tex, vec2<u32>(10u, 10u), vec4f(1.0, 0.0, 0.0, 1.0));
    // no early return; but fine if you add one
    return;
    }
    let screen_to_world = vec2<f32>(
        f32(uniforms.bounding_box.x) / uniforms.right.w, // steps left from centre
        f32(uniforms.bounding_box.y) // steps down from centre
        //f32(uniforms.bounding_box[2]) / uniforms.right[3], // steps right from centre 
        //f32(uniforms.bounding_box[3]) // steps up from centre
    );
    
    // all directions accessed by some r, u addition onto screen_to_world[0] and [1]
    let plane_coord = vec2<f32>(
        screen_to_world.x + f32(gid.x),
        screen_to_world.y + f32(gid.y)
    );

    let right = uniforms.right * plane_coord.x; // both orthogonal to centre
    let up = uniforms.up * plane_coord.y;

    let direction = uniforms.centre + right + up; 

    let magnitude = sqrt(((direction.x*direction.x) + (direction.y*direction.y) + (direction.z*direction.z)));
    let norm_dir = vec3<f32>(direction.x / magnitude, direction.y / magnitude, direction.z / magnitude);

    // direction into world coords
    // dot ruf onto ijk
    let ijk_direction = vec3<f32>(
        (direction.x * uniforms.right.x) + (direction.y * uniforms.up.x) + (direction.z * uniforms.forward.x),
        (direction.x * uniforms.right.y) + (direction.y * uniforms.up.y) + (direction.z * uniforms.forward.y),
        (direction.x * uniforms.right.z) + (direction.y * uniforms.up.z) + (direction.z * uniforms.forward.z)
    );
    
    // dot norm on ijk
    let ijk_step = vec3<f32>(
        (norm_dir.x * uniforms.right.x) + (norm_dir.y * uniforms.up.x) + (norm_dir.z * uniforms.forward.x),
        (norm_dir.x * uniforms.right.y) + (norm_dir.y * uniforms.up.y) + (norm_dir.z * uniforms.forward.y),
        (norm_dir.x * uniforms.right.z) + (norm_dir.y * uniforms.up.z) + (norm_dir.z * uniforms.forward.z)
    );

    // now direction is in terms of i, j and k
    // shift into voxel space by + dims/2.0 (treat voxel grid itself in R3, indices are sampled by weighted averages of N3)
    let voxel_direction = vec3<f32>(
        ijk_direction.x + (f32(uniforms.dims.x)/2.0),
        ijk_direction.y + (f32(uniforms.dims.y)/2.0),
        ijk_direction.z + (f32(uniforms.dims.z)/2.0)
    );

    // compute entry and exit plane intersection of voxel direction + k*ijk_step
    // + dims[x] coefficients
    let k = (f32(uniforms.dims.z) - voxel_direction.z) / ijk_step.z;
    let j = (f32(uniforms.dims.y) - voxel_direction.y) / ijk_step.y;
    let i = (f32(uniforms.dims.x) - voxel_direction.x) / ijk_step.x;
    // dims[x] = 0 coefficients
    let zerok = - voxel_direction.z / ijk_step.z;
    let zeroj = - voxel_direction.y / ijk_step.y;
    let zeroi = - voxel_direction.x / ijk_step.x;

    // near plane
    let mink = min(zerok, k);
    let minj = min(zeroj, j);
    let mini = min(zeroi, i);

    let entry = max(max(mink, minj), mini);

    // far plane
    let maxk = max(zerok, k);
    let maxj = max(zeroj, j);
    let maxi = max(zeroi, i);

    let exit = min(min(maxi,maxj), maxk);

    let x_nudge = voxel_direction.x / 10; // if ray lands intersects exactly at cell boundaries
    let y_nudge = voxel_direction.y / 10;
    let z_nudge = voxel_direction.z / 10;
    let nudged_direction = vec3<f32>(voxel_direction.x + x_nudge, voxel_direction.y + y_nudge, voxel_direction.z + z_nudge); // add a little nudge

    // get entry exit coords in voxel space (ijk but offset)
    let entry_point: vec3<f32> = nudged_direction + (ijk_step * entry);
    if entry_point.x >= f32(uniforms.dims.x) || entry_point.y >= f32(uniforms.dims.y) || entry_point.z >= f32(uniforms.dims.z) { return; }
    
    let exit_point: vec3<f32> = nudged_direction + (ijk_step * exit); // handles exit plane intersection at boundary

    let entry_idx: f32 = entry_point.x + (entry_point.y * f32(uniforms.dims.x)) + (entry_point.z * f32(uniforms.dims[3]));
    
    let flat_size: f32 = f32((uniforms.dims.z * uniforms.dims[3]));
    var accumulated_values: f32 = 0.0; // MUT
    let travel_vector = exit_point - entry_point;
    let max_projection = (travel_vector.x * ijk_step.x)  + (travel_vector.y * ijk_step.y) + (travel_vector.z * ijk_step.z);

    if entry_idx < flat_size && entry_idx >= 0 { // entry idx bounds check
        let floored_entry_idx = u32(floor(entry_idx));
        
        var next_point = vec3<f32>(entry_point + ijk_step); // MUT
        var next_projection = (ijk_step.x * ijk_step.x)  + (ijk_step.y * ijk_step.y) + (ijk_step.z * ijk_step.z); // MUT
        let unit_projection = ((ijk_step.x * ijk_step.x)  + (ijk_step.y * ijk_step.y) + (ijk_step.z * ijk_step.z));
        
        if uniforms.flags.x == 1u { // read a (reading from ping, this frame computes the frame displayed on succeeding loop)
            accumulated_values = grid_a[floored_entry_idx];
            while next_projection <= max_projection {
                 if (next_point.x >= f32(uniforms.dims.x) || next_point[1] >= f32(uniforms.dims.y) || next_point.z >= f32(uniforms.dims.z) || 
                 next_point.x < 0.0 || next_point.y < 0.0 || next_point.z < 0.0) { break; }
                    let idx = u32(floor(next_point.x 
                    + next_point.y * f32(uniforms.dims.x) 
                    + next_point.z * f32(uniforms.dims[3])
                    ));
                    accumulated_values += grid_a[idx]; // how are you going to handle colour and opacity?
                    next_point += ijk_step;
                    next_projection += unit_projection;
                }
        }
        else if uniforms.flags.x == 0u { // read b (ping buffer)
            accumulated_values = grid_b[floored_entry_idx];
            while next_projection <= max_projection {
                 if (next_point.x >= f32(uniforms.dims.x) || next_point.y >= f32(uniforms.dims.y) || next_point[2] >= f32(uniforms.dims.z) ||
                 next_point.x < 0.0 || next_point.y < 0.0 || next_point.z < 0.0) { break; }
                    let idx = u32(floor(next_point.x 
                    + next_point.y * f32(uniforms.dims.x) 
                    + next_point.z * f32(uniforms.dims.w)
                    ));
                    accumulated_values += grid_b[idx]; // how are you going to handle colour and opacity?
                    next_point += ijk_step;
                    next_projection = next_projection + next_projection;
                }
        }
    }
    else { return; }

    // map accumulated value to texture coord and write
    // output_tex is rgba8unorm
    // larger accumulate, more R and A
    // I want to be able to see through the voxel cuboid mostly, so accumulate of 1.0 == A 1.0 is not a good idea
    // the cells were initialised with the fract() of a stretched out sin(seed), so the max of a cell is .99999
    
    // Using Beer-Lambert
    let o: f32 = 0.6;
    let b: f32 = 0.4;

    let alpha = clamp(1 - exp((-accumulated_values/max_projection * o)), 0.0, 1.0);
    let blue = max(1 - exp((accumulated_values * - b)), 0.0);
    let red = alpha; // for now

    var write_val= vec4<f32>(red, 0.0 , blue, alpha);

    // write to storage texture    
    let pixel_coord = vec2<i32>(uniforms.bounding_box.x + i32(gid.x), uniforms.bounding_box.y + i32(gid.y));
    let final_window_coord = vec2<u32>(u32((i32(uniforms.mid_window.x) + pixel_coord.x)), u32(i32(uniforms.mid_window.y) + pixel_coord.y));

    if final_window_coord.x < 0 || final_window_coord.y < 0 || final_window_coord.x > uniforms.mid_window.x * 2 || final_window_coord.y > uniforms.mid_window.y * 2 {
        write_val.y = 1.0; // green indicates corrupt final window coord
    }

    textureStore(output_tex, final_window_coord, write_val);
    

}

