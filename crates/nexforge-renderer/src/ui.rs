/// 2D UI quad renderer — draws alpha-blended coloured rectangles in screen space.
/// Intended for HUD panel backgrounds. Rendered after the 3-D scene pass and before
/// the text pass so that text always sits on top of panels.
const UI_SHADER: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color:    vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0)       color:         vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    // `position` is already in NDC (-1..1 x, -1..1 y, y-up)
    out.clip_position = vec4<f32>(in.position, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

// Maximum number of rectangles we can batch per frame.
const MAX_RECTS: usize = 128;
const VERTS_PER_RECT: usize = 6; // two triangles, no index buffer

// ── Vertex layout ─────────────────────────────────────────────────────────────

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UiVertex {
    position: [f32; 2],
    color: [f32; 4],
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// A screen-space rectangle with an RGBA colour.
#[derive(Clone)]
pub struct UiRect {
    /// Pixels from the left edge of the window.
    pub x: f32,
    /// Pixels from the top edge of the window.
    pub y: f32,
    /// Width in pixels.
    pub w: f32,
    /// Height in pixels.
    pub h: f32,
    /// Colour [r, g, b, a] in 0.0–1.0 range.
    pub color: [f32; 4],
}

impl UiRect {
    pub fn new(x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) -> Self {
        Self { x, y, w, h, color }
    }
}

// ── Renderer ───────────────────────────────────────────────────────────────────

pub struct UiRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    screen_width: f32,
    screen_height: f32,
    rects: Vec<UiRect>,
}

impl UiRenderer {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: f32,
        height: f32,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("UI Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(UI_SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("UI Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("UI Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<UiVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            // No depth test — UI always renders on top.
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let buffer_size =
            (MAX_RECTS * VERTS_PER_RECT * std::mem::size_of::<UiVertex>()) as u64;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("UI Vertex Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            screen_width: width,
            screen_height: height,
            rects: Vec::new(),
        }
    }

    // ── Mutation ---------------------------------------------------------------

    /// Queue a rectangle for rendering this frame.
    pub fn add_rect(&mut self, rect: UiRect) {
        if self.rects.len() < MAX_RECTS {
            self.rects.push(rect);
        }
    }

    /// Clear all queued rectangles (call at the start of every frame).
    pub fn clear(&mut self) {
        self.rects.clear();
    }

    /// Update the stored screen dimensions (call from `resize`).
    pub fn resize(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }

    // ── Rendering ---------------------------------------------------------------

    /// Upload vertices and draw all queued rects into `pass`.
    pub fn render<'a>(
        &'a mut self,
        queue: &wgpu::Queue,
        pass: &mut wgpu::RenderPass<'a>,
    ) {
        if self.rects.is_empty() {
            return;
        }

        let mut vertices: Vec<UiVertex> =
            Vec::with_capacity(self.rects.len() * VERTS_PER_RECT);

        for rect in &self.rects {
            // Convert pixel-space corners to NDC.
            let tl = self.to_ndc(rect.x,           rect.y);
            let tr = self.to_ndc(rect.x + rect.w,  rect.y);
            let bl = self.to_ndc(rect.x,           rect.y + rect.h);
            let br = self.to_ndc(rect.x + rect.w,  rect.y + rect.h);
            let c = rect.color;

            // Triangle 1: TL → BL → TR
            vertices.push(UiVertex { position: tl, color: c });
            vertices.push(UiVertex { position: bl, color: c });
            vertices.push(UiVertex { position: tr, color: c });
            // Triangle 2: TR → BL → BR
            vertices.push(UiVertex { position: tr, color: c });
            vertices.push(UiVertex { position: bl, color: c });
            vertices.push(UiVertex { position: br, color: c });
        }

        queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&vertices),
        );

        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..vertices.len() as u32, 0..1);
    }

    // ── Helpers ----------------------------------------------------------------

    /// Convert a pixel coordinate to wgpu NDC (y-up, range -1..1).
    #[inline]
    fn to_ndc(&self, px: f32, py: f32) -> [f32; 2] {
        let nx = (px / self.screen_width)  * 2.0 - 1.0;
        let ny = 1.0 - (py / self.screen_height) * 2.0;
        [nx, ny]
    }
}
