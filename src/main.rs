#[macro_use]
extern crate glium;
#[macro_use]
extern crate cgmath;

mod entity;
mod engine;
mod render;
mod store;
mod input_manager;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
