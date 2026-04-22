pub mod device;
pub mod renderer;
pub mod texture;
pub mod shader;
pub mod pipeline;
pub mod surface;
pub mod rasterizer;

pub use device::{DeviceError, RenderDevice};
pub use renderer::{RenderError, Renderer};
pub use texture::{DepthTexture, RenderTexture, TextureError};
pub use shader::{
    BRUSH_FRAGMENT_SHADER_WGSL, BRUSH_VERTEX_SHADER_WGSL, FRAGMENT_SHADER_WGSL, Shader,
    ShaderError, VERTEX_SHADER_WGSL, create_brush_shaders, create_default_shaders,
};
pub use pipeline::{BlendComponent, PipelineBuilder, create_simple_vertex_layout};
pub use surface::{RenderSurface, SurfaceSize, create_surface_for_window};
pub use rasterizer::StrokeRasterizer;
