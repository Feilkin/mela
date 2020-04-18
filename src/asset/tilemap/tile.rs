//! Tile

use crate::assets::tilemap::{data, ObjectGroup, Tileset};
use crate::assets::Image;
use crate::debug::DebugDrawable;
use crate::gfx::Quad;
use imgui::{TextureId, Ui};
use imgui_glium_renderer::Renderer;
use std::rc::Rc;

#[derive(Clone)]
pub struct Tile {
    id: usize,
    object_groups: Vec<ObjectGroup>,
    size: [f32; 2],
    position: [f32; 2],
    source_image: Image,
    debug_texture_id: Option<TextureId>,
}

impl Tile {
    pub fn new(id: usize, position: [f32; 2], size: [f32; 2], source_image: Image) -> Tile {
        Tile {
            object_groups: Vec::new(),
            debug_texture_id: None,
            id,
            position,
            size,
            source_image,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn set_object_groups(&mut self, object_groups: Vec<ObjectGroup>) {
        self.object_groups = object_groups;
    }

    pub fn object_groups(&self) -> &[ObjectGroup] {
        &self.object_groups
    }

    pub fn quad(&self) -> Quad {
        Quad::new(self.position, self.size, self.source_image.dimensions())
    }

    pub fn image(&self) -> &Image {
        &self.source_image
    }

    pub fn size(&self) -> [f32; 2] {
        self.size
    }
}

impl DebugDrawable for Tile {
    fn draw_debug_ui(&mut self, ui: &Ui, renderer: &mut Renderer) {
        use imgui::*;

        ui.text(im_str!("Tile [{}]", self.id));

        if self.debug_texture_id.is_none() {
            self.debug_texture_id = Some(
                renderer
                    .textures()
                    .insert(Rc::clone(self.source_image.texture().into())),
            );
        }

        Image::new(self.debug_texture_id.unwrap(), self.size)
            .uv0([
                self.position[0] / self.source_image.dimensions().0 as f32,
                self.position[1] / self.source_image.dimensions().1 as f32,
            ])
            .uv1([
                (self.position[0] + self.size[0]) / self.source_image.dimensions().0 as f32,
                (self.position[1] + self.size[1]) / self.source_image.dimensions().1 as f32,
            ])
            .build(ui);

        if self.object_groups.len() > 0 {
            ui.tree_node(&im_str!("tile-{}-og", self.id))
                .label(im_str!("Object groups"))
                .build(|| {
                    for og in &mut self.object_groups {
                        og.draw_debug_ui(ui, renderer);
                    }
                });
        }
    }
}
