@group(0) @binding(1)
var<storage, read_write> grid: array<f32>;

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) -> {
    let prn: f32 = fract(sin(f32(gid.x)) * 523969.3496);


}

