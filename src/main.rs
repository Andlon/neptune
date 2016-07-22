#[macro_use]
extern crate glium;

mod ecs;
mod engine;
mod scene_renderer;
mod value_types;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
