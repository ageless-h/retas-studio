use wgpu::{Device, ShaderModule, ShaderModuleDescriptor, ShaderSource};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ShaderError {
    #[error("Failed to compile shader: {0}")]
    CompilationFailed(String),
}

pub struct Shader {
    pub module: ShaderModule,
    pub name: String,
}

impl Shader {
    pub fn from_wgsl(device: &Device, source: &str, name: &str) -> Self {
        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some(name),
            source: ShaderSource::Wgsl(source.into()),
        });

        Self {
            module,
            name: name.to_string(),
        }
    }
}

pub const VERTEX_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

struct Uniforms {
    transform: mat3x3<f32>,
    resolution: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = uniforms.transform * vec3<f32>(input.position, 1.0);
    output.position = vec4<f32>(pos.xy, 0.0, 1.0);
    output.tex_coord = input.tex_coord;
    output.color = input.color;
    return output;
}
"#;

pub const FRAGMENT_SHADER_WGSL: &str = r#"
struct FragmentInput {
    @location(0) tex_coord: vec2<f32>,
    @location(1) color: vec4<f32>,
}

@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var texture_data: texture_2d<f32>;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture_data, texture_sampler, input.tex_coord);
    return tex_color * input.color;
}
"#;

pub const BRUSH_VERTEX_SHADER_WGSL: &str = r#"
struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) normal: vec2<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct BrushUniforms {
    transform: mat3x3<f32>,
    brush_size: f32,
    brush_hardness: f32,
    brush_opacity: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: BrushUniforms;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let offset = input.normal * uniforms.brush_size * 0.5;
    let pos = uniforms.transform * vec3<f32>(input.position + offset, 1.0);
    output.position = vec4<f32>(pos.xy, 0.0, 1.0);
    output.uv = input.uv;
    return output;
}
"#;

pub const BRUSH_FRAGMENT_SHADER_WGSL: &str = r#"
struct FragmentInput {
    @location(0) uv: vec2<f32>,
}

struct BrushUniforms {
    transform: mat3x3<f32>,
    brush_size: f32,
    brush_hardness: f32,
    brush_opacity: f32,
    _padding: f32,
}

@group(0) @binding(0) var<uniform> uniforms: BrushUniforms;
@group(0) @binding(1) var brush_color: vec4<f32>;

@fragment
fn fragment_main(input: FragmentInput) -> @location(0) vec4<f32> {
    let dist = length(input.uv - vec2<f32>(0.5, 0.5)) * 2.0;
    let alpha = smoothstep(1.0, 1.0 - uniforms.brush_hardness, dist);
    return vec4<f32>(brush_color.rgb, brush_color.a * alpha * uniforms.brush_opacity);
}
"#;

pub fn create_default_shaders(device: &Device) -> (Shader, Shader) {
    let vertex = Shader::from_wgsl(device, VERTEX_SHADER_WGSL, "default_vertex");
    let fragment = Shader::from_wgsl(device, FRAGMENT_SHADER_WGSL, "default_fragment");
    (vertex, fragment)
}

pub fn create_brush_shaders(device: &Device) -> (Shader, Shader) {
    let vertex = Shader::from_wgsl(device, BRUSH_VERTEX_SHADER_WGSL, "brush_vertex");
    let fragment = Shader::from_wgsl(device, BRUSH_FRAGMENT_SHADER_WGSL, "brush_fragment");
    (vertex, fragment)
}
