#[macro_use]
extern crate glium;
#[macro_use]
extern crate cgmath;
extern crate time;
#[macro_use]
extern crate itertools;

#[macro_use]
extern crate approx;

mod entity;
mod engine;
mod render;
mod physics;
mod input_manager;
mod geometry;
mod message;
mod camera;
mod time_keeper;

use engine::Engine;

struct Initializer;

use entity::blueprints;
use camera::Camera;
use render::Color;
use engine::{SceneBlueprint, SceneInitializer};

impl engine::SceneInitializer for Initializer {
    fn create_scene(&self, index: usize) -> Option<SceneBlueprint> {
        match index {
            0 => Some(self.create_scene0()),
            _ => None
        }
    }
}

impl Initializer {
    fn create_scene0(&self) -> SceneBlueprint {
        use cgmath::{Point3, Vector3, Quaternion, EuclideanSpace};
        use geometry::{Sphere, Cuboid};

        let camera = Camera::look_in(Point3::new(40.0, 0.0, 0.0), -Vector3::unit_x(), Vector3::unit_z())
                            .unwrap();

        let mut blueprints = Vec::new();

        let blue = Color::rgb(0.0, 0.0, 1.0);
        let red = Color::rgb(1.0, 0.0, 0.0);
        let green = Color::rgb(0.0, 1.0, 0.0);
        let graybrown = Color::rgb(205.0 / 255.0, 133.0 / 255.0 ,63.0/255.0);

        {
            let sphere = Sphere {
                center: Point3::origin(),
                radius: 5.0
            };
            let mut blueprint = blueprints::sphere(sphere, 1e11, 4);
            blueprint.renderable.as_mut().unwrap().color = blue;
            blueprints.push(blueprint);
        }

        {
            let sphere = Sphere {
                center: Point3::new(0.0, 15.0, 15.0),
                radius: 1.0
            };
            let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
            blueprint.renderable.as_mut().unwrap().color = graybrown;
            blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 2.5, 0.0);
            blueprints.push(blueprint);
        }

        {
            let sphere = Sphere {
                center: Point3::new(5.0, 15.0, 0.0),
                radius: 1.0
            };
            let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
            blueprint.renderable.as_mut().unwrap().color = red;
            blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 0.0, 1.5);
            blueprints.push(blueprint);
        }

        {
            let sphere = Sphere {
                center: Point3::new(0.0, 15.0, -5.0),
                radius: 1.0
            };
            let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
            blueprint.renderable.as_mut().unwrap().color = red;
            blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 1.0, 2.0);
            blueprints.push(blueprint);
        }

        {
            let sphere = Sphere {
                center: Point3::new(0.0, 15.0, 0.0),
                radius: 1.0
            };
            let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
            blueprint.renderable.as_mut().unwrap().color = red;
            blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, -2.0, 0.0);
            blueprints.push(blueprint);
        }

        {
            let cuboid = Cuboid {
                center: Point3::new(0.0, -40.0, 0.0),
                half_size: Vector3::new(5.0, 5.0, 10.0),
                rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
            };

            let mut blueprint = blueprints::cuboid(cuboid, 0.2);
            blueprint.renderable.as_mut().unwrap().color = green;
            blueprint.physics.as_mut().unwrap().position = Point3::new(0.0, -40.0, 0.0);
            blueprints.push(blueprint);
        }

        SceneBlueprint {
            blueprints: blueprints,
            camera: camera
        }
    }
}



fn main() {
    let initializer = Initializer;
    let mut engine = Engine::new();
    engine.run(&initializer);
}
