use std::iter;

use winit::{
    event::*,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

// for create_buffer_init, use an extension trait
use wgpu::util::DeviceExt;

use crate::{camera::*, mytexture::*, vertex::*};

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: &'a Window,
    render_pipeline_triangle_interpol_buffer: wgpu::RenderPipeline,
    render_pipeline_triangle_interpol: wgpu::RenderPipeline,
    use_color: bool,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: wgpu::BindGroup,
    camera: Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl<'a> State<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();

        // The instance is the first thing we instantiate in WGPU
        // it'll handle the surface and the adapter

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        // adapter is a handle to the actual GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        //println!("{:?}", surface_caps);

        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        let texture = MyTexture::new(&device, &queue, include_bytes!("textures/happy-tree.png"));

        // a bind group describes a set of ressources and how they are accessed by a shader
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    // only visible to the fs
                    // possible values bitwise combinations
                    // of NONE, VERTEX, FRAGMENT, COMPUTE
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    // only visible to the fs
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    // This should match the filterable field of the
                    // corresponding Texture entry above.
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

        // This may seem not very DRY
        // BindGroup is a more specific declaration of the bind group layout
        // this pattern allows us to swap BindGroups on the fly as long as they have the same layout
        let diffuse_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.diffuse_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.diffuse_sampler),
                },
            ],
            label: Some("diffuse_bind_group"),
        });

        // a macro could also be used
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader_triangle = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/shader_triangle_interpol_buffer.wgsl").into(),
            ),
        });

        let camera = Camera::new(&config);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera.get_uniform()]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout, &camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline_triangle_interpol_buffer =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_triangle,
                    entry_point: Some("vs_main"),
                    // what type of vertices we want to pass to the vertex shader
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                // fragment is optional so it's in an Option
                // we need it as we want to store color data on the surface
                fragment: Some(wgpu::FragmentState {
                    module: &shader_triangle,
                    entry_point: Some("fs_main"),
                    // what color output it should set up
                    // currently we only need one for the surface
                    targets: &[Some(wgpu::ColorTargetState {
                        // use the surface's format so copying is easy
                        format: config.format,
                        // blending should replace old pixel data with new data
                        blend: Some(wgpu::BlendState::REPLACE),
                        // write all colors: rgb and alpha
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    // every three vertices will correspond to one triangle
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // front facing triangles are when vertices are given
                    // in counter clock-wise order
                    front_face: wgpu::FrontFace::Ccw,
                    // back facing triangles are not rendered
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    // only one sample
                    count: 1,
                    // which sample will be active (all of them, i.e one)
                    mask: !0,
                    // anti-aliasing related
                    alpha_to_coverage_enabled: false,
                },
                // we will not render to array textures
                multiview: None,
                // cache shader compilation data. TODO: why "only really useful for Android build target" ?
                cache: None,
            });

        // a macro could also be used
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader_triangle_interpol = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Color"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("shaders/shader_triangle_interpol.wgsl").into(),
            ),
        });

        let render_pipeline_triangle_interpol =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader_triangle_interpol,
                    entry_point: Some("vs_main"),
                    // what type of vertices we want to pass to the vertex shader
                    // for now it's specified in the shader itself
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                // fragment is optional so it's in an Option
                // we need it as we want to store color data on the surface
                fragment: Some(wgpu::FragmentState {
                    module: &shader_triangle_interpol,
                    entry_point: Some("fs_main"),
                    // what color output it should set up
                    // currently we only need one for the surface
                    targets: &[Some(wgpu::ColorTargetState {
                        // use the surface's format so copying is easy
                        format: config.format,
                        // blending should replace old pixel data with new data
                        blend: Some(wgpu::BlendState::REPLACE),
                        // write all colors: rgb and alpha
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    // every three vertices will correspond to one triangle
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    // front facing triangles are when vertices are given
                    // in counter clock-wise order
                    front_face: wgpu::FrontFace::Ccw,
                    // back facing triangles are not rendered
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    // only one sample
                    count: 1,
                    // which sample will be active (all of them, i.e one)
                    mask: !0,
                    // anti-aliasing related
                    alpha_to_coverage_enabled: false,
                },
                // we will not render to array textures
                multiview: None,
                // cache shader compilation data. TODO: why "only really useful for Android build target" ?
                cache: None,
            });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
            render_pipeline_triangle_interpol_buffer,
            render_pipeline_triangle_interpol,
            use_color: false,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            camera,
            camera_buffer,
            camera_bind_group,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    //#[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(KeyCode::Space),
                        ..
                    },
                ..
            } => {
                if *state == ElementState::Released {
                    self.use_color = !self.use_color
                };
                true
            }
            _ => false,
        }
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // wait for the surface to provide a surface texture to write to
        let output = self.surface.get_current_texture()?;

        // create a texture view with default settings
        // we need this because we want to control how the render interacts with this
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Actual commands sent to the GPU
        // Mots modern graphic frameworks need commands to be stored
        // in a buffer before being sent to the GPU
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // create a scope so we can call after encoder.finish()
        // as begin_render_pass borrows encoder mutably
        // we could also replace braces by drop(render_pass)
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // tells where we are drawing our colors to
                // we only supply in the array the render target that we care about
                color_attachments: &[
                    // this is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        // we use the texture view we created earlier to ensure we render to the screen
                        view: &view,
                        // texture that will receive the resolved output
                        // Same as view unless multisampling is enabled
                        // we don't need this
                        resolve_target: None,
                        // tells the GPU what to do with the colors on the screen (the one specified by view)
                        ops: wgpu::Operations {
                            // load tells the GPU how to handle the colors stored from the previous frame
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.1,
                                g: 0.2,
                                b: 0.3,
                                a: 1.0,
                            }),
                            // we want to store our render results to the texture behind the texture view
                            // (in our case the SurfaceTexture)
                            store: wgpu::StoreOp::Store,
                        },
                    }),
                ],
                depth_stencil_attachment: None,
                // TODO: not in documentation but in source code
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            if self.use_color == true {
                render_pass.set_pipeline(&self.render_pipeline_triangle_interpol);
            } else {
                render_pass.set_pipeline(&self.render_pipeline_triangle_interpol_buffer);
            }

            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_bind_group(1, &self.camera_bind_group, &[]);
            // slice(..) means we use the entier buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // tells WebGPU to draw something with 3 vertices and 1 instance
            // this is where in the shader @builtin(vertex_index) comes from
            // render_pass.draw(0..3, 0..1);
            // You can only have one index buffer set at a time
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // The draw method ignores the index buffer
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        // finish the command buffer and send it
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
