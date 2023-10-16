// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>
};

struct VertexOutput {
    // the value we want as clip coordinates
    // equivalent to gl_Position in GLSL
    // in the fragment shader, this value is framebuffer coordinates
    // i.e top-left corner at (0, 0)
    // so convenient for pixel coordinate in the buffer
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput
) -> VertexOutput {
    // var declared variables are mutable but must be explicetly typed
    var out: VertexOutput;
    // let declarded variables are immutable and type could be inferred
   
    out.clip_position = vec4<f32>(model.position, 1.0);
    out.color = model.color;

    return out;
}

// Fragment shader

@fragment
// @location(0) tells WebGPU to store the value
// returned in the first color target
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

