#[repr(C)]
// Pod: plain old data, data can be accessed as &[u8]
// Zeroable indactes we can use std::mem::zeroed
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

// Counter clock-wise (we are drawing only front-facing)
// TODO: dependency with state.rs
// front_face: wgpu::FrontFace::Ccw,
// textures coordinates have y-axis pointing down
// and wgpu world coordinates have y-axis pointing up
// so important to do 1 - y instead of y for texture y
pub const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

// Pod and Zeroable arleady implemented for basic types by bytemuck
pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            // how wide is Vertex
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            // tells the pipeline if the data is per vertex data or per instance
            step_mode: wgpu::VertexStepMode::Vertex,
            // here a macro exists
            // attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3]
            // but has to be tweaked as it returns a temporary
            // and we need to return from a function
            // anyway it may be nice to understand what attributes looks like
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // @location(0) x: vec3<f32> in the vertex shader will match the position
                    shader_location: 0,
                    // tells the shader it is a vec3<f32>
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    // @location(0) x: vec3<f32> in the vertex shader will match the color
                    shader_location: 1,
                    // tells the shader it is a vec2<f32>
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}
