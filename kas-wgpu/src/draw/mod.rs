// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License in the LICENSE-APACHE file or at:
//     https://www.apache.org/licenses/LICENSE-2.0

//! Drawing API for `kas_wgpu`
//!
//! Extensions to the API of [`kas::draw`], plus some utility types.

mod custom;
mod draw_pipe;
mod draw_text;
mod flat_round;
mod shaded_round;
mod shaded_square;
mod shaders;
mod vector;

use kas::geom::Rect;
use wgpu_glyph::GlyphBrush;

pub(crate) use flat_round::FlatRound;
pub(crate) use shaded_round::ShadedRound;
pub(crate) use shaded_square::ShadedSquare;
pub(crate) use shaders::ShaderManager;

pub use custom::{CustomPipe, CustomPipeBuilder, DrawCustom};
pub use vector::{Quad, Vec2};

/// 3-part colour data
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Rgb {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl From<kas::draw::Colour> for Rgb {
    fn from(c: kas::draw::Colour) -> Self {
        Rgb {
            r: c.r,
            g: c.g,
            b: c.b,
        }
    }
}

/// `kas-wgpu`'s implemention of [`kas::draw::Draw`] and friends
pub struct DrawPipe<C> {
    clip_regions: Vec<Rect>,
    shaded_round: ShadedRound,
    shaded_square: ShadedSquare,
    custom: C,
    flat_round: FlatRound,
    glyph_brush: GlyphBrush<'static, ()>,
}
