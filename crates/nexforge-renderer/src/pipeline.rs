#![deny(clippy::all)]

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

pub struct RenderContext {
    pub surface: Option<wgpu::Surface>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub size: (u32, u32),
}

impl RenderContext {
    pub fn new() -> Self {
        Self { surface: None, device: None, queue: None, config: None, size: (1920, 1080) }
    }

    pub fn initialize(&mut self, window: &winit::window::Window) -> Result<(), RenderError> {
        let size = window.inner_size();
        self.size = (size.width, size.height);
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: wgpu::Dx12Compiler::Fxc,
        });
        self.surface = Some(unsafe { instance.create_surface(window) }
            .map_err(|e| RenderError::SurfaceError(e.to_string()))?);
        let surface = self.surface.as_ref().unwrap();
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions { power_preference: wgpu::PowerPreference::HighPerformance, force_fallback_adapter: false, compatible_surface: Some(surface) },
        )).ok_or(RenderError::AdapterCreationFailed)?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor { label: Some("Nexforge Device"), features: wgpu::Features::empty(), limits: wgpu::Limits::default() }, None,
        )).map_err(|_| RenderError::DeviceCreationFailed)?;
        let caps = surface.get_capabilities(&adapter);
        let format = caps.formats[0];
        let present_mode = if caps.present_modes.contains(&wgpu::PresentMode::Mailbox) { wgpu::PresentMode::Mailbox } else { wgpu::PresentMode::Fifo };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT, format, width: size.width, height: size.height,
            present_mode, alpha_mode: caps.alpha_modes[0], view_formats: vec![],
        };
        surface.configure(&device, &config);
        self.device = Some(device); self.queue = Some(queue); self.config = Some(config);
        Ok(())
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        self.size = new_size;
        if let (Some(config), Some(device), Some(ref surface)) = (&mut self.config, &self.device, &self.surface) {
            config.width = new_size.0; config.height = new_size.1;
            surface.configure(device, config);
        }
    }

    pub fn is_initialized(&self) -> bool { self.device.is_some() }
}

impl Default for RenderContext { fn default() -> Self { Self::new() } }

pub struct ClearPipeline { render_pipeline: Option<wgpu::RenderPipeline> }

impl ClearPipeline {
    pub fn new() -> Self { Self { render_pipeline: None } }

    pub fn initialize(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<(), RenderError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Clear Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
                @vertex fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
                    let x = f32(i32(in_vertex_index) - 1);
                    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
                    return vec4<f32>(x, y, 0.0, 1.0);
                }
                @fragment fn fs_main() -> @location(0) vec4<f32> { return vec4<f32>(0.1, 0.2, 0.4, 1.0); }
            "#)),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Clear Pipeline Layout"), bind_group_layouts: &[], push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Clear Pipeline"), layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[] },
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", targets: &[Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL })] }),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, strip_index_format: None, front_face: wgpu::FrontFace::Ccw, cull_mode: Some(wgpu::Face::Back), polygon_mode: wgpu::PolygonMode::Fill, unclipped_depth: false, conservative: false },
            depth_stencil: None, multisample: wgpu::MultisampleState { count: 1, mask: !0, alpha_to_coverage_enabled: false }, multiview: None,
        });
        self.render_pipeline = Some(pipeline); Ok(())
    }

    pub fn render(&self, surface: &wgpu::Surface, device: &wgpu::Device, queue: &wgpu::Queue) -> Result<(), RenderError> {
        let output = surface.get_current_texture().map_err(|e| RenderError::SurfaceError(e.to_string()))?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("Clear Encoder") });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view, resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.02, g: 0.04, b: 0.08, a: 1.0 }), store: wgpu::StoreOp::Store },
                })], depth_stencil_attachment: None,
            });
            if let Some(ref pipeline) = self.render_pipeline { pass.set_pipeline(pipeline); pass.draw(0..3, 0..1); }
        }
        queue.submit(std::iter::once(encoder.finish())); output.present(); Ok(())
    }
}

impl Default for ClearPipeline { fn default() -> Self { Self::new() } }

/// Deferred geometry pass pipeline — renders position, normal, albedo, PBR to GBuffer
pub struct DeferredGeometryPipeline {
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group: Option<wgpu::BindGroup>,
}

impl DeferredGeometryPipeline {
    pub fn new() -> Self { Self { pipeline: None, bind_group: None } }

    pub fn initialize(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<(), RenderError> {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Deferred Geometry"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(r#"
                struct Uniforms { model: mat4x4<f32>, view: mat4x4<f32>, proj: mat4x4<f32>, }
                @group(0) @binding(0) var<uniform> uniforms: Uniforms;
                struct VSOut { @builtin(position) pos: vec4<f32>, @location(0) world_pos: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>, }
                @vertex fn vs_main(@location(0) pos: vec3<f32>, @location(1) normal: vec3<f32>, @location(2) uv: vec2<f32>) -> VSOut {
                    var out: VSOut; out.pos = uniforms.proj * uniforms.view * uniforms.model * vec4<f32>(pos, 1.0);
                    out.world_pos = (uniforms.model * vec4<f32>(pos, 1.0)).xyz;
                    out.normal = normalize((uniforms.model * vec4<f32>(normal, 0.0)).xyz);
                    out.uv = uv; return out;
                }
                struct GBuffer { albedo: @location(0) vec4<f32>, normal: @location(1) vec4<f32>, pbr: @location(2) vec4<f32>, }
                @fragment fn fs_main(@location(0) wpos: vec3<f32>, @location(1) nrm: vec3<f32>, @location(2) uv: vec2<f32>) -> GBuffer {
                    var gb: GBuffer; gb.albedo = vec4<f32>(0.8, 0.2, 0.2, 1.0); gb.normal = vec4<f32>(normalize(nrm), 1.0); gb.pbr = vec4<f32>(0.0, 0.5, 1.0, 0.0); return gb;
                }
            "#)),
        });
        let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Deferred Geo Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0, visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Deferred Geo Pipeline Layout"), bind_group_layouts: &[&bind_layout], push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Deferred Geometry"), layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState { module: &shader, entry_point: "vs_main", buffers: &[
                wgpu::VertexBufferLayout { array_stride: 32, step_mode: wgpu::VertexStepMode::Vertex, attributes: &[
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 0, shader_location: 0 },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x3, offset: 12, shader_location: 1 },
                    wgpu::VertexAttribute { format: wgpu::VertexFormat::Float32x2, offset: 24, shader_location: 2 },
                ]},
            ]},
            fragment: Some(wgpu::FragmentState { module: &shader, entry_point: "fs_main", targets: &[
                Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL }),
                Some(wgpu::ColorTargetState { format, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL }),
                Some(wgpu::ColorTargetState { format: wgpu::TextureFormat::Rgba32Float, blend: Some(wgpu::BlendState::REPLACE), write_mask: wgpu::ColorWrites::ALL }),
            ]}),
            primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, ..Default::default() },
            depth_stencil: Some(wgpu::DepthStencilState { format: wgpu::TextureFormat::Depth32Float, depth_write_enabled: true, depth_compare: wgpu::CompareFunction::Less, stencil: Default::default() }),
            multisample: wgpu::MultisampleState::default(), multiview: None,
        });
        self.pipeline = Some(pipeline); Ok(())
    }
}

impl Default for DeferredGeometryPipeline { fn default() -> Self { Self::new() } }
