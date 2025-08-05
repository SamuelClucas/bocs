@group(0) @binding(0)
var<storage, read_write> grid: array<f64>;

let id = global_invocation_id;

@compute @workgroup_size(8, 8, 8)
fn main() -> {}

