//! glTF scene

use std::path::Path;

use gltf::Error;

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
        render_ctx: &mut RenderContext,
    ) -> Result<AssetState<Scene>, AssetError> {
        let (document, buffers, _) = gltf::import(self.as_ref())?;

        Ok(AssetState::Done(Scene { document, buffers }))
    }
}
