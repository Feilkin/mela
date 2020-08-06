//! glTF scene

use std::path::Path;

use gltf::Error;
use serde::{Deserialize, Serialize};

use crate::asset::{Asset, AssetError, AssetState};
use crate::gfx::RenderContext;

pub struct Scene {
    pub document: gltf::Document,
    pub buffers: Vec<gltf::buffer::Data>,
}

impl From<gltf::Error> for AssetError {
    fn from(_: Error) -> Self {
        unimplemented!()
    }
}

impl<T> Asset<Scene> for T
where
    T: AsRef<Path>,
{
    fn poll(
        self: Box<Self>,
        _render_ctx: &mut RenderContext,
    ) -> Result<AssetState<Scene>, AssetError> {
        let (document, buffers, _) = gltf::import(self.as_ref())?;

        Ok(AssetState::Done(Scene { document, buffers }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeAttributes {
    pub ground: Option<u8>,
    pub ball: Option<u8>,
}
