//! Debugging utilities

use crate::components::Transform;
use crate::ecs::storage::{Archetype, Component};
use crate::ecs::world::Entry;
use crate::ecs::{Entity, World};
use crate::imgui::__core::fmt::Formatter;
use crate::imgui::{im_str, ComboBox, ImString, TreeNode, Window};
use imgui::ImStr;
use legion::IntoQuery;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;

pub struct DebugContext<'a> {
    pub ui: imgui::Ui<'a>,
    pub ui_renderer: &'a mut imgui_wgpu::Renderer,
}

pub trait DebugDrawable {
    fn draw_debug_ui(&mut self, _debug_ctx: &DebugContext) {}
}

/// Component that can be used to give editor-visible names to entities.
pub struct DebugName(pub String);

impl Display for DebugName {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Deref for DebugName {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DebugDrawable for DebugName {
    fn draw_debug_ui(&mut self, debug_ctx: &DebugContext) {
        debug_ctx
            .ui
            .label_text(im_str!("Debug Name"), &im_str!("{}", self.0));
    }
}

pub type DebugDrawer = Box<dyn Fn(&mut Entry, &DebugContext) -> ()>;
pub type ComponentClosure = Box<dyn FnMut(&DebugContext, &mut Entry) -> bool>;
pub type ComponentClosureBuilder = Box<dyn Fn() -> ComponentClosure>;

pub struct DebugDrawers {
    pub drawers: HashMap<TypeId, DebugDrawer>,
    component_closure_builders: HashMap<TypeId, ComponentClosureBuilder>,
    component_kinds: HashMap<String, TypeId>,
    ui_current_component_index: usize,
    ui_component_names: Vec<ImString>,
    ui_current_new_component: Option<ComponentClosure>,
}

impl Default for DebugDrawers {
    fn default() -> Self {
        let mut drawers = DebugDrawers {
            drawers: Default::default(),
            component_closure_builders: Default::default(),
            component_kinds: Default::default(),
            ui_current_component_index: 0,
            ui_component_names: vec![],
            ui_current_new_component: None,
        };

        drawers.insert::<DebugName>();
        drawers.insert_addable::<Transform>();

        drawers
    }
}

impl DebugDrawers {
    pub fn insert<C: Any + Component + DebugDrawable>(&mut self) {
        self.drawers.insert(
            TypeId::of::<C>(),
            Box::new(|entry, debug_ctx| {
                if let Ok(component) = entry.get_component_mut::<C>() {
                    debug_ctx.ui.text(im_str!("{}", std::any::type_name::<C>()));
                    component.draw_debug_ui(debug_ctx);
                    debug_ctx.ui.spacing();
                }
            }),
        );
    }

    pub fn insert_addable<C: Any + Component + DebugDrawable + Default + Clone>(&mut self) {
        let type_id = TypeId::of::<C>();
        let type_name = std::any::type_name::<C>();

        self.insert::<C>();

        self.component_kinds.insert(type_name.to_string(), type_id);
        self.ui_component_names
            .push(ImString::from(type_name.to_string()));

        let closure_builder = Box::new(|| {
            let mut value = C::default();

            Box::new(move |debug_ctx: &DebugContext, entry: &mut Entry| {
                value.draw_debug_ui(debug_ctx);

                if debug_ctx.ui.button(im_str!("Add"), [100., 25.]) {
                    entry.add_component(value.clone());
                    debug_ctx.ui.close_current_popup();

                    return true;
                }

                false
            }) as Box<dyn FnMut(&DebugContext, &mut Entry) -> bool>
        });

        self.component_closure_builders
            .insert(type_id, closure_builder);
    }

    pub fn for_archetype(&self, archetype: &Archetype) -> Vec<&DebugDrawer> {
        archetype
            .layout()
            .component_types()
            .iter()
            .filter_map(|c| self.drawers.get(&c.type_id()))
            .collect::<Vec<_>>()
    }

    #[profiling::function]
    pub fn draw_world(&mut self, world: &mut World, debug_ctx: &mut DebugContext) -> () {
        // TODO: figure out how we can get filtering done from the UI
        let mut query = <Entity>::query();

        // this is stupid and slow
        let mut entities = Vec::new();

        for chunk in query.iter_chunks(world) {
            for entity in chunk {
                entities.push(*entity);
            }
        }

        if let Some(token) = Window::new(&im_str!("Entities")).begin(&debug_ctx.ui) {
            if debug_ctx.ui.button(im_str!("Add entity"), [100., 25.]) {
                world.push(());
            }

            let id_token = debug_ctx.ui.push_id(im_str!("entity-list"));
            for (i, entity) in entities.into_iter().enumerate() {
                let mut entry = world.entry(entity).unwrap();

                let id = im_str!("Entity: {}", i);
                let mut label = id.clone();
                let mut node = TreeNode::new(&id);

                if let Some(debug_name) = entry.get_component::<DebugName>().ok() {
                    label.clear();
                    label.push_str(debug_name.as_str());
                    node = node.label(&label);
                }

                if let Some(tree_token) = node.push(&debug_ctx.ui) {
                    {
                        self.for_archetype(entry.archetype())
                            .iter()
                            .for_each(|d| d(&mut entry, debug_ctx));
                    }

                    if debug_ctx.ui.button(&im_str!("Add component"), [100., 20.]) {
                        debug_ctx.ui.open_popup(im_str!("add-component-modal"));
                    }

                    debug_ctx
                        .ui
                        .popup_modal(im_str!("add-component-modal"))
                        .always_auto_resize(true)
                        .build(|| {
                            let current_item = &mut self.ui_current_component_index;
                            if ComboBox::new(im_str!("Component")).build_simple_string(
                                &debug_ctx.ui,
                                current_item,
                                &self.ui_component_names.iter().collect::<Vec<_>>(),
                            ) {
                                let kind = &self.component_kinds
                                    [self.ui_component_names[*current_item].to_str()];
                                let builder = &self.component_closure_builders[kind];
                                self.ui_current_new_component = Some(builder());
                            }

                            debug_ctx.ui.spacing();

                            let was_added = if let Some(component_closure) =
                                self.ui_current_new_component.as_mut()
                            {
                                component_closure(debug_ctx, &mut entry)
                            } else {
                                false
                            };

                            if was_added {
                                self.ui_current_new_component = None;
                            }

                            debug_ctx.ui.spacing();

                            if debug_ctx.ui.button(&im_str!("Close"), [100., 25.]) {
                                debug_ctx.ui.close_current_popup();
                            }
                        });
                    tree_token.pop(&debug_ctx.ui);
                };
            }

            id_token.pop(&debug_ctx.ui);
            token.end(&debug_ctx.ui);
        };
    }
}
