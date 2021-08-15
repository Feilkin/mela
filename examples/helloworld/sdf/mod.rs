//! Signed Distance Fields

use mela::debug::{DebugContext, DebugDrawable};
use mela::imgui::{im_str, ComboBox};

pub mod renderer;

#[derive(Clone)]
pub enum SdfShape {
    Ball(f32),
    Cuboid(f32, f32, f32),
}

impl DebugDrawable for SdfShape {
    fn draw_debug_ui(&mut self, debug_ctx: &DebugContext) {
        match self {
            SdfShape::Ball(radius) => {
                debug_ctx.ui.input_float(im_str!("Radius"), radius).build();
            }
            SdfShape::Cuboid(hx, hy, hz) => {
                let mut half_extends = [*hx, *hy, *hz];
                let changed = debug_ctx
                    .ui
                    .input_float3(im_str!("Half extends"), &mut half_extends)
                    .build();

                if changed {
                    *hx = half_extends[0];
                    *hy = half_extends[1];
                    *hz = half_extends[2];
                }
            }
        }
    }
}

impl Default for SdfShape {
    fn default() -> Self {
        SdfShape::Ball(10.)
    }
}

#[derive(Default, Clone)]
pub struct SdfObject {
    pub smoothing: f32,
    pub shape: SdfShape,
}

impl DebugDrawable for SdfObject {
    fn draw_debug_ui(&mut self, debug_ctx: &DebugContext) {
        let mut current_index = match self.shape {
            SdfShape::Ball(_) => 0,
            SdfShape::Cuboid(_, _, _) => 1,
        };

        let changed = ComboBox::new(im_str!("Shape")).build_simple_string(
            &debug_ctx.ui,
            &mut current_index,
            &[im_str!("Ball"), im_str!("Box")],
        );

        self.shape.draw_debug_ui(debug_ctx);

        if changed {
            match current_index {
                0 => self.shape = SdfShape::Ball(1.),
                1 => self.shape = SdfShape::Cuboid(1., 1., 1.),
                _ => {}
            }
        }

        debug_ctx
            .ui
            .input_float(im_str!("Smoothing"), &mut self.smoothing)
            .build();
    }
}
