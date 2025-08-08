struct Uniforms{
    @align(16)
    cam_pos: vec4<f32>,
    forward: vec4<f32>,
    up: vec4<f32>,
    right: vec4<f32>,
    timestep: vec4<f32>,
    seed: vec4<f32>
}
@group(0) @binding(1)
var<storage, read_write> grid: array<f32>;

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@compute @workgroup_size(8, 4, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let prn: f32 = fract(sin(uniforms.seed[0] + f32(gid.x)) * 523969.3496);

    let idx: u32 = gid.x + (gid.y * 200u) + (gid.z * 40000u);

    grid[idx] = prn;

}

