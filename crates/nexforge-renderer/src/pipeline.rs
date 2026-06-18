use crate::camera::Camera;
use crate::scene::Scene;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("Failed to create WGPU adapter")]
    AdapterCreationFailed,
    #[error("Failed to create WGPU device")]
    DeviceCreationFailed,
    #[error("Surface error: {0}")]
    SurfaceError(String),
    #[error("Shader compilation error: {0}")]
    ShaderCompilation(String),
    #[error("Pipeline creation error")]
    PipelineCreationFailed,
}

pub struct RenderContext<'a> {
    pub surface: Option<wgpu::Surface<'a>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub size: (u32, u32),
    pub depth_texture: Option<wgpu::Texture>,
    pub depth_view: Option<wgpu::TextureView>,
    pub scene: Option<Scene>,
    pub camera: Camera,
}

impl<'a> RenderContext<'a> {
    pub fn new(aspect: f32) -> Self {
        Self {
            surface: None,
            device: None,
            queue: None,
            config: None,
            size: (1920, 1080),
            depth_texture: None,
            depth_view: None,
            scene: None,
            camera: Camera::new(aspect),
        }
    }

    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn initialize(&mut self, window: &'a winit::window::Window) -> Result<(), RenderError> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);
        self.size = (width, height);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });
        self.surface = Some(
            instance
                .create_surface(window)
                .map_err(|e| RenderError::SurfaceError(e.to_string()))?,
        );
        let surface = self
            .surface
            .as_ref()
            .ok_or_else(|| RenderError::SurfaceError("Surface not initialized".to_string()))?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
        }))
        .ok_or(RenderError::AdapterCreationFailed)?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Nexforge Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        ))
        .map_err(|_| RenderError::DeviceCreationFailed)?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) {
            wgpu::PresentMode::Mailbox
        } else {
            wgpu::PresentMode::Fifo
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);
        let scene = Scene::new(&device, &config);

        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.depth_texture = Some(depth_texture);
        self.depth_view = Some(depth_view);
        self.scene = Some(scene);
        Ok(())
    }

    pub fn update_scene(&mut self, dt: f64) {
        if let Some(ref mut scene) = self.scene {
            scene.update(dt);
        }
    }

    pub fn render(&mut self, vp_matrix: [[f32; 4]; 4]) -> Result<(), RenderError> {
        let surface = self.surface.as_ref();
        let device = self.device.as_ref();
        let queue = self.queue.as_ref();
        let depth_view = self.depth_view.as_ref();
        if let (Some(surface), Some(device), Some(queue), Some(depth_view)) = (surface, device, queue, depth_view) {
            let output = surface
                .get_current_texture()
                .map_err(|e| RenderError::SurfaceError(e.to_string()))?;
            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Main Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.02,
                                g: 0.04,
                                b: 0.08,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                if let Some(ref scene) = self.scene {
                    scene.render(queue, &mut pass, vp_matrix);
                }
            }
            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }
        Ok(())
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        if let (Some(config), Some(device), Some(ref surface)) = (&mut self.config, &self.device, &self.surface) {
            config.width = new_size.0.max(1);
            config.height = new_size.1.max(1);
            surface.configure(device, config);
            let (depth_texture, depth_view) = Self::create_depth_texture(device, new_size.0.max(1), new_size.1.max(1));
            self.depth_texture = Some(depth_texture);
            self.depth_view = Some(depth_view);
        }
    }

    pub fn is_initialized(&self) -> bool {
        self.device.is_some()
    }
}

impl<'a> Default for RenderContext<'a> {
    fn default() -> Self {
        Self::new(16.0 / 9.0)
    }
}

impl<'a> std::fmt::Debug for RenderContext<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderContext").field("size", &self.size).finish()
    }
}
