use wgpu::{
    Device, Queue, CommandEncoder, LoadOp, StoreOp, Operations, RenderPassColorAttachment,
    Color, BindGroup, BindGroupLayout, Buffer, RenderPipeline,
};
use wgpu::util::DeviceExt;
use retas_core::{Document, LayerId, Color8, BlendMode, Matrix2D, Point, Layer};
use crate::{RenderDevice, RenderTexture, RenderSurface, Shader};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("No surface available")]
    NoSurface,
    #[error("Failed to get current texture: {0}")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("Invalid layer: {0:?}")]
    InvalidLayer(LayerId),
    #[error("Device creation failed: {0}")]
    DeviceError(String),
    #[error("Failed to create bind group: {0}")]
    BindGroupError(String),
    #[error("Failed to create buffer: {0}")]
    BufferError(String),
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4],
    resolution: [f32; 2],
    opacity: f32,
    _padding: f32,
}

impl Uniforms {
    fn new(transform: &Matrix2D, resolution: (u32, u32), opacity: f32) -> Self {
        let matrix = transform.to_column_major_3x3();
        Self {
            transform: [
                [matrix[0] as f32, matrix[1] as f32, 0.0, matrix[2] as f32],
                [matrix[3] as f32, matrix[4] as f32, 0.0, matrix[5] as f32],
                [0.0, 0.0, 1.0, 0.0],
                [matrix[6] as f32, matrix[7] as f32, 0.0, matrix[8] as f32],
            ],
            resolution: [resolution.0 as f32, resolution.1 as f32],
            opacity,
            _padding: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    tex_coord: [f32; 2],
    color: [f32; 4],
}

pub struct Renderer {
    device: RenderDevice,
    clear_color: Color,
    bind_group_layout: BindGroupLayout,
    texture_pipeline: RenderPipeline,
    blend_pipelines: std::collections::HashMap<BlendMode, RenderPipeline>,
    quad_vertex_buffer: Buffer,
    quad_index_buffer: Buffer,
}

impl Renderer {
    pub async fn new() -> Result<Self, RenderError> {
        let device = RenderDevice::new()
            .await
            .map_err(|e| RenderError::DeviceError(e.to_string()))?;

        Self::create_with_device(device)
    }

    pub async fn with_surface(surface: &wgpu::Surface<'_>) -> Result<Self, RenderError> {
        let device = RenderDevice::with_surface(surface)
            .await
            .map_err(|e| RenderError::DeviceError(e.to_string()))?;

        Self::create_with_device(device)
    }

    fn create_with_device(device: RenderDevice) -> Result<Self, RenderError> {
        let bind_group_layout = Self::create_bind_group_layout(&device.device);
        let texture_pipeline = Self::create_texture_pipeline(&device.device, &bind_group_layout);
        let blend_pipelines = Self::create_blend_pipelines(&device.device, &bind_group_layout);

        let (quad_vertex_buffer, quad_index_buffer) = Self::create_quad_buffers(&device.device);

        Ok(Self {
            device,
            clear_color: Color::WHITE,
            bind_group_layout,
            texture_pipeline,
            blend_pipelines,
            quad_vertex_buffer,
            quad_index_buffer,
        })
    }

    fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Renderer Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        })
    }

    fn create_texture_pipeline(device: &Device, bind_group_layout: &BindGroupLayout) -> RenderPipeline {
        let shader = Shader::from_wgsl(device, include_str!("shaders/texture.wgsl"), "texture_shader");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Texture Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Texture Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x4,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader.module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
        })
    }

    fn create_blend_pipelines(device: &Device, bind_group_layout: &BindGroupLayout) -> std::collections::HashMap<BlendMode, RenderPipeline> {
        let mut pipelines = std::collections::HashMap::new();
        let shader = Shader::from_wgsl(device, include_str!("shaders/texture.wgsl"), "texture_shader");

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Blend Pipeline Layout"),
            bind_group_layouts: &[bind_group_layout],
            push_constant_ranges: &[],
        });

        let blend_modes = vec![
            (BlendMode::Normal, wgpu::BlendState::ALPHA_BLENDING),
            (BlendMode::Multiply, Self::multiply_blend()),
            (BlendMode::Screen, Self::screen_blend()),
            (BlendMode::Overlay, Self::screen_blend()),
            (BlendMode::Darken, Self::darken_blend()),
            (BlendMode::Lighten, Self::lighten_blend()),
        ];

        for (mode, blend_state) in blend_modes {
            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(&format!("{:?} Pipeline", mode)),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader.module,
                    entry_point: "vs_main",
                    buffers: &[wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                            wgpu::VertexAttribute {
                                offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    }],
                },
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader.module,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: Some(blend_state),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });
            pipelines.insert(mode, pipeline);
        }

        pipelines
    }

    fn multiply_blend() -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::Dst,
                dst_factor: wgpu::BlendFactor::Zero,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        }
    }

    fn screen_blend() -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrc,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        }
    }

    fn darken_blend() -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Min,
            },
            alpha: wgpu::BlendComponent::OVER,
        }
    }

    fn lighten_blend() -> wgpu::BlendState {
        wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::One,
                operation: wgpu::BlendOperation::Max,
            },
            alpha: wgpu::BlendComponent::OVER,
        }
    }

    fn create_quad_buffers(device: &Device) -> (Buffer, Buffer) {
        let vertices = [
            Vertex { position: [-1.0, -1.0], tex_coord: [0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [1.0, -1.0], tex_coord: [1.0, 1.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [1.0, 1.0], tex_coord: [1.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
            Vertex { position: [-1.0, 1.0], tex_coord: [0.0, 0.0], color: [1.0, 1.0, 1.0, 1.0] },
        ];

        let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Quad Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        (vertex_buffer, index_buffer)
    }

    pub fn device(&self) -> &RenderDevice {
        &self.device
    }

    pub fn set_clear_color(&mut self, color: Color8) {
        self.clear_color = Color {
            r: color.r as f64 / 255.0,
            g: color.g as f64 / 255.0,
            b: color.b as f64 / 255.0,
            a: color.a as f64 / 255.0,
        };
    }

    pub fn render_document(&self, document: &Document, target_texture: &RenderTexture) -> CommandEncoder {
        let mut encoder = self.device.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Document Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Clear Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &target_texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(self.clear_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        for layer_id in document.timeline.layer_order.iter().rev() {
            if let Some(layer) = document.layers.get(layer_id) {
                if !layer.base().visible {
                    continue;
                }
                self.render_layer(layer, target_texture, &mut encoder);
            }
        }

        encoder
    }

    fn render_layer(&self, layer: &Layer, target_texture: &RenderTexture, encoder: &mut CommandEncoder) {
        match layer {
            Layer::Raster(raster_layer) => {
                self.render_raster_layer(raster_layer, target_texture, encoder);
            }
            Layer::Vector(vector_layer) => {
                self.render_vector_layer(vector_layer, target_texture, encoder);
            }
            Layer::Camera(camera_layer) => {
                self.render_camera_layer(camera_layer, target_texture, encoder);
            }
            Layer::Text(text_layer) => {
                self.render_text_layer(text_layer, target_texture, encoder);
            }
            Layer::Sound(_) => {}
        }
    }

    fn render_raster_layer(
        &self,
        layer: &retas_core::RasterLayer,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
    ) {
        let frame = match layer.frames.get(&layer.current_frame) {
            Some(f) if f.width > 0 && f.height > 0 => f,
            _ => return,
        };

        let layer_texture = RenderTexture::from_rgba8(
            &self.device.device,
            &self.device.queue,
            frame.width,
            frame.height,
            &frame.image_data,
            Some(&layer.base.name),
        );

        let sampler = self.device.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Layer Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let transform = Matrix2D::identity();
        let uniforms = Uniforms::new(&transform, target_texture.size(), layer.base.opacity as f32);

        let uniform_buffer = self.device.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Layer Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&layer_texture.view),
                },
            ],
        });

        let pipeline = self.blend_pipelines
            .get(&layer.base.blend_mode)
            .unwrap_or(&self.texture_pipeline);

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Raster Layer Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target_texture.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }

    fn render_vector_layer(
        &self,
        layer: &retas_core::VectorLayer,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
    ) {
        let frame = match layer.frames.get(&layer.current_frame) {
            Some(f) => f,
            _ => return,
        };

        for stroke in &frame.strokes {
            self.render_stroke_direct(stroke, target_texture, encoder, layer.base.opacity);
        }
    }

    fn render_stroke_direct(
        &self,
        stroke: &retas_core::Stroke,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
        layer_opacity: f64,
    ) {
        use retas_vector::{Stroke as VectorStroke, StrokeStyle, StrokeCap, StrokeJoin, tessellate_stroke, PressurePoint};
        
        if stroke.points.len() < 2 {
            return;
        }

        let style = StrokeStyle {
            color: stroke.color,
            width: stroke.brush_size,
            opacity: stroke.opacity * layer_opacity,
            cap: StrokeCap::Round,
            join: StrokeJoin::Round,
            miter_limit: 4.0,
            dash_pattern: Vec::new(),
            dash_offset: 0.0,
        };

        let mut vector_stroke = VectorStroke::new(style);
        for point in &stroke.points {
            let p = PressurePoint::new(point.position, point.pressure);
            vector_stroke.add_point(p);
        }

        let mesh = tessellate_stroke(&vector_stroke);
        if mesh.vertices.is_empty() || mesh.indices.is_empty() {
            return;
        }

        let vertices: Vec<Vertex> = mesh.vertices.iter().map(|v| Vertex {
            position: [v.position.x as f32, v.position.y as f32],
            tex_coord: [v.uv.0, v.uv.1],
            color: [
                v.color.r as f32 / 255.0,
                v.color.g as f32 / 255.0,
                v.color.b as f32 / 255.0,
                v.color.a as f32 / 255.0,
            ],
        }).collect();

        let vertex_buffer = self.device.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stroke Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = self.device.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stroke Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let transform = Matrix2D::identity();
        let uniforms = Uniforms::new(&transform, target_texture.size(), 1.0);

        let uniform_buffer = self.device.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Stroke Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let sampler = self.device.device.create_sampler(&wgpu::SamplerDescriptor::default());
        let dummy_texture = RenderTexture::solid_color(
            &self.device.device,
            &self.device.queue,
            1, 1,
            Color8::new(255, 255, 255, 255),
            Some("Dummy"),
        );

        let bind_group = self.device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Stroke Bind Group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_texture.view),
                },
            ],
        });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Stroke Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &target_texture.view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Load,
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.texture_pipeline);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
    }

    fn render_camera_layer(
        &self,
        layer: &retas_core::CameraLayer,
        _target_texture: &RenderTexture,
        _encoder: &mut CommandEncoder,
    ) {
        if !layer.base.visible {
            return;
        }

        let mut transform = Matrix2D::identity();
        transform.translate(-layer.position.x, -layer.position.y);
        transform.rotate(layer.rotation);
        transform.scale(layer.zoom, layer.zoom);
        let _transform = transform;
    }

    fn render_text_layer(
        &self,
        layer: &retas_core::TextLayer,
        _target_texture: &RenderTexture,
        _encoder: &mut CommandEncoder,
    ) {
        if layer.text.is_empty() {
            return;
        }
    }

    pub fn submit(&self, encoder: CommandEncoder) {
        self.device.queue.submit(std::iter::once(encoder.finish()));
    }

    pub fn create_texture(&self, width: u32, height: u32) -> RenderTexture {
        RenderTexture::new(
            &self.device.device,
            width,
            height,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            Some("render_texture"),
        )
    }

    pub fn clear_texture(&self, texture: &RenderTexture, color: Option<Color8>) {
        let clear_color = color
            .map(|c| Color {
                r: c.r as f64 / 255.0,
                g: c.g as f64 / 255.0,
                b: c.b as f64 / 255.0,
                a: c.a as f64 / 255.0,
            })
            .unwrap_or(self.clear_color);

        let mut encoder = self.device.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("clear_encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &texture.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(clear_color),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.device.queue.submit(std::iter::once(encoder.finish()));
    }
}
