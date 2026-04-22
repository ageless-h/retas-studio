use wgpu::{Device, Queue, Texture, TextureDescriptor, TextureView, TextureFormat, Extent3d, ImageDataLayout};
use retas_core::Color8;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TextureError {
    #[error("Invalid texture dimensions")]
    InvalidDimensions,
    #[error("Failed to create texture: {0}")]
    CreationFailed(String),
}

pub struct RenderTexture {
    pub texture: Texture,
    pub view: TextureView,
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
}

impl RenderTexture {
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        format: TextureFormat,
        label: Option<&str>,
    ) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            width,
            height,
            format,
        }
    }

    pub fn from_rgba8(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> Self {
        let texture = Self::new(device, width, height, TextureFormat::Rgba8UnormSrgb, label);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    pub fn from_bgra8(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        data: &[u8],
        label: Option<&str>,
    ) -> Self {
        let texture = Self::new(device, width, height, TextureFormat::Bgra8UnormSrgb, label);

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        texture
    }

    pub fn solid_color(
        device: &Device,
        queue: &Queue,
        width: u32,
        height: u32,
        color: Color8,
        label: Option<&str>,
    ) -> Self {
        let data: Vec<u8> = (0..width * height)
            .flat_map(|_| [color.r, color.g, color.b, color.a])
            .collect();

        Self::from_rgba8(device, queue, width, height, &data, label)
    }

    pub fn view(&self) -> &TextureView {
        &self.view
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
}

pub struct DepthTexture {
    pub texture: Texture,
    pub view: TextureView,
}

impl DepthTexture {
    pub fn new(device: &Device, width: u32, height: u32, label: Option<&str>) -> Self {
        let texture = device.create_texture(&TextureDescriptor {
            label,
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view }
    }
}
