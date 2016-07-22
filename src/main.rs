#[macro_use]
extern crate glium;

mod entity;
mod engine;
mod render;
mod value_types;
mod store;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
