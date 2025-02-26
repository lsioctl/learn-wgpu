// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
};

struct VertexOutput {
    // the value we want as clip coordinates
    // equivalent to gl_Position in GLSL
    // in the fragment shader, this value is framebuffer coordinates
    // i.e top-left corner at (0, 0)
    // so convenient for pixel coordinate in the buffer
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    // var declared variables are mutable but must be explicetly typed
    var out: VertexOutput;
    // let declarded variables are immutable and type could be inferred
   
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.tex_coords = model.tex_coords;

    return out;
}

// Fragment shader

// uniforms
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@fragment
// @location(0) tells WebGPU to store the value
// returned in the first color target
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}

