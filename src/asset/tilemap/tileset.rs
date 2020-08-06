//! Tiled tilesets

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use crate::asset::{
    tilemap::{data, tile::Tile, ObjectGroup},
    Asset, AssetError, AssetState,
};
use crate::debug::DebugDrawable;
use crate::gfx::primitives::Quad;
use crate::gfx::{RenderContext, Texture};

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
        _first_gid: usize,
        render_ctx: &mut RenderContext,
    ) -> Result<Tileset, AssetError> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        let data: data::Tileset = serde_xml_rs::from_reader(reader)?;

        Tileset::build(data, path.as_ref(), render_ctx)
    }

    pub fn build<P: AsRef<Path>>(
        data: data::Tileset,
        path: P,
        render_ctx: &mut RenderContext,
    ) -> Result<Tileset, AssetError> {
        // TODO: fix this very lazy hack
        let mut texture_asset: Box<dyn Asset<Texture>> = Box::new(
            path.as_ref()
                .parent()
                .unwrap_or(Path::new("."))
                .join(data.image.source),
        );

        let texture = loop {
            match texture_asset.poll(render_ctx).unwrap() {
                AssetState::Done(texture) => break texture,
                AssetState::Loading(new_state) => texture_asset = new_state,
            }
        };

        let mut tiles = Vec::with_capacity(data.tilecount);

        let columns = data.columns;
        let rows = data.tilecount / columns;
        let tile_size = [data.tilewidth as f32, data.tileheight as f32];
        let source_size = [data.image.width as f32, data.image.height as f32];

        for row in 0..rows {
            for column in 0..columns {
                let id = column + row * columns;
                let position = [
                    ((data.spacing.unwrap_or(0) + data.tilewidth) * column as u32) as f32,
                    ((data.spacing.unwrap_or(0) + data.tileheight) * row as u32) as f32,
                ];

                let quad = Quad::new(
                    position[0],
                    position[1],
                    tile_size[0],
                    tile_size[1],
                    source_size[0],
                    source_size[1],
                );

                tiles.insert(id, Tile::new(id, quad, texture.clone()));
            }
        }

        match data.tile {
            Some(data_tiles) => {
                for tile in data_tiles {
                    tiles[tile.id].set_object_groups(
                        tile.objectgroup
                            .into_iter()
                            .map(ObjectGroup::from)
                            .collect(),
                    );
                }
            }
            None => (),
        }

        Ok(Tileset {
            texture,
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
    fn draw_debug_ui(&mut self, _render_ctx: &mut RenderContext) {
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
