use std::iter;

use winit::{
    event::*,
    window::Window
};


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

        Self {
            surface,
            device,
            queue,
            config,
            size,
            window,
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

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
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
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                // tells where we are drawing our colors to
                // we only supply in the array the render target that we care about
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
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
        }

        // finish the command buffer and send it
        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}