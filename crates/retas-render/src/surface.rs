use wgpu::{
    Surface, SurfaceConfiguration, SurfaceError, TextureFormat, PresentMode,
    CompositeAlphaMode, Device, Queue, Adapter,
};
use raw_window_handle::{HasWindowHandle, HasDisplayHandle};
use crate::RenderTexture;

#[derive(Debug)]
pub struct SurfaceSize {
    pub width: u32,
    pub height: u32,
}

pub struct RenderSurface<'window> {
    pub surface: Surface<'window>,
    pub config: SurfaceConfiguration,
}

impl<'window> RenderSurface<'window> {
    pub fn new(
        surface: Surface<'window>,
        device: &Device,
        adapter: &Adapter,
        width: u32,
        height: u32,
    ) -> Self {
        let caps = surface.get_capabilities(adapter);
        
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| matches!(f, TextureFormat::Bgra8UnormSrgb | TextureFormat::Rgba8UnormSrgb))
            .unwrap_or(caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: caps
                .present_modes
                .iter()
                .copied()
                .find(|m| *m == PresentMode::Mailbox)
                .unwrap_or(PresentMode::Fifo),
            alpha_mode: CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(device, &config);

        Self { surface, config }
    }

    pub fn resize(&mut self, device: &Device, width: u32, height: u32) {
        self.config.width = width.max(1);
        self.config.height = height.max(1);
        self.surface.configure(device, &self.config);
    }

    pub fn size(&self) -> SurfaceSize {
        SurfaceSize {
            width: self.config.width,
            height: self.config.height,
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.config.format
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn render(&self, device: &Device, queue: &Queue) -> RenderTexture {
        RenderTexture::new(
            device,
            self.config.width,
            self.config.height,
            self.config.format,
            Some("surface_texture"),
        )
    }
}

pub fn create_surface_for_window<'window, W: HasWindowHandle + HasDisplayHandle + Sync + Send>(
    instance: &wgpu::Instance,
    window: &'window W,
) -> Option<Surface<'window>> {
    instance.create_surface(window).ok()
}
