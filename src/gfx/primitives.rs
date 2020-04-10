//! Graphical primitives

use zerocopy::{AsBytes, FromBytes};

use crate::gfx::Texture;

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_coords: [f32; 2],
    pub color: [f32; 4],
}

pub struct Quad {
    position: [f32; 2],
    size: [f32; 2],
    source_size: [f32; 2],
}

impl Quad {
    pub fn new(x: f32, y: f32, width: f32, height: f32, sw: f32, sh: f32) -> Quad {
        Quad {
            position: [x, y],
            size: [width, height],
            source_size: [sw, sh],
        }
    }

    pub fn vertices_and_indices(
        &self,
        translation: [f32; 3],
        color: [f32; 4],
    ) -> ([Vertex; 4], [u16; 6]) {
        let [sw, sh] = self.source_size;

        // left
        let x0 = self.position[0] / sw;
        // top
        let y0 = self.position[1] / sh;
        // right
        let x1 = x0 + self.size[0] / sw;
        // down
        let y1 = y0 + self.size[1] / sh;
        let z = translation[2];

        // make normal face Z axis because we lazy
        let normal = [0., 0., -1.];

        (
            [
                // top left
                Vertex {
                    position: [translation[0], translation[1], z],
                    normal,
                    color,
                    texture_coords: [x0, y0],
                },
                // top right
                Vertex {
                    position: [x1 + translation[0], y0 + translation[1], z],
                    normal,
                    color,
                    texture_coords: [x1, y0],
                },
                // bottom left
                Vertex {
                    position: [x0 + translation[0], y1 + translation[1], z],
                    normal,
                    color,
                    texture_coords: [x0, y1],
                },
                // bottom right
                Vertex {
                    position: [x1 + translation[0], y1 + translation[1], z],
                    normal,
                    color,
                    texture_coords: [x1, y1],
                },
            ],
            [0, 1, 3, 0, 3, 2],
        )
    }
}

// TODO: wtf is this??
#[derive(Debug, Clone, Copy, AsBytes, FromBytes)]
#[repr(C)]
pub struct MVP {
    pub model: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
}
