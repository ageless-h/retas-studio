use wgpu::{Device, BindGroupLayout, RenderPipeline};
use std::sync::Arc;
use std::collections::HashMap;
use retas_core::BlendMode;
use crate::Shader;

pub(crate) fn create_bind_group_layout(device: &Device) -> BindGroupLayout {
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

pub(crate) fn create_texture_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    let shader = Shader::from_wgsl(device, include_str!("../shaders/texture.wgsl"), "texture_shader");

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
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: vertex_size,
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
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}

pub(crate) fn create_blend_pipelines(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> HashMap<BlendMode, RenderPipeline> {
    let mut pipelines = HashMap::new();
    let shader = Shader::from_wgsl(device, include_str!("../shaders/texture.wgsl"), "texture_shader");

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Blend Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    let blend_modes = vec![
        (BlendMode::Normal, wgpu::BlendState::ALPHA_BLENDING),
        (BlendMode::Multiply, multiply_blend()),
        (BlendMode::Screen, screen_blend()),
        (BlendMode::Darken, darken_blend()),
        (BlendMode::Lighten, lighten_blend()),
    ];

    for (mode, blend_state) in blend_modes {
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(&format!("{:?} Pipeline", mode)),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader.module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: vertex_size,
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
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        });
        pipelines.insert(mode, pipeline);
    }

    pipelines
}

pub(crate) fn create_complex_blend_bind_group_layout(device: &Device) -> BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Complex Blend Bind Group Layout"),
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
            wgpu::BindGroupLayoutEntry {
                binding: 3,
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

pub(crate) fn create_complex_blend_pipelines(device: &Device, layout: &Arc<BindGroupLayout>, vertex_size: wgpu::BufferAddress) -> HashMap<BlendMode, RenderPipeline> {
    let mut pipelines = HashMap::new();
    
    pipelines.insert(BlendMode::Overlay, create_overlay_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::HardLight, create_hard_light_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::SoftLight, create_soft_light_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::Difference, create_difference_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::Exclusion, create_exclusion_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::ColorDodge, create_color_dodge_pipeline(device, layout, vertex_size));
    pipelines.insert(BlendMode::ColorBurn, create_color_burn_pipeline(device, layout, vertex_size));
    
    pipelines
}

fn create_overlay_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/overlay.wgsl"), "overlay", "Overlay", vertex_size)
}

fn create_hard_light_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/hard_light.wgsl"), "hard_light", "HardLight", vertex_size)
}

fn create_soft_light_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/soft_light.wgsl"), "soft_light", "SoftLight", vertex_size)
}

fn create_difference_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/difference.wgsl"), "difference", "Difference", vertex_size)
}

fn create_exclusion_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/exclusion.wgsl"), "exclusion", "Exclusion", vertex_size)
}

fn create_color_dodge_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/color_dodge.wgsl"), "color_dodge", "ColorDodge", vertex_size)
}

fn create_color_burn_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    create_shader_pipeline(device, bind_group_layout, include_str!("../shaders/color_burn.wgsl"), "color_burn", "ColorBurn", vertex_size)
}

fn create_shader_pipeline(device: &Device, bind_group_layout: &BindGroupLayout, shader_source: &str, shader_name: &str, pipeline_name: &str, vertex_size: wgpu::BufferAddress) -> RenderPipeline {
    let shader = Shader::from_wgsl(device, shader_source, shader_name);

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some(&format!("{} Pipeline Layout", pipeline_name)),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(&format!("{} Pipeline", pipeline_name)),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader.module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: vertex_size,
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
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
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
