struct Uniforms {
    transform: mat4x4<f32>,
    resolution: vec2<f32>,
    opacity: f32,
    _padding: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var texture_sampler: sampler;

@group(0) @binding(2)
var source_texture: texture_2d<f32>;

@group(0) @binding(3)
var backdrop_texture: texture_2d<f32>;

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

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    let pos = uniforms.transform * vec4<f32>(input.position, 0.0, 1.0);
    output.position = pos;
    output.tex_coord = input.tex_coord;
    output.color = input.color * uniforms.opacity;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let src = textureSample(source_texture, texture_sampler, input.tex_coord);
    let dst = textureSample(backdrop_texture, texture_sampler, input.tex_coord);

    let blended = vec4<f32>(
        dst.r + src.r - 2.0 * dst.r * src.r,
        dst.g + src.g - 2.0 * dst.g * src.g,
        dst.b + src.b - 2.0 * dst.b * src.b,
        src.a
    );

    return blended * input.color;
}
