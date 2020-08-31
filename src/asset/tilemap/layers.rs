//! Tilemap layers

use nalgebra::Matrix4;
use wgpu::TextureView;

use crate::asset::tilemap::tile::Tile;
use crate::asset::tilemap::Object;
use crate::asset::{Asset, AssetState};
use crate::ecs::world::World;
use crate::gfx::{RenderContext, Spritebatch, Texture};

pub trait Layer {
    fn update(&mut self, render_ctx: &mut RenderContext);

    fn objects(&self) -> &[Object] {
        &[]
    }

    fn draw(&self, camera: &Matrix4<f32>, render_ctx: &mut RenderContext);
    fn draw_to(
        &self,
        camera: &Matrix4<f32>,
        view: &[&wgpu::TextureView],
        render_ctx: &mut RenderContext,
    );
}

pub struct TileLayer {
    data: Vec<Option<Tile>>,
    id: usize,
    name: String,
    offset: [f32; 2],
    size: (usize, usize),
    spritebatch: Spritebatch,
    material_spritebatch: Option<Spritebatch>,
}

impl TileLayer {
    pub fn new(
        data: Vec<Option<Tile>>,
        id: usize,
        name: String,
        offset: [f32; 2],
        size: (usize, usize),
    ) -> TileLayer {
        let texture = data
            .iter()
            .find(|t| t.is_some())
            .unwrap()
            .as_ref()
            .unwrap()
            .texture()
            .clone();

        let mut spritebatch = Spritebatch::new(texture);

        for row in 0..size.1 {
            for column in 0..size.0 {
                let id = column + row * size.0;

                // TODO: tilesize needs to come from parent, not from tile
                let tile_size = [16., 16.];

                if let Some(tile) = &data[id] {
                    let position = [
                        offset[0] + column as f32 * tile_size[0],
                        offset[1] + row as f32 * tile_size[1],
                    ];

                    spritebatch.add_quad(tile.quad(), position);
                }
            }
        }

        TileLayer {
            spritebatch,
            material_spritebatch: None,
            data,
            id,
            name,
            offset,
            size,
        }
    }
}

impl Layer for TileLayer {
    fn update(&mut self, render_ctx: &mut RenderContext) {
        if self.material_spritebatch.is_none() {
            // FIXME: get rid of this :D
            let mut texture_asset: Box<dyn Asset<Texture>> =
                Box::new("assets/spritesheets/ld46_mat.png");

            let material_texture = loop {
                match texture_asset.poll(render_ctx).unwrap() {
                    AssetState::Done(texture) => break texture,
                    AssetState::Loading(new_state) => texture_asset = new_state,
                }
            };
            let mut material_spritebatch = Spritebatch::new(material_texture);

            for row in 0..self.size.1 {
                for column in 0..self.size.0 {
                    let id = column + row * self.size.0;

                    // TODO: tilesize needs to come from parent, not from tile
                    let tile_size = [16., 16.];

                    if let Some(tile) = &self.data[id] {
                        let position = [
                            self.offset[0] + column as f32 * tile_size[0],
                            self.offset[1] + row as f32 * tile_size[1],
                        ];

                        material_spritebatch.add_quad(tile.quad(), position);
                    }
                }
            }

            self.material_spritebatch = Some(material_spritebatch);
        }

        self.spritebatch.update(render_ctx);
        self.material_spritebatch
            .as_mut()
            .unwrap()
            .update(render_ctx);
    }

    fn draw(&self, camera: &Matrix4<f32>, render_ctx: &mut RenderContext) {
        self.spritebatch.draw(camera, render_ctx);
    }

    fn draw_to(
        &self,
        camera: &Matrix4<f32>,
        view: &[&TextureView],
        render_ctx: &mut RenderContext,
    ) {
        self.spritebatch.draw_to(camera, view[0], render_ctx);

        if let Some(ref msb) = self.material_spritebatch {
            msb.draw_to(camera, view[1], render_ctx);
        }
    }
}

pub struct ObjectLayer {
    objects: Vec<Object>,
    id: usize,
    name: String,
}

// TODO: impl ObjectLayer
impl ObjectLayer {
    pub fn new(objects: Vec<Object>, id: usize, name: String) -> ObjectLayer {
        ObjectLayer { id, name, objects }
    }
}

impl Layer for ObjectLayer {
    fn update(&mut self, _render_ctx: &mut RenderContext) {
        // TODO: implement
    }

    fn objects(&self) -> &[Object] {
        &self.objects
    }

    fn draw(&self, _camera: &Matrix4<f32>, _render_ctx: &mut RenderContext) {
        // TODO: implement
    }

    fn draw_to(
        &self,
        _camera: &Matrix4<f32>,
        _view: &[&TextureView],
        _render_ctx: &mut RenderContext,
    ) {
        // TODO: implement
    }
}
