@group(0) @binding(0)
var my_sampler: sampler;

@group(0) @binding(1)
var input_tex: texture_2d<f32>;

@fragment
fn main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    let flipped = vec2f(uv.x, 1.0 - uv.y);
    let p = vec2<i32>(10, 10);
    let c = textureLoad(input_tex, p, 0);
    return c;
    //return textureSample(input_tex, my_sampler, flipped);
   // return textureLoad(input_tex, vec2(10, 10), 0);
   //return vec4f(1.0, 0.0, 1.0, 1.0); // solid magenta
}
