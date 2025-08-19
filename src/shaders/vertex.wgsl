struct VertexShaderOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>
};

@vertex
fn main(@builtin(vertex_index) vertex_index: u32) -> VertexShaderOutput {
    // full-screen quad covering clip space
    let pos = array(
        vec2f(-1.0,  1.0), // top-left
        vec2f(-1.0, -1.0), // bottom-left
        vec2f( 1.0,  1.0), // top-right
        vec2f(-1.0, -1.0), // bottom-left
        vec2f( 1.0, -1.0), // bottom-right
        vec2f( 1.0,  1.0)  // top-right
    );

    let uv = array(
        vec2f(0.0, 0.0), // match top-left
        vec2f(0.0, 1.0),
        vec2f(1.0, 0.0),
        vec2f(0.0, 1.0),
        vec2f(1.0, 1.0),
        vec2f(1.0, 0.0)
    );

    var out: VertexShaderOutput;
    out.position = vec4f(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    return out;
}
