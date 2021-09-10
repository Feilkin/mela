//! here we go again

use std::collections::HashMap;
use std::io::{BufReader, BufWriter, Read};
use std::path::{Path, PathBuf};
use std::time::Duration;

use eyre::{Result, WrapErr};
use serde::{
    de::{Deserialize, DeserializeSeed},
    Serialize,
};
use winit::{event::Event, event_loop::ControlFlow};

use crate::debug::{DebugContext, DebugDrawable, DebugDrawers};
use crate::ecs::query::Any;
use crate::ecs::serialize::Canon;
use crate::ecs::storage::IntoComponentSource;
use crate::ecs::systems::{Builder as ScheduleBuilder, Runnable};
use crate::ecs::{Entity, Registry, Resources, Schedule, World};
use crate::gfx::MiddlewareRenderer;
use crate::na::Isometry3;
use crate::wgpu::util::StagingBelt;
use crate::wgpu::{CommandEncoder, Device, TextureView};
use crate::wgpu::{RenderPassColorAttachment, RenderPassDescriptor, TextureFormat};
use crate::winit::event::{ElementState, MouseButton, WindowEvent};
use crate::Delta;

use crate::components::Transform;
use legion::storage::Component;
use rapier3d::dynamics::{
    CCDSolver, IntegrationParameters, IslandManager, JointSet, RigidBodyBuilder, RigidBodySet,
};
use rapier3d::geometry::BroadPhase;
use rapier3d::pipeline::PhysicsPipeline;
use rapier3d::prelude::{ColliderBuilder, ColliderSet, NarrowPhase};
use rodio::OutputStream;

pub struct PhysicsStuff {
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub joint_set: JointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
}

impl PhysicsStuff {
    pub fn new() -> PhysicsStuff {
        let mut collider_set = ColliderSet::new();

        PhysicsStuff {
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            joint_set: JointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set,
        }
    }
}

impl Default for PhysicsStuff {
    fn default() -> Self {
        PhysicsStuff::new()
    }
}
pub trait Playable: Sized {
    /// Advances this game to next state
    fn update(&mut self, delta: Duration, debug_ctx: &mut DebugContext);

    /// Handle window events
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;

    /// Renders this game
    fn redraw(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame: &TextureView,
        staging_belt: &mut StagingBelt,
        debug_ctx: &mut DebugContext,
    ) -> ();
}

#[derive(Debug, Clone)]
pub struct MouseEvent {
    pub position: [f32; 2],
    pub button: MouseButton,
    pub state: ElementState,
}

impl MouseEvent {
    pub fn new(position: [f32; 2], button: MouseButton, state: ElementState) -> MouseEvent {
        MouseEvent {
            position,
            button,
            state,
        }
    }
}

#[derive(Default, Clone)]
pub struct IoState {
    pub mouse_position: [f32; 2],
    pub mouse_buttons: [bool; 3],
    pub keys: HashMap<winit::event::ScanCode, bool>,
    pub last_frame_keys: HashMap<winit::event::ScanCode, bool>,
    pub last_frame_mouse_buttons: [bool; 3],
    pub events: Vec<MouseEvent>,
}

impl IoState {
    pub fn set_key(&mut self, key: winit::event::ScanCode, state: bool) {
        self.keys.insert(key, state);
    }

    pub fn is_down(&self, key: winit::event::ScanCode) -> bool {
        *self.keys.get(&key).unwrap_or(&false)
    }

    pub fn pressed(&self, key: winit::event::ScanCode) -> bool {
        self.last_frame_keys
            .get(&key)
            .and_then(|last_state| {
                let cur_state = self.keys.get(&key).unwrap_or(&false);
                Some(*last_state == false && *cur_state == true)
            })
            .unwrap_or_else(|| *self.keys.get(&key).unwrap_or(&false))
    }

