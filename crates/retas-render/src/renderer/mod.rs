use wgpu::{
    Device, CommandEncoder, LoadOp, StoreOp, Operations, RenderPassColorAttachment,
    Color, BindGroupLayout, Buffer, RenderPipeline,
};
use wgpu::util::DeviceExt;
use retas_core::{Document, LayerId, Color8, BlendMode, Matrix2D};
use crate::{RenderDevice, RenderTexture};
use thiserror::Error;
use std::sync::Arc;
use std::collections::HashMap;

mod blend;
mod layer_render;

/// Cached GPU texture entry. Keyed by (LayerId, frame_number).
/// Uses Arc pointer identity to detect when pixel data changes.
pub(crate) struct CachedTexture {
    pub texture: RenderTexture,
    /// The raw pointer of the Arc<Vec<u8>> when this texture was uploaded.
    /// If the Arc pointer changes, the cache is stale.
    pub data_ptr: usize,
}

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
pub(crate) struct Uniforms {
    transform: [[f32; 4]; 4],
    resolution: [f32; 2],
    opacity: f32,
    _padding: f32,
}

impl Uniforms {
    pub(crate) fn new(transform: &Matrix2D, resolution: (u32, u32), opacity: f32) -> Self {
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
pub(crate) struct Vertex {
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
    complex_blend_layout: Arc<BindGroupLayout>,
    complex_blend_pipelines: std::collections::HashMap<BlendMode, RenderPipeline>,
    quad_vertex_buffer: Buffer,
    quad_index_buffer: Buffer,
    /// GPU texture cache: avoids re-uploading unchanged raster frame data.
    texture_cache: HashMap<(LayerId, u32), CachedTexture>,
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
        let vertex_size = std::mem::size_of::<Vertex>() as wgpu::BufferAddress;
        let bind_group_layout = blend::create_bind_group_layout(&device.device);
        let texture_pipeline = blend::create_texture_pipeline(&device.device, &bind_group_layout, vertex_size);
        let blend_pipelines = blend::create_blend_pipelines(&device.device, &bind_group_layout, vertex_size);
        let complex_blend_layout = Arc::new(blend::create_complex_blend_bind_group_layout(&device.device));
        let complex_blend_pipelines = blend::create_complex_blend_pipelines(&device.device, &complex_blend_layout, vertex_size);

        let (quad_vertex_buffer, quad_index_buffer) = Self::create_quad_buffers(&device.device);

        Ok(Self {
            device,
            clear_color: Color::WHITE,
            bind_group_layout,
            texture_pipeline,
            blend_pipelines,
            complex_blend_layout,
            complex_blend_pipelines,
            quad_vertex_buffer,
            quad_index_buffer,
            texture_cache: HashMap::new(),
        })
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

    pub fn render_document(&mut self, document: &Document, target_texture: &RenderTexture) -> CommandEncoder {
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
