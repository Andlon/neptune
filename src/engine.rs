use entity::{EntityManager, EntityBlueprint, Entity};
use render::*;
use physics::{PhysicsEngine, CollisionComponentStore,
    PhysicsComponentStore, CollisionEngine, ContactCollection};
use input_manager::InputManager;
use message::{Message, MessageReceiver};
use camera::{Camera, CameraController};
use time_keeper::TimeKeeper;
use std;

pub struct Engine {
    should_continue: bool
}

pub struct ComponentStores {
    pub scene: SceneRenderableStore,
    pub transform: SceneTransformStore,
    pub physics: PhysicsComponentStore,
    pub collision: CollisionComponentStore,
}

struct Systems {
    pub scene: SceneRenderer,
    pub input: InputManager,
    pub camera: CameraController,
    pub physics: PhysicsEngine,
    pub collision: CollisionEngine
}

impl ComponentStores {
    pub fn assemble_blueprint(&mut self, entity: Entity, blueprint: EntityBlueprint) {
        if let Some(physics) = blueprint.physics {
            self.physics.set_component_properties(entity, physics);
        }
        if let Some(collision) = blueprint.collision {
            self.collision.set_component_model(entity, collision);
        }
        if let Some(transform) = blueprint.transform {
            self.transform.set_transform(entity, transform);
        }
        if let Some(renderable) = blueprint.renderable {
            self.scene.set_renderable(entity, renderable);
        }
    }
}

impl Engine {

    pub fn new() -> Engine {
        Engine { should_continue: true }
    }

    pub fn run(&mut self) {
        let window = Window::new();

        const TIMESTEP: f64 = 0.02;

        let mut entity_manager = EntityManager::new();
        let mut stores = prepare_component_stores();
        let mut systems = prepare_systems(&window);
        let mut contacts = ContactCollection::new();
        let mut time_keeper = TimeKeeper::new();

        let camera = initialize_scene(&mut entity_manager, &mut stores);
        systems.camera.set_camera(camera);

        while self.should_continue {
            let frame_time = time_keeper.produce_frame();

            while time_keeper.consume(TIMESTEP) {
                systems.physics.simulate(TIMESTEP, &mut stores.physics);
                systems.collision.detect_collisions(&stores.physics, &stores.collision, &mut contacts);
                systems.collision.resolve_collisions(&mut stores.physics, &contacts);
            }

            let progress = time_keeper.accumulated() / TIMESTEP;
            interpolate_transforms(&mut stores.transform, &stores.physics, progress);

            let camera = systems.camera.update(frame_time);

            systems.scene.update_buffers(&window, &stores.scene);

            // Render
            let mut frame = window.begin_frame();
            systems.scene.render(&mut frame, camera, &stores.scene, &stores.transform);
            frame.finish();

            let messages = window.check_events();
            self.dispatch_messages(messages, &mut systems);
        }
    }

    fn dispatch_messages(&mut self, messages: Vec<Message>, systems: &mut Systems) {
        let mut messages = messages;
        let mut response = Vec::new();

        while !messages.is_empty() {
            response.clear();
            response.extend(systems.input.process_messages(&messages));
            response.extend(systems.camera.process_messages(&messages));
            response.extend(self.process_messages(&messages));

            std::mem::swap(&mut messages, &mut response);
        }
    }
}

impl MessageReceiver for Engine {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message> {
        for message in messages {
            match message.clone() {
                Message::WindowClosed => self.should_continue = false,
                _ => ()
            };
        }
        Vec::new()
    }
}

fn prepare_component_stores() -> ComponentStores {
    ComponentStores {
        scene: SceneRenderableStore::new(),
        transform: SceneTransformStore::new(),
        physics: PhysicsComponentStore::new(),
        collision: CollisionComponentStore::new()
    }
}

fn prepare_systems(window: &Window) -> Systems {
    use cgmath::{Point3, Vector3, EuclideanSpace};
    let default_camera = Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap();
    Systems {
        scene: SceneRenderer::new(window),
        input: InputManager::new(),
        camera: CameraController::from(default_camera),
        physics: PhysicsEngine::new(),
        collision: CollisionEngine::new()
    }
}

