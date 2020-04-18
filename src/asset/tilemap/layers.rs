//! Tilemap layers

use crate::assets::tilemap::tile::Tile;
use crate::components::physics::{Body, Material, Position};
use crate::ecs::world::{World, WorldStorage};
use crate::gfx::{Mesh, Spritebatch, RenderContext};
use glium::{Display, Frame, Program};
use nalgebra::{Matrix4, Vector2};
use ncollide2d::shape::{Cuboid, ShapeHandle};
use std::rc::Rc;
use crate::asset::tilemap::tile::Tile;
use gltf::Mesh;

pub trait Layer<W: World> {
    fn draw(&self, camera: &Matrix4<f32>, render_ctx: &mut RenderContext);

    /// Adds all entities defined by this layer to given world
    fn add_entities(&self, world: W) -> W;
}

pub struct TileLayer {
    data: Vec<Option<Tile>>,
    id: usize,
    name: String,
    offset: [f32; 2],
    size: (usize, usize),
}

impl TileLayer {
    pub fn new(
        data: Vec<Option<Tile>>,
        id: usize,
        name: String,
        offset: [f32; 2],
        size: (usize, usize),
    ) -> TileLayer {
        TileLayer {
            data,
            id,
            name,
            offset,
            size,
        }
    }
}

impl<W: World> Layer<W> for TileLayer {
    fn draw(&self, camera: &Matrix4<f32>, render_ctx: &mut RenderContext) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

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

                    let index_offset = vertices.len() as u16;

                    let (tile_vertices, tile_indices) =
                        tile.quad().vertices_and_indices(position, tile_size);
                    vertices.extend_from_slice(&tile_vertices);
                    indices.extend(tile_indices.iter().map(|i| i + index_offset));
                }
            }
        }

        Mesh::new(
            vertices,
            indices,
            Rc::clone(self.data[0].as_ref().unwrap().image().texture()),
        )
        .draw(camera, display, target, shader);
    }

    fn add_entities(&self, mut world: W) -> W {
        for row in 0..self.size.1 {
            for column in 0..self.size.0 {
                let id = column + row * self.size.0;

                if let Some(tile) = &self.data[id] {
                    for og in tile.object_groups() {
                        for obj in og.objects() {
                            let (half_width, half_height) =
                                (tile.size()[0] / 2., tile.size()[1] / 2.);

                            let shape = ShapeHandle::new(Cuboid::new(Vector2::new(
                                half_width,
                                half_height,
                            )));

                            // TODO: material from properties?
                            let material = Material {
                                friction: 0.80,
                                bounciness: 1.0,
                            };

                            let tile_body = Body {
                                shape,
                                material,
                                _static: true,
                            };

                            world = world
                                .add_entity()
                                .with_component(Position::new(
                                    column as f32 * tile.size()[0] + half_width,
                                    row as f32 * tile.size()[1] + half_height,
                                ))
                                .with_component(tile_body)
                                .build();
                        }
                    }
                }
            }
        }

        world
    }
}
