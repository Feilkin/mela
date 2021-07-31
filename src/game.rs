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

use crate::debug::DebugContext;
use crate::ecs::query::Any;
use crate::ecs::serialize::Canon;
use crate::ecs::systems::{Builder as ScheduleBuilder, Runnable};
use crate::ecs::{Registry, Resources, Schedule, World};
use crate::winit::event::{ElementState, MouseButton, WindowEvent};
use crate::Delta;

pub trait Playable: Sized {
    /// Advances this game to next state
    fn update(&mut self, delta: Duration, debug_ctx: &mut DebugContext);

    /// Handle window events
    fn push_event<T>(&mut self, event: &Event<T>) -> Option<ControlFlow>;

    /// Renders this game
    fn redraw(&self, debug_ctx: &mut DebugContext) -> ();
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
    scene_dir: String,
    first_scene: String,
    schedule: ScheduleBuilder,
}

impl SceneGameBuilder {
    pub fn new() -> SceneGameBuilder {
        let mut registry = Registry::default();
        registry.register::<nalgebra::Isometry2<f32>>("isometry2".to_string());

        SceneGameBuilder {
            registry,
            scene_dir: "./scenes".to_string(),
            first_scene: "splash.json".to_string(),
            schedule: Schedule::builder(),
        }
    }

    pub fn build(mut self) -> SceneGame {
        // load first scene
        let mut world = load_scene(
            Path::new(&self.scene_dir).join(&self.first_scene),
            &self.registry,
        )
        .unwrap_or_default();

        world.push((crate::nalgebra::Isometry2::new(
            nalgebra::Vector2::new(100f32, 200.),
            0.,
        ),));

        save_scene(
            &world,
            std::env::current_dir()
                .unwrap()
                .join("examples/helloworld/scenes/splash.json"),
            &mut self.registry,
        )
        .unwrap();

        SceneGame {
            registry: self.registry,
            scene_dir: self.scene_dir,
            schedule: self.schedule.build(),
            world,
            resources: Default::default(),
            io_state: Default::default(),
        }
    }
}

/// Scene-based game
pub struct SceneGame {
    registry: Registry<String>,
    scene_dir: String,
    schedule: Schedule,
    world: World,
    resources: Resources,
    io_state: IoState,
}

impl SceneGame {
    pub fn builder() -> SceneGameBuilder {
        SceneGameBuilder::new()
    }
}

impl Playable for SceneGame {
    fn update(&mut self, delta: Duration, debug_ctx: &mut DebugContext) {
        self.resources.insert(Delta(delta));
        self.resources.insert(self.io_state.clone());

        self.schedule.execute(&mut self.world, &mut self.resources)
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
            _ => (),
        };

        None
    }

    fn redraw(&self, _debug_ctx: &mut DebugContext) -> () {}
}
