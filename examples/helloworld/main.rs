//! Hello, World! example mela game
//!

use crate::sdf::{SdfObject, SdfShape};
use mela::components::Transform;
use mela::{Application, Delta};
use nalgebra::Isometry3;

mod sdf;

#[mela::ecs::system(for_each)]
fn move_ball(pos: &mut Transform, #[resource] delta: &Delta) {
    pos.0.translation.x += (rand::random::<f32>() - 0.5) * delta.as_secs_f32() * 100.;
    pos.0.translation.y += (rand::random::<f32>() - 0.5) * delta.as_secs_f32() * 100.;

    if pos.0.translation.x > 200. {
        pos.0.translation.x = 56.;
    }
    if pos.0.translation.x < 56. {
        pos.0.translation.x = 200.;
    }
    if pos.0.translation.y > 200. {
        pos.0.translation.y = 56.;
    }
    if pos.0.translation.y < 56. {
        pos.0.translation.y = 200.;
    }
}

fn main() {
    puffin::set_scopes_on(true);
    env_logger::init();
    let mut game = mela::SceneGame::builder()
        .with_system(move_ball_system())
        .with_renderer::<sdf::renderer::SdfRenderer>()
        .build();

    game.push_debug_entity((
        Transform(Isometry3::translation(64., 80., 32.)),
        SdfObject {
            shape: SdfShape::Ball(15.),
        },
    ));

    game.push_debug_entity((
        Transform(Isometry3::translation(128., 128., 32.)),
        SdfObject {
            shape: SdfShape::Ball(15.),
        },
    ));

    game.push_debug_entity((
        Transform(Isometry3::translation(73., 60., 50.)),
        SdfObject {
            shape: SdfShape::Ball(6.),
        },
    ));

    let app = Application::new(game, "Hello, World!");

    futures::executor::block_on(app.setup()).run();
}
