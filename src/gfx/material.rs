//! Rendering materials

use gltf::Material;
use zerocopy::AsBytes;

const MAX_MATERIALS: usize = 256;

/// Metallic-Roughness model material.
// TODO: implement material textures
#[derive(Clone, Copy, AsBytes)]
#[repr(C)]
pub struct MetallicRoughness {
    base_color: [f32; 4],
    metallic_factor: f32,
    roughness_factor: f32,
    __padding: [f32; 2],
}

impl Default for MetallicRoughness {
    fn default() -> Self {
        MetallicRoughness {
            base_color: [1.0, 0.0, 1.0, 1.0],
            metallic_factor: 0.0,
            roughness_factor: 0.5,
            __padding: [0., 0.],
        }
    }
}

impl<'a> From<gltf::Material<'a>> for MetallicRoughness {
    fn from(data: Material<'a>) -> Self {
        let pbr_mr = data.pbr_metallic_roughness();

        MetallicRoughness {
            base_color: pbr_mr.base_color_factor(),
            metallic_factor: pbr_mr.metallic_factor(),
            roughness_factor: pbr_mr.roughness_factor(),
            __padding: [0., 0.],
        }
    }
}

/// Container type for materials of a scene
#[derive(AsBytes)]
#[repr(C)]
pub struct Materials {
    materials: [MetallicRoughness; MAX_MATERIALS],
}

impl Materials {
    pub fn material(&self, id: usize) -> MetallicRoughness {
        self.materials[id].clone()
    }
}

impl<'a, T> From<T> for Materials
where
    T: Iterator<Item = gltf::Material<'a>>,
{
    fn from(data: T) -> Self {
        let mut materials = [MetallicRoughness::default(); MAX_MATERIALS];
        let mut last_index = 0;
        for (i, mat) in data.map(MetallicRoughness::from).enumerate() {
            materials[i] = mat;
            last_index = i;
        }

        Materials { materials }
    }
}
