//! 2D raycast lighting and more maybe

use nalgebra::{Isometry3, Matrix4, Translation3, UnitQuaternion, Vector3};
use zerocopy::{AsBytes, FromBytes};

/// Wrapper for data sent to the GPU
#[derive(Debug, Default, Clone, Copy, PartialEq, AsBytes, FromBytes)]
#[repr(C)]
pub struct LightData {
    pub view_matrix: [[f32; 4]; 4],
    pub direction: [f32; 3],
    pub _padding: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

/// Direction light source, like sun :)
#[derive(Debug, Clone, Default)]
pub struct DirectionalLight {
    direction: [f32; 3],
    color: [f32; 3],
    intensity: f32,
}

// constructors
impl DirectionalLight {
    pub fn new(direction: [f32; 3], color: [f32; 3], intensity: f32) -> DirectionalLight {
        DirectionalLight {
            direction,
            color,
            intensity,
        }
    }
}

// public methods
// TODO: abstract to a trait
impl DirectionalLight {
    pub fn light_data(&self, transform: &Matrix4<f32>) -> LightData {
        LightData {
            view_matrix: self.view_matrix(transform).into(),
            direction: self.direction,
            color: self.color,
            intensity: self.intensity,
            _padding: 0.,
        }
    }
}

// private methods
impl DirectionalLight {
    fn view_matrix(&self, transform: &Matrix4<f32>) -> Matrix4<f32> {
        let near_plane = 0.001f32;
        let far_plane = 10.0f32;
        let light_projection = Matrix4::new_orthographic(-1., 1., -1., 1., near_plane, far_plane);

        let isometry: Isometry3<f32> = nalgebra::try_convert_ref(transform).unwrap();

        light_projection
            * (UnitQuaternion::look_at_rh(
                &isometry
                    .rotation
                    .transform_vector(&Vector3::new(0., 0., -1.)),
                &Vector3::y(),
            ) * Translation3::from(Vector3::new(
                -isometry.translation.x,
                -isometry.translation.y,
                -isometry.translation.z,
            )))
            .to_homogeneous()
    }
}
