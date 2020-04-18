//! Tiled tilesets

use crate::assets::tilemap::tile::Tile;
use crate::assets::tilemap::{data, ObjectGroup};
use crate::assets::{AssetError, Image};
use crate::debug::{DebugDrawable, DebugContext};
use crate::gfx::{Texture, RenderContext};
use glium::Display;
use imgui::Ui;
use imgui_glium_renderer::Renderer;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::rc::Rc;
use crate::asset::tilemap::tile::Tile;

pub struct Tileset {
    first_gid: usize,
    texture: Texture,
    tiles: Vec<Tile>,
    tile_size: (u32, u32),
    source_size: (u32, u32),
    name: String,
}

impl Tileset {
    pub fn from_file<P: AsRef<Path>>(
        path: P,
        display: &Display,
        first_gid: usize,
    ) -> Result<Tileset, AssetError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let data: data::Tileset = serde_xml_rs::from_reader(reader)?;

        Tileset::build(data, path.as_ref(), display)
    }

    pub fn build<P: AsRef<Path>>(
        data: data::Tileset,
        path: P,
        display: &Display,
    ) -> Result<Tileset, AssetError> {
        let image = Image::from_file(
            path.as_ref()
                .parent()
                .unwrap_or(Path::new("."))
                .join(data.image.source),
            display,
        )?;
        let mut tiles = Vec::with_capacity(data.tilecount);

        let columns = data.columns;
        let rows = data.tilecount / columns;
        let tile_size = [data.tilewidth as f32, data.tileheight as f32];

        for row in 0..rows {
            for column in 0..columns {
                let id = column + row * columns;
                let position = [
                    ((data.spacing + data.tilewidth) * column as u32) as f32,
                    ((data.spacing + data.tileheight) * row as u32) as f32,
                ];

                tiles.insert(id, Tile::new(id, position, tile_size, image.clone()));
            }
        }

        for tile in data.tile {
            tiles[tile.id].set_object_groups(
                tile.objectgroup
                    .into_iter()
                    .map(ObjectGroup::from)
                    .collect(),
            );
        }

        Ok(Tileset {
            texture: image,
            tiles,
            first_gid: data.firstgid,
            tile_size: (data.tilewidth, data.tileheight),
            source_size: (data.image.width, data.image.height),
            name: data.name,
        })
    }

    pub fn tile(&self, id: usize) -> &Tile {
        &self.tiles[id]
    }

    pub fn tile_gid(&self, id: usize) -> Option<&Tile> {
        if id < self.first_gid {
            return None;
        }

        if id > self.first_gid + self.tiles.len() {
            return None;
        }

        Some(&self.tiles[id - self.first_gid])
    }
}

impl DebugDrawable for Tileset {
    fn draw_debug_ui(&mut self, render_ctx: &mut RenderContext) {
//        use imgui::*;
//
//        ui.text(&im_str!("name: {}", self.name));
//
//        ui.tree_node(&im_str!("tileset-{}-tiles", self.name))
//            .label(im_str!("Tiles"))
//            .build(|| {
//                for tile in &mut self.tiles {
//                    ui.tree_node(&im_str!("tileset-{}-tile-{}", self.name, tile.id()))
//                        .label(&im_str!("{}", tile.id()))
//                        .build(|| tile.draw_debug_ui(ui, renderer));
//                }
//            });
    }
}