    pub fn mouse_pressed(&self, mouse_button: usize) -> bool {
        (self.last_frame_mouse_buttons[mouse_button] == false)
            && (self.mouse_buttons[mouse_button] == true)
    }

    pub fn update(&mut self) {
        self.last_frame_keys = self.keys.clone();
        self.last_frame_mouse_buttons = self.mouse_buttons.clone();
        self.events.clear();
    }
}

pub(crate) fn load_scene<P: AsRef<Path>>(path: P, registry: &Registry<String>) -> Result<World> {
    let file = std::fs::File::open(path.as_ref()).wrap_err_with(|| {
        format!(
            "Failed to read scene from {}",
            path.as_ref().to_string_lossy()
        )
    })?;
    let reader = BufReader::new(file);
    let json: serde_json::Value = serde_json::from_reader(reader).wrap_err_with(|| {
        format!(
            "Failed to parse scene {} to JSON",
            path.as_ref().to_string_lossy()
        )
    })?;

    let ret = registry
        .as_deserialize(&Canon::default())
        .deserialize(json)?;

    Ok(ret)
}

pub(crate) fn save_scene<P: AsRef<Path>>(
    world: &World,
    path: P,
    registry: &mut Registry<String>,
) -> Result<()> {
    let file = std::fs::File::create(path.as_ref()).wrap_err_with(|| {
        format!(
            "Failed to create scene file {}",
            path.as_ref().to_string_lossy()
        )
    })?;

    let writer = BufWriter::new(file);

    let json =
        serde_json::to_value(&world.as_serializable(Any::default(), registry, &Canon::default()))?;

    serde_json::to_writer(writer, &json)?;

    Ok(())
}

/// Builder for Scene-based game
pub struct SceneGameBuilder {
    registry: Registry<String>,
    debug_drawers: DebugDrawers,
    stupid_ass_whole_world_accessor:
        Box<dyn Fn(crate::ecs::systems::SystemBuilder) -> crate::ecs::systems::SystemBuilder>,
    scene_dir: String,
    first_scene: String,
    renderers: Vec<Box<dyn Fn(&Device, &TextureFormat, [f32; 2]) -> Box<dyn MiddlewareRenderer>>>,
    schedule: ScheduleBuilder,
}

impl SceneGameBuilder {
    pub fn new() -> SceneGameBuilder {
        let mut builder = SceneGameBuilder {
            registry: Registry::default(),
            debug_drawers: Default::default(),
            stupid_ass_whole_world_accessor: Box::new(|builder| builder),
            scene_dir: "./scenes".to_string(),
            first_scene: "splash.json".to_string(),
            renderers: Vec::new(),
            schedule: Schedule::builder(),
        };

        builder.register_addable_component::<Transform>()
    }

    pub fn barrier(mut self) -> SceneGameBuilder {
        self.schedule.flush();
        self
    }

    pub fn with_system<S: crate::ecs::systems::ParallelRunnable + 'static>(
        mut self,
        system: S,
    ) -> SceneGameBuilder {
        self.schedule.add_system(system);

        self
    }

    pub fn with_renderer<R: MiddlewareRenderer + 'static>(mut self) -> SceneGameBuilder {
        self.renderers
            .push(Box::new(|device, texture_format, screen_size| {
                Box::new(R::new(device, texture_format, screen_size))
            }));

        self
    }

    pub fn register_component<C: Component + DebugDrawable>(mut self) -> SceneGameBuilder {
        self.debug_drawers.insert::<C>();

        self
    }

    pub fn register_addable_component<
        C: Component + DebugDrawable + DebugDrawable + Default + Clone,
    >(
        mut self,
    ) -> SceneGameBuilder {
        self.debug_drawers.insert_addable::<C>();

        let previous_builder = self.stupid_ass_whole_world_accessor;
        self.stupid_ass_whole_world_accessor =
            Box::new(move |builder| (*previous_builder)(builder).write_component::<C>());

        self
    }

    pub fn system_builder_for_accessing_whole_known_world(
        &self,
        name: &'static str,
    ) -> crate::ecs::SystemBuilder {
        (*self.stupid_ass_whole_world_accessor)(crate::ecs::SystemBuilder::new(name))
    }

    pub fn build(mut self) -> SceneGame {
        // load first scene
        let mut world = load_scene(
            Path::new(&self.scene_dir).join(&self.first_scene),
            &self.registry,
        )
        .unwrap_or_default();

        let mut resources = Resources::default();
        resources.insert(PhysicsStuff::default());
        let (stream, handle) = OutputStream::try_default().unwrap();
        resources.insert(stream);
        resources.insert(handle);

        SceneGame {
            registry: self.registry,
            debug_drawers: self.debug_drawers,
            scene_dir: self.scene_dir,
            schedule: self.schedule.build(),
            world,
            resources,
            io_state: Default::default(),
            renderer_inits: self.renderers,
            renderers: Vec::new(),
        }
    }
}

