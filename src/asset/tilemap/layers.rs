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
        self.spritebatch.update(render_ctx);
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
