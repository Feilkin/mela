//! 2D raycast lighting and more maybe

use zerocopy::{AsBytes, FromBytes};

#[derive(Debug, Clone, Copy, AsBytes, FromBytes, Default)]
#[repr(C)]
pub struct Light {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub strength: f32,
    pub angle: f32,
    pub sector: f32,
    pub _padding: [f32; 2],
}

#[derive(Debug, Clone, Copy, AsBytes, FromBytes)]
#[repr(C)]
pub struct Lights {
    lights: [Light; 30],
    num_lights: u32,
}

impl Lights {
    pub fn new(lights: &[Light]) -> Lights {
        assert!(lights.len() <= 30);

        let mut light_array = [Light::default(); 30];

        for i in 0..lights.len() {
            light_array[i] = lights[i];
        }

        Lights {
            lights: light_array,
            num_lights: lights.len() as u32,
        }
    }
}
