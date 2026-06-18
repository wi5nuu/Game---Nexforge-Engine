use glyphon::{
    self, FontSystem, SwashCache, Buffer, Metrics, Attrs, Shaping, TextAtlas,
    TextRenderer as GlyphonRenderer, TextArea, TextBounds, Resolution,
};
use glyphon::Color;

pub struct TextRenderer {
    font_system: FontSystem,
    cache: SwashCache,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
    buffers: Vec<(Buffer, f32, f32, f32, Color)>,
    screen_width: f32,
    screen_height: f32,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        width: f32,
        height: f32,
    ) -> Self {
        let font_system = FontSystem::new();
        let cache = SwashCache::new();
        let mut atlas = TextAtlas::new(device, queue, format);
        let renderer = GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        Self {
            font_system,
            cache,
            atlas,
            renderer,
            buffers: Vec::new(),
            screen_width: width,
            screen_height: height,
        }
    }

    pub fn add_text(&mut self, text: &str, x: f32, y: f32, scale: f32, color: [f32; 4]) {
        let mut buffer = Buffer::new(&mut self.font_system, Metrics::new(scale, scale * 1.2));
        buffer.set_size(&mut self.font_system, self.screen_width, self.screen_height);
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system);
        let c = Color::rgba(
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        );
        self.buffers.push((buffer, x, y, scale, c));
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
    }

    pub fn render<'a>(&'a mut self, device: &wgpu::Device, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>) {
        let resolution = Resolution {
            width: self.screen_width as u32,
            height: self.screen_height as u32,
        };
        let text_areas: Vec<TextArea> = self
            .buffers
            .iter()
            .map(|(buffer, left, top, _scale, default_color)| TextArea {
                buffer,
                left: *left,
                top: *top,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: self.screen_width as i32,
                    bottom: self.screen_height as i32,
                },
                default_color: *default_color,
            })
            .collect();
        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            resolution,
            text_areas,
            &mut self.cache,
        );
        let _ = self.renderer.render(&self.atlas, pass);
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.screen_width = width;
        self.screen_height = height;
    }
}
