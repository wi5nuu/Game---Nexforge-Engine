use crate::mesh::{Uniforms, Vertex, MESH_SHADER};
use wgpu::util::DeviceExt;

pub struct SceneObject {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub position: [f32; 3],
    pub scale: f32,
    pub rotation: f32,
}

impl SceneObject {
    pub fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        position: [f32; 3],
        color: [f32; 3],
        scale: f32,
    ) -> Self {
        let vertices = create_box_vertices(color);
        let indices: Vec<u16> = vec![
            0, 1, 2, 0, 2, 3, 4, 6, 5, 4, 7, 6, 0, 4, 5, 0, 5, 1, 1, 5, 6, 1, 6, 2, 2, 6, 7, 2, 7, 3, 3, 7, 4, 3, 4, 0,
        ];
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Object Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Object Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Scene Object Uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Object Bind Group"),
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });
        Self {
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            uniform_buffer,
            bind_group,
            position,
            scale,
            rotation: 0.0,
        }
    }

    pub fn update_uniforms(&self, queue: &wgpu::Queue, vp: [[f32; 4]; 4]) {
        let sin_r = self.rotation.sin();
        let cos_r = self.rotation.cos();
        let model: [[f32; 4]; 4] = [
            [cos_r * self.scale, 0.0, sin_r * self.scale, 0.0],
            [0.0, self.scale, 0.0, 0.0],
            [-sin_r * self.scale, 0.0, cos_r * self.scale, 0.0],
            [self.position[0], self.position[1], self.position[2], 1.0],
        ];
        let uniforms = Uniforms {
            vp_matrix: vp,
            model_matrix: model,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[uniforms]));
    }

    pub fn render_render_pass<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

fn create_box_vertices(color: [f32; 3]) -> Vec<Vertex> {
    let (r, g, b) = (color[0], color[1], color[2]);
    vec![
        Vertex {
            position: [-0.5, -0.5, -0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [0.5, -0.5, -0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [0.5, 0.5, -0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, 0.5, -0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, -1.0],
        },
        Vertex {
            position: [-0.5, -0.5, 0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [0.5, 0.5, 0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, 1.0],
        },
        Vertex {
            position: [-0.5, 0.5, 0.5],
            color: [r, g, b],
            normal: [0.0, 0.0, 1.0],
        },
    ]
}

pub struct Scene {
    pub objects: Vec<SceneObject>,
    pub render_pipeline: wgpu::RenderPipeline,
}

impl Scene {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Scene Bind Group Layout"),
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
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Scene Shader"),
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(MESH_SHADER)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Scene Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Scene Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let objects = vec![
            SceneObject::new(device, &bind_group_layout, [0.0, 0.0, 0.0], [0.8, 0.2, 0.2], 1.0),
            SceneObject::new(device, &bind_group_layout, [3.0, 0.0, 0.0], [0.2, 0.8, 0.2], 0.8),
            SceneObject::new(device, &bind_group_layout, [-3.0, 0.0, 0.0], [0.2, 0.2, 0.8], 0.8),
            SceneObject::new(device, &bind_group_layout, [0.0, 0.0, 3.0], [0.8, 0.8, 0.2], 0.7),
            SceneObject::new(device, &bind_group_layout, [0.0, 0.0, -3.0], [0.8, 0.2, 0.8], 0.7),
            SceneObject::new(device, &bind_group_layout, [2.0, 0.5, 2.0], [0.2, 0.8, 0.8], 0.5),
            SceneObject::new(device, &bind_group_layout, [-2.0, 0.5, -2.0], [0.5, 0.5, 0.8], 0.5),
            SceneObject::new(device, &bind_group_layout, [0.0, -1.0, 0.0], [0.3, 0.3, 0.3], 20.0),
        ];

        Self {
            objects,
            render_pipeline,
        }
    }

    pub fn update(&mut self, dt: f64) {
        let count = self.objects.len();
        for (i, obj) in self.objects.iter_mut().enumerate() {
            if i < count - 1 {
                obj.rotation += (dt * 0.5 + i as f64 * 0.1) as f32;
            }
        }
    }

    pub fn render<'a>(&'a self, queue: &wgpu::Queue, pass: &mut wgpu::RenderPass<'a>, vp: [[f32; 4]; 4]) {
        pass.set_pipeline(&self.render_pipeline);
        for obj in &self.objects {
            obj.update_uniforms(queue, vp);
            obj.render_render_pass(pass);
        }
    }
}
