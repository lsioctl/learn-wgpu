// Vertex shader

struct VertexOutput {
    // the value we want as clip coordinates
    // equivalent to gl_Position in GLSL
    // in the fragment shader, this value is framebuffer coordinates
    // i.e top-left corner at (0, 0)
    // so convenient for pixel coordinate in the buffer
    @builtin(position) clip_position: vec4<f32>,
    // but if we want to keep the position coordinates we have to pass them
    // separately:
    @location(0) vert_pos: vec3<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    // var declared variables are mutable but must be explicetly typed
    var out: VertexOutput;
    // let declarded variables are immutable and type could be inferred
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    // keep the position coordinates
    out.vert_pos = out.clip_position.xyz;
    return out;
}

// Fragment shader

@fragment
// @location(0) tells WebGPU to store the value
// returned in the first color target
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.2, 0.1, 1.0);
}

