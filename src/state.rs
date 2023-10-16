use std::iter;

use winit::{
    event::*,
    window::Window
};

// for create_buffer_init, use an extension trait
use wgpu::util::DeviceExt;

use crate::vertex::*;

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
    render_pipeline_triangle_interpol_buffer: wgpu::RenderPipeline,
    render_pipeline_triangle_interpol: wgpu::RenderPipeline,
    use_color: bool,
    vertex_buffer: wgpu::Buffer
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is the first thing we instantiate in WGPU
        // it'll handle the surface and the adapter

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(
            wgpu::InstanceDescriptor {
                backends: wgpu::Backends::all(),
                dx12_shader_compiler: Default::default(),
            }
        );

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        // adapter is a handle to the actual GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                // two variants: LowPower and HighPerformance
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                // Some(&std::path::Path::new("trace")), // Trace path
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        //println!("{:?}", surface_caps);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // How to sync the surface with the display
            // Fifo will cap the rate at the display's framerate
            // This mode is guaranteed to be supported in all platforms
            //present_mode: surface_caps.present_modes[0],
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        // a macro could also be used
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader_triangle = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_triangle_interpol_buffer.wgsl").into()),
        });
        
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
        });

        let render_pipeline_triangle_interpol_buffer = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_triangle,
                entry_point: "vs_main",
                // what type of vertices we want to pass to the vertex shader
                buffers: &[
                    Vertex::desc()
                ],
            },
            // fragment is optional so it's in an Option
            // we need it as we want to store color data on the surface
            fragment: Some(wgpu::FragmentState {
                module: &shader_triangle,
                entry_point: "fs_main",
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
        });

        // a macro could also be used
        // let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let shader_triangle_interpol = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Color"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader_triangle_interpol.wgsl").into()),
        });

        let render_pipeline_triangle_interpol = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_triangle_interpol,
                entry_point: "vs_main",
                // what type of vertices we want to pass to the vertex shader
                // for now it's specified in the shader itself
                buffers: &[],
            },
            // fragment is optional so it's in an Option
            // we need it as we want to store color data on the surface
            fragment: Some(wgpu::FragmentState {
                module: &shader_triangle_interpol,
                entry_point: "fs_main",
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
        });

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

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
            vertex_buffer
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
                input:
                    KeyboardInput {
                        state,
                        virtual_keycode: Some(VirtualKeyCode::Space),
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
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            if self.use_color == true {
                render_pass.set_pipeline(&self.render_pipeline_triangle_interpol);
            } else {
                render_pass.set_pipeline(&self.render_pipeline_triangle_interpol_buffer);
            }

            // slice(..) means we use the entier buffer
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // tells WebGPU to draw something with 3 vertices and 1 instance
            // this is where in the shader @builtin(vertex_index) comes from
            render_pass.draw(0..3, 0..1); // 3.
        }

        // finish the command buffer and send it
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}