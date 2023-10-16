#[repr(C)]
// Pod: plain old data, data can be accessed as &[u8]
// Zeroable indactes we can use std::mem::zeroed
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

// Counter clock-wise (we are drawing only front-facing)
// TODO: dependency with state.rs
// front_face: wgpu::FrontFace::Ccw,
pub const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

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
                     // tells the shader it is a vec3<f32>
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }
}




