// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Rounded flat pipeline

use std::mem::size_of;

use crate::draw::{Rgb, Vec2};
use crate::shared::SharedState;
use kas::draw::Colour;
use kas::geom::{Coord, Rect, Size};

/// Offset relative to the size of a pixel used by the fragment shader to
/// implement multi-sampling.
const OFFSET: f32 = 0.125;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Vertex(Vec2, Rgb, f32, Vec2, Vec2);

/// A pipeline for rendering rounded shapes
pub struct FlatRound {
    bind_group: wgpu::BindGroup,
    scale_buf: wgpu::Buffer,
    render_pipeline: wgpu::RenderPipeline,
    passes: Vec<Vec<Vertex>>,
}

impl FlatRound {
    /// Construct
    pub fn new<C, T>(shared: &SharedState<C, T>, size: Size) -> Self {
        let device = &shared.device;

        type Scale = [f32; 2];
        let scale_factor: Scale = [2.0 / size.0 as f32, 2.0 / size.1 as f32];
        let scale_buf = device
            .create_buffer_mapped(
                scale_factor.len(),
                wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            )
            .fill_from_slice(&scale_factor);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            bindings: &[wgpu::BindGroupLayoutBinding {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer { dynamic: false },
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[wgpu::Binding {
                binding: 0,
                resource: wgpu::BindingResource::Buffer {
                    buffer: &scale_buf,
                    range: 0..(size_of::<Scale>() as u64),
                },
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &shared.shaders.vert_3122,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &shared.shaders.frag_flat_round,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha_blend: wgpu::BlendDescriptor {
                    src_factor: wgpu::BlendFactor::Zero,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float3,
                        offset: size_of::<Vec2>() as u64,
                        shader_location: 1,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float,
                        offset: (size_of::<Vec2>() + size_of::<Rgb>()) as u64,
                        shader_location: 2,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: (size_of::<Vec2>() + size_of::<Rgb>() + size_of::<f32>()) as u64,
                        shader_location: 3,
                    },
                    wgpu::VertexAttributeDescriptor {
                        format: wgpu::VertexFormat::Float2,
                        offset: (2 * size_of::<Vec2>() + size_of::<Rgb>() + size_of::<f32>())
                            as u64,
                        shader_location: 4,
                    },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        FlatRound {
            bind_group,
            scale_buf,
            render_pipeline,
            passes: vec![],
        }
    }

    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        size: Size,
    ) {
        type Scale = [f32; 2];
        let scale_factor: Scale = [2.0 / size.0 as f32, 2.0 / size.1 as f32];
        let scale_buf = device
            .create_buffer_mapped(scale_factor.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(&scale_factor);
        let byte_len = size_of::<Scale>() as u64;

        encoder.copy_buffer_to_buffer(&scale_buf, 0, &self.scale_buf, 0, byte_len);
    }

    /// Render queued triangles and clear the queue
    pub fn render(&mut self, device: &wgpu::Device, pass: usize, rpass: &mut wgpu::RenderPass) {
        if pass >= self.passes.len() {
            return;
        }
        let v = &mut self.passes[pass];
        let buffer = device
            .create_buffer_mapped(v.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&v);
        let count = v.len() as u32;

        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.bind_group, &[]);
        rpass.set_vertex_buffers(0, &[(&buffer, 0)]);
        rpass.draw(0..count, 0..1);

        v.clear();
    }

    pub fn line(&mut self, pass: usize, p1: Coord, p2: Coord, radius: f32, col: Colour) {
        if p1 == p2 {
            let rect = Rect {
                pos: p1 - Coord::uniform(radius as i32),
                size: Size::uniform((radius * 2.0) as u32),
            };
            self.circle(pass, rect, radius, col);
            return;
        }

        let col = col.into();

        let p1 = Vec2::from(p1);
        let p2 = Vec2::from(p2);
        let vx = p2 - p1;
        let vx = vx * radius / (vx.0 * vx.0 + vx.1 * vx.1).sqrt();
        let vy = Vec2(-vx.1, vx.0);

        let n0 = Vec2::splat(0.0);
        let nb = (vx + vy).sign();
        let na = -nb;

        // Since we take the mid-point, all offsets are uniform
        let p = Vec2::splat(OFFSET / radius);

        let ma1 = Vertex(p1 - vy, col, 0.0, Vec2(0.0, na.1), p);
        let mb1 = Vertex(p1 + vy, col, 0.0, Vec2(0.0, nb.1), p);
        let aa1 = Vertex(ma1.0 - vx, col, 0.0, Vec2(na.0, na.1), p);
        let ab1 = Vertex(mb1.0 - vx, col, 0.0, Vec2(na.0, nb.1), p);
        let ma2 = Vertex(p2 - vy, col, 0.0, Vec2(0.0, na.1), p);
        let mb2 = Vertex(p2 + vy, col, 0.0, Vec2(0.0, nb.1), p);
        let ba2 = Vertex(ma2.0 + vx, col, 0.0, Vec2(nb.0, na.1), p);
        let bb2 = Vertex(mb2.0 + vx, col, 0.0, Vec2(nb.0, nb.1), p);
        let p1 = Vertex(p1, col, 0.0, n0, p);
        let p2 = Vertex(p2, col, 0.0, n0, p);

        #[rustfmt::skip]
        self.add_vertices(pass, &[
            ab1, p1, mb1,
            aa1, p1, ab1,
            ma1, p1, aa1,
            mb1, p1, mb2,
            mb2, p1, p2,
            mb2, p2, bb2,
            bb2, p2, ba2,
            ba2, p2, ma2,
            ma2, p2, p1,
            p1, ma1, ma2,
        ]);
    }

    /// Bounds on input: `0 ≤ inner_radius ≤ 1`.
    pub fn circle(&mut self, pass: usize, rect: Rect, inner_radius: f32, col: Colour) {
        let aa = Vec2::from(rect.pos);
        let bb = aa + Vec2::from(rect.size);

        if !aa.lt(bb) {
            // zero / negative size: nothing to draw
            return;
        }

        let inner = inner_radius.max(0.0).min(1.0);

        let col = col.into();

        let ab = Vec2(aa.0, bb.1);
        let ba = Vec2(bb.0, aa.1);
        let mid = (aa + bb) * 0.5;

        let n0 = Vec2::splat(0.0);
        let nb = (bb - aa).sign();
        let na = -nb;
        let nab = Vec2(na.0, nb.1);
        let nba = Vec2(nb.0, na.1);

        // Since we take the mid-point, all offsets are uniform
        let p = nb / (bb - mid) * OFFSET;

        let aa = Vertex(aa, col, inner, na, p);
        let ab = Vertex(ab, col, inner, nab, p);
        let ba = Vertex(ba, col, inner, nba, p);
        let bb = Vertex(bb, col, inner, nb, p);
        let mid = Vertex(mid, col, inner, n0, p);

        #[rustfmt::skip]
        self.add_vertices(pass, &[
            ba, mid, aa,
            bb, mid, ba,
            ab, mid, bb,
            aa, mid, ab,
        ]);
    }

    /// Bounds on input: `aa < cc < dd < bb`, `0 ≤ inner_radius ≤ 1`.
    pub fn rounded_frame(
        &mut self,
        pass: usize,
        outer: Rect,
        inner: Rect,
        inner_radius: f32,
        col: Colour,
    ) {
        let aa = Vec2::from(outer.pos);
        let bb = aa + Vec2::from(outer.size);
        let mut cc = Vec2::from(inner.pos);
        let mut dd = cc + Vec2::from(inner.size);

        if !aa.lt(bb) {
            // zero / negative size: nothing to draw
            return;
        }
        if !aa.le(cc) || !cc.le(bb) {
            cc = aa;
        }
        if !aa.le(dd) || !dd.le(bb) {
            dd = bb;
        }
        if !cc.le(dd) {
            dd = cc;
        }

        let inner = inner_radius.max(0.0).min(1.0);

        let col = col.into();

        let ab = Vec2(aa.0, bb.1);
        let ba = Vec2(bb.0, aa.1);
        let cd = Vec2(cc.0, dd.1);
        let dc = Vec2(dd.0, cc.1);

        let n0 = Vec2::splat(0.0);
        let nb = (bb - aa).sign();
        let na = -nb;
        let nab = Vec2(na.0, nb.1);
        let nba = Vec2(nb.0, na.1);
        let na0 = Vec2(na.0, 0.0);
        let nb0 = Vec2(nb.0, 0.0);
        let n0a = Vec2(0.0, na.1);
        let n0b = Vec2(0.0, nb.1);

        let paa = na / (aa - cc) * OFFSET;
        let pab = nab / (ab - cd) * OFFSET;
        let pba = nba / (ba - dc) * OFFSET;
        let pbb = nb / (bb - dd) * OFFSET;

        // We must add corners separately to ensure correct interpolation of dir
        // values, hence need 16 points:
        let ab = Vertex(ab, col, inner, nab, pab);
        let ba = Vertex(ba, col, inner, nba, pba);
        let cd = Vertex(cd, col, inner, n0, pab);
        let dc = Vertex(dc, col, inner, n0, pba);

        let ac = Vertex(Vec2(aa.0, cc.1), col, inner, na0, paa);
        let ad = Vertex(Vec2(aa.0, dd.1), col, inner, na0, pab);
        let bc = Vertex(Vec2(bb.0, cc.1), col, inner, nb0, pba);
        let bd = Vertex(Vec2(bb.0, dd.1), col, inner, nb0, pbb);

        let ca = Vertex(Vec2(cc.0, aa.1), col, inner, n0a, paa);
        let cb = Vertex(Vec2(cc.0, bb.1), col, inner, n0b, pab);
        let da = Vertex(Vec2(dd.0, aa.1), col, inner, n0a, pba);
        let db = Vertex(Vec2(dd.0, bb.1), col, inner, n0b, pbb);

        let aa = Vertex(aa, col, inner, na, paa);
        let bb = Vertex(bb, col, inner, nb, pbb);
        let cc = Vertex(cc, col, inner, n0, paa);
        let dd = Vertex(dd, col, inner, n0, pbb);

        // TODO: the four sides are simple rectangles, hence could use simpler rendering

        #[rustfmt::skip]
        self.add_vertices(pass, &[
            // top bar: ba - dc - cc - aa
            ba, dc, da,
            da, dc, ca,
            dc, cc, ca,
            ca, cc, aa,
            // left bar: aa - cc - cd - ab
            aa, cc, ac,
            ac, cc, cd,
            ac, cd, ad,
            ad, cd, ab,
            // bottom bar: ab - cd - dd - bb
            ab, cd, cb,
            cb, cd, dd,
            cb, dd, db,
            db, dd, bb,
            // right bar: bb - dd - dc - ba
            bb, dd, bd,
            bd, dd, dc,
            bd, dc, bc,
            bc, dc, ba,
        ]);
    }

    fn add_vertices(&mut self, pass: usize, slice: &[Vertex]) {
        if self.passes.len() <= pass {
            // We only need one more, but no harm in adding extra
            self.passes.resize(pass + 8, vec![]);
        }

        self.passes[pass].extend_from_slice(slice);
    }
}
