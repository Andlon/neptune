#[macro_use]
extern crate glium;
#[macro_use]
extern crate cgmath;
extern crate time;
#[macro_use]
extern crate itertools;

mod entity;
mod engine;
mod render;
mod physics;
mod store;
mod input_manager;
mod geometry;
mod message;
mod camera;
mod time_keeper;

use engine::Engine;

fn main() {
    let mut engine = Engine::new();
    engine.run();
}
