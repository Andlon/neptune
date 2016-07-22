#[macro_use]
extern crate glium;

mod entity;
mod engine;
mod render;
mod value_types;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
