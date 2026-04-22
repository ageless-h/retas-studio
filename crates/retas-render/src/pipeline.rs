use wgpu::{
    Device, RenderPipeline, BindGroupLayout, VertexBufferLayout,
    VertexAttribute, VertexFormat, VertexStepMode, PrimitiveState, PrimitiveTopology,
    FrontFace, PolygonMode, MultisampleState, ColorTargetState, BlendState, BlendFactor,
    BlendOperation, ColorWrites,
};
use crate::Shader;

pub struct PipelineBuilder {
    vertex_shader: Option<Shader>,
    fragment_shader: Option<Shader>,
    bind_group_layouts: Vec<BindGroupLayout>,
    vertex_buffers: Vec<VertexBufferLayout<'static>>,
    primitive_state: PrimitiveState,
    multisample_state: MultisampleState,
    color_targets: Vec<Option<ColorTargetState>>,
    label: Option<String>,
}

impl PipelineBuilder {
    pub fn new() -> Self {
        Self {
            vertex_shader: None,
            fragment_shader: None,
            bind_group_layouts: Vec::new(),
            vertex_buffers: Vec::new(),
            primitive_state: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            multisample_state: MultisampleState::default(),
            color_targets: vec![Some(ColorTargetState {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                blend: Some(BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: BlendFactor::SrcAlpha,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: BlendFactor::One,
                        dst_factor: BlendFactor::OneMinusSrcAlpha,
                        operation: BlendOperation::Add,
                    },
                }),
                write_mask: ColorWrites::ALL,
            })],
            label: None,
        }
    }

    pub fn vertex_shader(mut self, shader: Shader) -> Self {
        self.vertex_shader = Some(shader);
        self
    }

    pub fn fragment_shader(mut self, shader: Shader) -> Self {
        self.fragment_shader = Some(shader);
        self
    }

    pub fn add_bind_group_layout(mut self, layout: BindGroupLayout) -> Self {
        self.bind_group_layouts.push(layout);
        self
    }

    pub fn add_vertex_buffer(mut self, buffer_layout: VertexBufferLayout<'static>) -> Self {
        self.vertex_buffers.push(buffer_layout);
        self
    }

    pub fn primitive_topology(mut self, topology: PrimitiveTopology) -> Self {
        self.primitive_state.topology = topology;
        self
    }

    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn color_format(mut self, format: wgpu::TextureFormat) -> Self {
        if let Some(target) = self.color_targets.first_mut() {
            if let Some(t) = target {
                *target = Some(ColorTargetState {
                    format,
                    blend: t.blend,
                    write_mask: t.write_mask,
                });
            }
        }
        self
    }

    pub fn build(self, device: &Device) -> Option<RenderPipeline> {
        let vertex_shader = self.vertex_shader?;
        let fragment_shader = self.fragment_shader?;

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: self.label.as_deref(),
            bind_group_layouts: &self.bind_group_layouts.iter().collect::<Vec<_>>(),
            push_constant_ranges: &[],
        });

        Some(device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: self.label.as_deref(),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader.module,
                entry_point: "vertex_main",
                buffers: &self.vertex_buffers,
            },
            primitive: self.primitive_state,
            depth_stencil: None,
            multisample: self.multisample_state,
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader.module,
                entry_point: "fragment_main",
                targets: &self.color_targets,
            }),
            multiview: None,
        }))
    }
}

impl Default for PipelineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlendComponent {
    pub src_factor: BlendFactor,
    pub dst_factor: BlendFactor,
    pub operation: BlendOperation,
}

impl Default for BlendComponent {
    fn default() -> Self {
        Self {
            src_factor: BlendFactor::SrcAlpha,
            dst_factor: BlendFactor::OneMinusSrcAlpha,
            operation: BlendOperation::Add,
        }
    }
}

impl BlendComponent {
    pub const REPLACE: BlendComponent = BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::Zero,
        operation: BlendOperation::Add,
    };

    pub const ALPHA: BlendComponent = BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };

    pub const PREMULTIPLIED: BlendComponent = BlendComponent {
        src_factor: BlendFactor::One,
        dst_factor: BlendFactor::OneMinusSrcAlpha,
        operation: BlendOperation::Add,
    };
}

pub fn create_simple_vertex_layout() -> VertexBufferLayout<'static> {
    VertexBufferLayout {
        array_stride: std::mem::size_of::<f32>() as wgpu::BufferAddress * 8,
        step_mode: VertexStepMode::Vertex,
        attributes: &[
            VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: VertexFormat::Float32x2,
            },
            VertexAttribute {
                offset: std::mem::size_of::<f32>() as wgpu::BufferAddress * 2,
                shader_location: 1,
                format: VertexFormat::Float32x2,
            },
            VertexAttribute {
                offset: std::mem::size_of::<f32>() as wgpu::BufferAddress * 4,
                shader_location: 2,
                format: VertexFormat::Float32x4,
            },
        ],
    }
}
