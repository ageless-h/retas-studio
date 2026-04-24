use wgpu::{CommandEncoder, LoadOp, StoreOp, Operations, RenderPassColorAttachment};
use wgpu::util::DeviceExt;
use retas_core::{Layer, Matrix2D, Color8};
use super::{Renderer, RenderTexture, Uniforms, CachedTexture};

impl Renderer {
    pub(crate) fn render_layer(&mut self, layer: &Layer, target_texture: &RenderTexture, encoder: &mut CommandEncoder) {
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
        &mut self,
        layer: &retas_core::RasterLayer,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
    ) {
        let frame = match layer.frames.get(&layer.current_frame) {
            Some(f) if f.width > 0 && f.height > 0 => f,
            _ => return,
        };

        // Texture cache: skip GPU re-upload if Arc<Vec<u8>> pointer unchanged
        let cache_key = (layer.base.id, layer.current_frame);
        let current_ptr = std::sync::Arc::as_ptr(&frame.image_data) as usize;
        
        let needs_upload = match self.texture_cache.get(&cache_key) {
            Some(cached) => cached.data_ptr != current_ptr,
            None => true,
        };
        
        if needs_upload {
            let new_texture = RenderTexture::from_rgba8(
                &self.device.device,
                &self.device.queue,
                frame.width,
                frame.height,
                &frame.image_data,
                Some(&layer.base.name),
            );
            self.texture_cache.insert(cache_key, CachedTexture {
                texture: new_texture,
                data_ptr: current_ptr,
            });
        }
        
        let layer_texture = &self.texture_cache[&cache_key].texture;

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

        let is_complex_blend = matches!(layer.base.blend_mode,
            retas_core::BlendMode::Overlay | retas_core::BlendMode::HardLight | retas_core::BlendMode::SoftLight |
            retas_core::BlendMode::Difference | retas_core::BlendMode::Exclusion |
            retas_core::BlendMode::ColorDodge | retas_core::BlendMode::ColorBurn);

        if is_complex_blend {
            let backdrop_texture = RenderTexture::new(
                &self.device.device,
                target_texture.width,
                target_texture.height,
                target_texture.format,
                Some("Complex Blend Backdrop"),
            );

            encoder.copy_texture_to_texture(
                wgpu::ImageCopyTexture {
                    texture: &target_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyTexture {
                    texture: &backdrop_texture.texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: target_texture.width,
                    height: target_texture.height,
                    depth_or_array_layers: 1,
                },
            );

            if let Some(pipeline) = self.complex_blend_pipelines.get(&layer.base.blend_mode) {
                let blend_bind_group = self.device.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Complex Blend Bind Group"),
                    layout: &self.complex_blend_layout,
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
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: wgpu::BindingResource::TextureView(&backdrop_texture.view),
                        },
                    ],
                });

                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Complex Blend Raster Layer Render Pass"),
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
                render_pass.set_bind_group(0, &blend_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
                render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..6, 0, 0..1);
            }
        } else {
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

        let vertices: Vec<super::Vertex> = mesh.vertices.iter().map(|v| super::Vertex {
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

    pub fn render_onion_skin(
        &self,
        document: &retas_core::Document,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
        prev_frames: u32,
        next_frames: u32,
        base_opacity: f32,
    ) {
        let current_frame = document.timeline.current_frame;
        let tint_prev = Color8::new(255, 100, 100, 255);
        let tint_next = Color8::new(100, 100, 255, 255);

        for offset in 1..=prev_frames {
            if current_frame < offset {
                break;
            }
            let frame_num = current_frame - offset;
            let opacity = base_opacity / offset as f32;
            self.render_document_frame(document, target_texture, encoder, frame_num, opacity, tint_prev);
        }

        for offset in 1..=next_frames {
            let frame_num = current_frame + offset;
            if frame_num > document.timeline.end_frame {
                break;
            }
            let opacity = base_opacity / offset as f32;
            self.render_document_frame(document, target_texture, encoder, frame_num, opacity, tint_next);
        }
    }

    fn render_document_frame(
        &self,
        document: &retas_core::Document,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
        frame_number: u32,
        opacity: f32,
        tint: Color8,
    ) {
        for layer_id in document.timeline.layer_order.iter().rev() {
            if let Some(layer) = document.layers.get(layer_id) {
                if !layer.base().visible {
                    continue;
                }
                match layer {
                    Layer::Raster(raster_layer) => {
                        self.render_raster_layer_onion(raster_layer, target_texture, encoder, frame_number, opacity, tint);
                    }
                    _ => {}
                }
            }
        }
    }

    fn render_raster_layer_onion(
        &self,
        layer: &retas_core::RasterLayer,
        target_texture: &RenderTexture,
        encoder: &mut CommandEncoder,
        frame_number: u32,
        opacity: f32,
        tint: Color8,
    ) {
        let frame = match layer.frames.get(&frame_number) {
            Some(f) if f.width > 0 && f.height > 0 => f,
            _ => return,
        };

        let mut image_data = (*frame.image_data).clone();
        for pixel in image_data.chunks_exact_mut(4) {
            pixel[0] = (pixel[0] as f32 * tint.r as f32 / 255.0) as u8;
            pixel[1] = (pixel[1] as f32 * tint.g as f32 / 255.0) as u8;
            pixel[2] = (pixel[2] as f32 * tint.b as f32 / 255.0) as u8;
        }

        let layer_texture = RenderTexture::from_rgba8(
            &self.device.device,
            &self.device.queue,
            frame.width,
            frame.height,
            &image_data,
            Some(&format!("{}_onion", layer.base.name)),
        );

        let sampler = self.device.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Onion Skin Sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let transform = Matrix2D::identity();
        let uniforms = Uniforms::new(&transform, target_texture.size(), opacity);

        let uniform_buffer = self.device.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Onion Skin Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = self.device.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Onion Skin Bind Group"),
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

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Onion Skin Render Pass"),
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
        render_pass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..6, 0, 0..1);
    }
}
