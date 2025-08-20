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
const ray_group: u32 = 16; 


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


