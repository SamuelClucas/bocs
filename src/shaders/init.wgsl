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

// CONSTS
const group_x: u32 = 8;
const group_y: u32 = 4;
const group_z: u32 = 8;

// BINDINGS
@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var<storage, read_write> grid_a: array<f32>;

// RANDOM INIT OF GRID_A
@compute @workgroup_size(group_x, group_y, group_z)
fn init(@builtin(global_invocation_id) gid: vec3<u32>) {
    // OOB check for when grid_n % group_n != 0 (ceiling to access all cells)
    if gid.x >= uniforms.dims[0] || gid.y >= uniforms.dims[1] || gid.z >= uniforms.dims[2] {return;}

    let prn: f32 = fract(sin(uniforms.seed[0] + f32(gid.x) + f32(gid.y) + f32(gid.z)) * 523969.3496);

    let idx: u32 = gid.x + (gid.y * uniforms.dims[0]) + (gid.z * uniforms.dims[3]);

    grid_a[idx] = prn;
}

