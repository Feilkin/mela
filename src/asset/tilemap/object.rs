//! Tilemap objects

use crate::asset::tilemap::data;
use crate::asset::tilemap::data::DrawOrder;
use crate::debug::DebugDrawable;

#[derive(Debug, Clone)]
pub struct Object {
    id: usize,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl DebugDrawable for Object {}

impl From<data::Object> for Object {
    fn from(data: data::Object) -> Self {
        Object {
            id: data.id,
            x: data.x as f32,
            y: data.y as f32,
            width: data.width as f32,
            height: data.height as f32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ObjectGroup {
    draw_order: DrawOrder,
    id: usize,
    objects: Vec<Object>,
}

impl ObjectGroup {
    pub fn objects(&self) -> &[Object] {
        &self.objects
    }
}

impl DebugDrawable for ObjectGroup {}

impl From<data::ObjectGroup> for ObjectGroup {
    fn from(data: data::ObjectGroup) -> Self {
        ObjectGroup {
            draw_order: data.draworder,
            id: data.id,
            objects: data.object.into_iter().map(|o| o.into()).collect(),
        }
    }
}
