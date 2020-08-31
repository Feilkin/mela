//! Data type definitions for import

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use serde::Deserialize;

use crate::asset::tilemap::{layers, tileset};
use crate::asset::AssetError;
use crate::ecs::world::World;

#[derive(Debug, Deserialize)]
pub struct Tileset {
    pub firstgid: usize,
    pub version: String,
    pub tiledversion: String,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub spacing: Option<u32>,
    pub tilecount: usize,
    pub columns: usize,
    pub image: Image,
    pub tile: Option<Vec<Tile>>,
}

#[derive(Debug, Deserialize)]
pub struct ExternalTileset {
    pub version: String,
    pub tiledversion: String,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub spacing: Option<u32>,
    pub tilecount: usize,
    pub columns: usize,
    pub image: Image,
    pub tile: Option<Vec<Tile>>,
}

impl ExternalTileset {
    pub fn into_internal(self, firstgid: usize) -> Tileset {
        Tileset {
            firstgid,
            version: self.version,
            tiledversion: self.tiledversion,
            name: self.name,
            tilewidth: self.tilewidth,
            tileheight: self.tileheight,
            spacing: self.spacing,
            tilecount: self.tilecount,
            columns: self.columns,
            image: self.image,
            tile: self.tile,
        }
    }

    pub fn with_root_path<P: AsRef<Path>>(self, path: P) -> ExternalTileset {
        let Image { source, .. } = self.image;

        let image = Image {
            source: path.as_ref().join(source).to_string_lossy().into_owned(),
            ..self.image
        };

        ExternalTileset { image, ..self }
    }
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    pub id: usize,
    pub objectgroup: Vec<ObjectGroup>,
}

#[derive(Debug, Deserialize)]
pub struct ObjectGroup {
    pub draworder: DrawOrder,
    pub id: usize,
    pub object: Vec<Object>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum DrawOrder {
    Index,
    Topdown,
}

#[derive(Debug, Deserialize)]
pub struct Object {
    pub id: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(rename = "type")]
    pub _type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    ///	Hex-formatted color (#RRGGBB or #AARRGGBB) (optional)
    pub backgroundcolor: Option<String>,
    /// Number of tile rows
    pub height: usize,
    /// Length of the side of a hex tile in pixels (hexagonal maps only)
    pub hexsidelength: Option<usize>,
    /// Whether the map has infinite dimensions
    pub infinite: bool,
    /// Array of Layers
    pub layers: Vec<Layer>,
    /// Auto-increments for each layer
    pub nextlayerid: usize,
    /// Auto-increments for each placed object
    pub nextobjectid: usize,
    /// orthogonal, isometric, staggered or hexagonal
    pub orientation: MapOrientation,
    /// Array of Properties
    #[serde(default)]
    pub properties: Vec<Property>,
    /// right-down (the default), right-up, left-down or left-up (orthogonal maps only)
    pub renderorder: Option<RenderOrder>,
    /// x or y (staggered / hexagonal maps only)
    pub staggeraxis: Option<StaggeredAxis>,
    /// odd or even (staggered / hexagonal maps only)
    pub staggerindex: Option<StaggeredIndex>,
    /// The Tiled version used to save the file
    pub tiledversion: String,
    /// Map grid height
    pub tileheight: usize,
    /// Array of Tilesets
    pub tilesets: Vec<MaybeInlinedTilesetOrMaybeExternal>,
    /// Map grid width
    pub tilewidth: usize,
    //pub type: String, //	map (since 1.0)
    /// The JSON format version
    pub version: f64,
    /// Number of tile columns
    pub width: usize,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MaybeInlinedTilesetOrMaybeExternal {
    Inlined(Tileset),
    External { firstgid: usize, source: String },
}

impl MaybeInlinedTilesetOrMaybeExternal {
    pub fn into_tileset<P: AsRef<Path>>(self, path: P) -> Result<Tileset, AssetError> {
        use MaybeInlinedTilesetOrMaybeExternal::*;

        match self {
            Inlined(tileset) => Ok(tileset),
            External { firstgid, source } => {
                let source_path = Path::new(&source);
                let actual_path = path
                    .as_ref()
                    .parent()
                    .unwrap_or(Path::new("."))
                    .join(source_path);
                dbg!(&actual_path);
                let file = File::open(&actual_path)?;
                let reader = BufReader::new(file);
                let data: ExternalTileset = serde_xml_rs::from_reader(reader)?;

                Ok(data
                    .with_root_path(source_path.parent().unwrap_or(Path::new(".")))
                    .into_internal(firstgid))
            }
        }
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MapOrientation {
    Orthogonal,
    Isometric,
    Staggered,
    Hexagonal,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub enum RenderOrder {
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

#[derive(Debug, Deserialize)]
pub enum StaggeredAxis {
    X,
    Y,
}

#[derive(Debug, Deserialize)]
pub enum StaggeredIndex {
    Odd,
    Even,
}

#[derive(Debug, Deserialize)]
pub struct Property {
    name: String,
    value: PropertyValue,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "lowercase")]
pub enum PropertyValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Color,
    File,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Layer {
    TileLayer(TileLayer),
    ObjectGroup(ObjectLayer),
}

impl Layer {
    pub fn into_actual(self, tilesets: &[tileset::Tileset]) -> Box<dyn layers::Layer> {
        match self {
            Layer::TileLayer(layer_data) => Box::new(layer_data.build(tilesets)),
            // TODO: implement object layers
            Layer::ObjectGroup(layer_data) => Box::new(layer_data.build()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TileLayer {
    // TODO: chunks
    // TODO: compression and encoding
    //       for now we only support data as JSON array, because the documentation for the JSON
    //       format makes absolutely no sense for compression and encoding fields, so we just ignore
    //       those and assume the data is not encoded
    data: Vec<usize>,
    height: usize,
    id: usize,
    name: String,
    #[serde(default)]
    offsetx: usize,
    #[serde(default)]
    offsety: usize,
    #[serde(default)]
    properties: Vec<Property>,
    startx: Option<isize>,
    starty: Option<isize>,
    visible: bool,
    width: usize,
}

impl TileLayer {
    pub fn build(self, tilesets: &[tileset::Tileset]) -> layers::TileLayer {
        let data = self
            .data
            .into_iter()
            .map(|gid| {
                tilesets
                    .iter()
                    .filter_map(|ts| ts.tile_gid(gid))
                    .next()
                    .and_then(|tile| Some(tile.to_owned()))
            })
            .collect();

        layers::TileLayer::new(
            data,
            self.id,
            self.name,
            [self.offsetx as f32, self.offsety as f32],
            (self.width, self.height),
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct ObjectLayer {
    draworder: DrawOrder,
    id: usize,
    name: String,
    objects: Vec<Object>,
    #[serde(default)]
    offsetx: usize,
    #[serde(default)]
    offsety: usize,
    #[serde(default)]
    properties: Vec<Property>,
    visible: bool,
}

impl ObjectLayer {
    pub fn build(self) -> layers::ObjectLayer {
        let objects = self.objects.into_iter().map(|obj| obj.into()).collect();

        layers::ObjectLayer::new(objects, self.id, self.name)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Compression {
    Zlib,
    Gzip,
}
