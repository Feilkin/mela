//! Core ECS components

use crate::debug::{DebugContext, DebugDrawable};
use crate::imgui::im_str;
use crate::na::Isometry3;
use rapier3d::prelude::{Rotation, Translation};

#[derive(Clone)]
pub struct Transform(pub Isometry3<f32>);

impl Default for Transform {
    fn default() -> Self {
        Transform(Isometry3::translation(0., 0., 0.))
    }
}

impl DebugDrawable for Transform {
    fn draw_debug_ui(&mut self, debug_ctx: &DebugContext) {
        let mut translation_data: [f32; 3] = self.0.translation.vector.into();
        let mut rotation_data: [f32; 4] = self.0.rotation.coords.into();

        if debug_ctx
            .ui
            .input_float3(im_str!("Translation"), &mut translation_data)
            .build()
        {
            self.0.translation = Translation::new(
                translation_data[0],
                translation_data[1],
                translation_data[2],
            );
        }
        if debug_ctx
            .ui
            .input_float4(im_str!("Rotation"), &mut rotation_data)
            .build()
        {
            self.0.rotation = Rotation::from_quaternion(rotation_data.into());
        }
    }
}
