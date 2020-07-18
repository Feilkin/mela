//! Tilemap layers

use std::path::Path;
use std::rc::Rc;

use gltf::Mesh;
use nalgebra::{Matrix, Matrix4, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use wgpu::TextureView;

use crate::asset::tilemap::tile::Tile;
use crate::asset::tilemap::Object;
use crate::asset::{Asset, AssetState};
use crate::ecs::world::{World, WorldStorage};
use crate::gfx::{RenderContext, Spritebatch, Texture};

pub trait Layer<W: World> {
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

    /// Adds all entities defined by this layer to given world
    fn add_entities(&self, world: W) -> W;
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

impl<W: World> Layer<W> for TileLayer {
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

    fn add_entities(&self, mut world: W) -> W {
        for row in 0..self.size.1 {
            for column in 0..self.size.0 {
                let id = column + row * self.size.0;

                if let Some(tile) = &self.data[id] {
                    for og in tile.object_groups() {
                        for obj in og.objects() {
                            // FIXME: object adding
                            //                            let (half_width, half_height) =
                            //                                (tile.size()[0] / 2., tile.size()[1] / 2.);
                            //
                            //                            let shape = ShapeHandle::new(Cuboid::new(Vector2::new(
                            //                                half_width,
                            //                                half_height,
                            //                            )));
                            //
                            //                            // TODO: material from properties?
                            //                            let material = Material {
                            //                                friction: 0.80,
                            //                                bounciness: 1.0,
                            //                            };
                            //
                            //                            let tile_body = Body {
                            //                                shape,
                            //                                material,
                            //                                _static: true,
                            //                            };
                            //
                            //                            world = world
                            //                                .add_entity()
                            //                                .with_component(Position::new(
                            //                                    column as f32 * tile.size()[0] + half_width,
                            //                                    row as f32 * tile.size()[1] + half_height,
                            //                                ))
                            //                                .with_component(tile_body)
                            //                                .build();
                        }
                    }
                }
            }
        }

        world
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

impl<W: World> Layer<W> for ObjectLayer {
    fn update(&mut self, render_ctx: &mut RenderContext) {
        // TODO: implement
    }

    fn objects(&self) -> &[Object] {
        &self.objects
    }

    fn draw(&self, camera: &Matrix4<f32>, render_ctx: &mut RenderContext) {
        // TODO: implement
    }

    fn draw_to(
        &self,
        camera: &Matrix4<f32>,
        view: &[&TextureView],
        render_ctx: &mut RenderContext,
    ) {
        // TODO: implement
    }

    fn add_entities(&self, world: W) -> W {
        // TODO: implement
        unimplemented!()
    }
}