fn interpolate_transforms(transforms: &mut SceneTransformStore,
                          physics: &PhysicsComponentStore,
                          fraction: f64) {
    use cgmath::{Point3, EuclideanSpace, Quaternion};
    assert!(fraction >= 0.0 && fraction <= 1.0);

    for (&entity, &component) in physics.entity_component_pairs() {
        let interpolated_pos = {
            let prev_pos = physics.lookup_prev_position(&component).to_vec();
            let current_pos = physics.lookup_position(&component).to_vec();

            // We have to work around the fact that cgmath does not implement .cast()
            // for Point3<S>, but only for Vector3<S>.
            let vector_form = fraction * current_pos + (1.0 - fraction) * prev_pos;
            Point3::from_vec(vector_form.cast::<f32>())
        };

        let interpolated_orientation: Quaternion<f32> = {
            let prev_orient = physics.lookup_prev_orientation(&component);
            let current_orient = physics.lookup_orientation(&component);
            let orient = prev_orient.nlerp(current_orient, fraction);

            // This is very awkward, but there is no easy way to cast a
            // Quaternion<f64> to Quaternion<f32>
            Quaternion::from_sv(orient.s as f32, orient.v.cast())
        };

        let current_transform = match transforms.lookup(&entity) {
            Some(current) => current.clone(),
            None => SceneTransform::default()
        };

        let transform = SceneTransform {
            position: interpolated_pos,
            orientation: interpolated_orientation,
            .. current_transform
        };

        // Note: This implicitly adds a SceneTransform component to any
        // component which has a Physics component, which we deem
        // to be desirable behavior.
        transforms.set_transform(entity, transform);
    }
}

fn initialize_scene(entity_manager: &mut EntityManager, stores: &mut ComponentStores)
    -> Camera {
    use cgmath::{Point3, Vector3, EuclideanSpace, Quaternion};

    let blue = Color::rgb(0.0, 0.0, 1.0);
    let red = Color::rgb(1.0, 0.0, 0.0);
    let green = Color::rgb(0.0, 1.0, 0.0);
    let graybrown = Color::rgb(205.0 / 255.0, 133.0 / 255.0 ,63.0/255.0);

    use entity::blueprints;
    use geometry::{Sphere, Cuboid};

    {
        let sphere = Sphere {
            center: Point3::origin(),
            radius: 5.0
        };
        let mut blueprint = blueprints::sphere(sphere, 1e11, 4);
        blueprint.renderable.as_mut().unwrap().color = blue;
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }

    {
        let sphere = Sphere {
            center: Point3::new(0.0, 15.0, 15.0),
            radius: 1.0
        };
        let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
        blueprint.renderable.as_mut().unwrap().color = graybrown;
        blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 2.5, 0.0);
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }

    {
        let sphere = Sphere {
            center: Point3::new(5.0, 15.0, 0.0),
            radius: 1.0
        };
        let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
        blueprint.renderable.as_mut().unwrap().color = red;
        blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 0.0, 1.5);
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }

    {
        let sphere = Sphere {
            center: Point3::new(0.0, 15.0, -5.0),
            radius: 1.0
        };
        let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
        blueprint.renderable.as_mut().unwrap().color = red;
        blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, 1.0, 2.0);
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }

    {
        let sphere = Sphere {
            center: Point3::new(0.0, 15.0, 0.0),
            radius: 1.0
        };
        let mut blueprint = blueprints::sphere(sphere, 1.0, 3);
        blueprint.renderable.as_mut().unwrap().color = red;
        blueprint.physics.as_mut().unwrap().velocity = Vector3::new(0.0, -2.0, 0.0);
        stores.assemble_blueprint(entity_manager.create(), blueprint);
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
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }

    Camera::look_in(Point3::new(40.0, 0.0, 0.0), -Vector3::unit_x(), Vector3::unit_z()).unwrap()
}
