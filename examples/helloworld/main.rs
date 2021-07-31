//! Hello, World! example mela game
//!

use mela::Application;

fn main() {
    let game = mela::SceneGame::builder().build();

    let app = Application::new(game, "Hello, World!");

    futures::executor::block_on(app.setup()).run();
}