/// Scene-based game
pub struct SceneGame {
    registry: Registry<String>,
    debug_drawers: DebugDrawers,
    scene_dir: String,
    schedule: Schedule,
    world: World,
    resources: Resources,
    io_state: IoState,
    renderer_inits:
        Vec<Box<dyn Fn(&Device, &TextureFormat, [f32; 2]) -> Box<dyn MiddlewareRenderer>>>,
    renderers: Vec<Box<dyn MiddlewareRenderer>>,
}

impl SceneGame {
    pub fn builder() -> SceneGameBuilder {
        SceneGameBuilder::new()
    }

    pub fn push_debug_entity<T>(&mut self, components: T) -> Entity
    where
        Option<T>: IntoComponentSource,
    {
        self.world.push(components)
    }
}

impl Playable for SceneGame {
    fn update(&mut self, delta: Duration, debug_ctx: &mut DebugContext) {
        // TODO: move to editor game or something?
        self.debug_drawers.draw_world(&mut self.world, debug_ctx);

        self.resources.insert(Delta(delta));
        self.resources.insert(self.io_state.clone());

        self.schedule.execute(&mut self.world, &mut self.resources);
        self.io_state.update();
    }

    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow> {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => return Some(ControlFlow::Exit),
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                // TODO: hidpi support?
                self.io_state.mouse_position = [position.x as f32, position.y as f32]
            }
            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                let button_num = match button {
                    MouseButton::Left => 0,
                    MouseButton::Right => 1,
                    MouseButton::Middle => 2,
                    _ => panic!("invalid button {:?}", button),
                };

                let state_bool = match state {
                    ElementState::Pressed => true,
                    ElementState::Released => false,
                };

                self.io_state.mouse_buttons[button_num] = state_bool;
                self.io_state.events.push(MouseEvent::new(
                    self.io_state.mouse_position,
                    *button,
                    *state,
                ));
            }
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                ..
            } => {
                self.io_state
                    .set_key(input.scancode, input.state == ElementState::Pressed);
            }
            _ => (),
        };

        None
    }

    fn redraw(
        &mut self,
        device: &Device,
        encoder: &mut CommandEncoder,
        frame: &TextureView,
        staging_belt: &mut StagingBelt,
        _debug_ctx: &mut DebugContext,
    ) -> () {
        if self.renderers.len() == 0 {
            for init in &self.renderer_inits {
                self.renderers
                    .push(init(device, &TextureFormat::Bgra8UnormSrgb, [1920., 1080.]));
            }
        }

        for renderer in &mut self.renderers {
            renderer.prepare(&mut self.world, device, staging_belt, encoder);
        }

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[RenderPassColorAttachment {
                view: &frame,
                resolve_target: None,
                ops: Default::default(),
            }],
            depth_stencil_attachment: None,
        });

        for renderer in &mut self.renderers {
            renderer.render(&mut pass);
        }
    }
}
