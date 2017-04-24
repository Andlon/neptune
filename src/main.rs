#[macro_use]
extern crate glium;
extern crate cgmath;
extern crate time;

#[macro_use]
extern crate itertools;

extern crate alga;
extern crate nalgebra;
extern crate ncollide;

extern crate num;

#[cfg(test)]
#[macro_use]
extern crate approx;

extern crate ordered_float;

mod core;
mod entity;
mod engine;
mod render;
mod physics;
mod input_manager;
mod geometry;
mod message;
mod camera;
mod time_keeper;
mod interop;

use engine::Engine;

struct Initializer;

use entity::EntityBlueprint;
use entity::blueprints;
use camera::Camera;
use render::Color;
use engine::{SceneBlueprint, SceneInitializer};
use physics::RigidBody;

use cgmath::{Point3, Vector3, EuclideanSpace, Zero, Quaternion};
use geometry::{Sphere, Cuboid};

impl SceneInitializer for Initializer {
    fn create_scene(&self, index: usize) -> Option<SceneBlueprint> {
        match index {
            0 => Some(self.create_scene0()),
            1 => Some(self.create_scene1()),
            _ => None
        }
    }
}

struct SphereObject {
    center: Point3<f64>,
    velocity: Vector3<f64>,
    radius: f64,
    mass: f64,
    color: Color,
    subdivisions: u32
}

struct CuboidObject {
    center: Point3<f64>,
    orientation: Quaternion<f64>,
    velocity: Vector3<f64>,
    half_size: Vector3<f64>,
    mass: f64,
    color: Color,
}

fn main() {
    let mut engine = Engine::new(Initializer);
    engine.run();
}

impl Default for SphereObject {
    fn default() -> Self {
        let gray = Color::rgb(0.5, 0.5, 0.5);
        SphereObject {
            center: Point3::origin(),
            velocity: Vector3::zero(),
            radius: 1.0,
            mass: 1.0,
            color: gray,
            subdivisions: 3
        }
    }
}

impl SphereObject {
    fn center(mut self, center: Point3<f64>) -> SphereObject {
        self.center = center;
        self
    }

    fn velocity(mut self, velocity: Vector3<f64>) -> SphereObject {
        self.velocity = velocity;
        self
    }

    fn radius(mut self, radius: f64) -> SphereObject {
        self.radius = radius;
        self
    }

    fn mass(mut self, mass: f64) -> SphereObject {
        self.mass = mass;
        self
    }

    fn color(mut self, color: Color) -> SphereObject {
        self.color = color;
        self
    }

    fn subdivisions(mut self, subdivisions: u32) -> SphereObject {
        self.subdivisions = subdivisions;
        self
    }

    fn create_blueprint(self) -> EntityBlueprint {
        let sphere = Sphere {
            center: interop::cgmath_point3_to_nalgebra(&self.center),
            radius: self.radius
        };
        let mut blueprint = blueprints::sphere(sphere, self.mass, self.subdivisions);
        blueprint.renderable.as_mut().unwrap().color = self.color;

        if let &mut RigidBody::Dynamic(ref mut rb) = blueprint.rigid_body.as_mut().unwrap() {
            rb.state.velocity = interop::cgmath_vector3_to_nalgebra(&self.velocity);
            rb.prev_state.velocity = interop::cgmath_vector3_to_nalgebra(&self.velocity);
        }

        blueprint
    }
}

impl Default for CuboidObject {
    fn default() -> Self {
        let gray = Color::rgb(0.5, 0.5, 0.5);
        CuboidObject {
            center: Point3::origin(),
            velocity: Vector3::zero(),
            half_size: Vector3::new(0.5, 0.5, 0.5),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            mass: 1.0,
            color: gray,
        }
    }
}

impl CuboidObject {
    fn center(mut self, center: Point3<f64>) -> Self {
        self.center = center;
        self
    }

    fn velocity(mut self, velocity: Vector3<f64>) -> Self {
        self.velocity = velocity;
        self
    }

    fn half_size(mut self, half_size: Vector3<f64>) -> Self {
        self.half_size = half_size;
        self
    }

    fn orientation(mut self, orientation: Quaternion<f64>) -> Self {
        self.orientation = orientation;
        self
    }

    fn mass(mut self, mass: f64) -> Self {
        self.mass = mass;
        self
    }

    fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    fn create_blueprint(self) -> EntityBlueprint {
        let cuboid = Cuboid {
            center: interop::cgmath_point3_to_nalgebra(&self.center),
            half_size: interop::cgmath_vector3_to_nalgebra(&self.half_size),
            rotation: nalgebra::UnitQuaternion::new_normalize(
                interop::cgmath_quat_to_nalgebra(&self.orientation))
        };

        let mut blueprint = blueprints::cuboid(cuboid, self.mass);
        blueprint.renderable.as_mut().unwrap().color = self.color;

        if let &mut RigidBody::Dynamic(ref mut rb) = blueprint.rigid_body.as_mut().unwrap() {
            rb.state.velocity = interop::cgmath_vector3_to_nalgebra(&self.velocity);
            rb.prev_state.velocity = interop::cgmath_vector3_to_nalgebra(&self.velocity);
        }
        blueprint
    }
}

impl Initializer {
    fn create_scene0(&self) -> SceneBlueprint {
        use cgmath::{Quaternion};

        let camera = Camera::look_in(Point3::new(40.0, 0.0, 0.0), -Vector3::unit_x(), Vector3::unit_z())
                            .unwrap();

        let blue = Color::rgb(0.0, 0.0, 1.0);
        let red = Color::rgb(1.0, 0.0, 0.0);
        let green = Color::rgb(0.0, 1.0, 0.0);
        let graybrown = Color::rgb(205.0 / 255.0, 133.0 / 255.0 ,63.0/255.0);

        let blueprints = vec![
            SphereObject::default()
                         .radius(5.0)
                         .mass(1e11)
                         .color(blue)
                         .subdivisions(4)
                         .create_blueprint(),

            SphereObject::default()
                         .center(Point3::new(0.0, 15.0, 15.0))
                         .velocity(Vector3::new(0.0, 2.5, 0.0))
                         .radius(1.0)
                         .mass(1.0)
                         .color(graybrown)
                         .create_blueprint(),

            SphereObject::default()
                         .center(Point3::new(5.0, 15.0, 0.0))
                         .velocity(Vector3::new(0.0, 0.0, 1.5))
                         .radius(1.0)
                         .mass(1.0)
                         .color(red)
                         .create_blueprint(),

            SphereObject::default()
                         .center(Point3::new(0.0, 15.0, -5.0))
                         .velocity(Vector3::new(0.0, 1.0, 2.0))
                         .radius(1.0)
                         .mass(1.0)
                         .color(red)
                         .create_blueprint(),

            SphereObject::default()
                         .center(Point3::new(0.0, 15.0, 0.0))
                         .velocity(Vector3::new(0.0, -2.0, 0.0))
                         .radius(1.0)
                         .mass(1.0)
                         .color(red)
                         .create_blueprint(),

            CuboidObject::default()
                         .center(Point3::new(0.0, -40.0, 0.0))
                         .velocity(Vector3::zero())
                         .half_size(Vector3::new(5.0, 5.0, 10.0))
                         .orientation(Quaternion::new(1.0, 0.0, 0.0, 0.0))
                         .mass(0.2)
                         .color(green)
                         .create_blueprint()
        ];

        SceneBlueprint {
            blueprints: blueprints,
            camera: camera
        }
    }

    fn create_scene1(&self) -> SceneBlueprint {
        let camera = Camera::look_in(Point3::new(5.0, 0.0, 0.0), -Vector3::unit_x(), Vector3::unit_z())
                            .unwrap();

        let red = Color::rgb(1.0, 0.0, 0.0);
        let graybrown = Color::rgb(205.0 / 255.0, 133.0 / 255.0 ,63.0/255.0);

        use cgmath::{Rad, Quaternion, Rotation3};

        let blueprints = vec![
            CuboidObject::default()
                         .center(Point3::new(0.0, 0.0, -5.0))
                         .half_size(Vector3::new(5.0, 5.0, 5.0))
                         .mass(1e10)
                         .color(red)
                         .create_blueprint(),

            CuboidObject::default()
                         .center(Point3::new(2.5, 0.0, 2.0))
                         .velocity(Vector3::new(-0.1, 0.0, 0.0))
                         .mass(1.0)
                         .orientation(Quaternion::from_axis_angle(Vector3::unit_y() + Vector3::unit_x(), Rad(0.8)))
                         .color(graybrown)
                         .create_blueprint()
        ];

        SceneBlueprint {
            blueprints: blueprints,
            camera: camera
        }
    }
}
