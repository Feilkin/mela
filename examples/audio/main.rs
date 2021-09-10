use futures::io::Error;
use mela::audio::audio_system;
use mela::audio::{Audio, AudioState};
use mela::components::Transform;
use mela::debug::DebugDrawable;
use mela::Application;

fn main() {
    let mut game = mela::SceneGame::builder()
        .with_system(audio_system())
        .register_addable_component::<Audio>()
        .build();

    game.push_debug_entity((Audio::new("examples/audio/"), Transform::default()));
    let app = Application::new(game, "Hello, World!");
    futures::executor::block_on(app.setup()).run();
}
