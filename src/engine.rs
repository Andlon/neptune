use entity::{Entity, EntityManager};
use render::*;
use physics::*;
use geometry::Sphere;
use input_manager::InputManager;
use message::{Message, MessageReceiver};
use camera::{Camera, CameraController};
use time_keeper::TimeKeeper;

pub struct Engine {
    should_continue: bool
}

pub struct ComponentStores {
    pub scene: SceneRenderableStore,
    pub transform: SceneTransformStore,
    pub physics: PhysicsComponentStore,
    pub collision: CollisionComponentStore,
}

pub struct Systems {
    pub scene: SceneRenderer,
    pub input: InputManager,
    pub camera: CameraController,
    pub physics: PhysicsEngine,
    pub collision: CollisionEngine
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

        initialize_scene(&window, &mut entity_manager, &mut stores);

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

        while !messages.is_empty() {
            let mut response = Vec::new();
            response.extend(systems.input.process_messages(&messages));
            response.extend(systems.camera.process_messages(&messages));

            for message in messages {
                match message {
                    Message::WindowClosed => self.should_continue = false,
                    _ => ()
                };
            }

            messages = response.clone();
        }
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
    use cgmath::{Point3, EuclideanSpace};
    assert!(fraction >= 0.0 && fraction <= 1.0);

    for (&entity, &component) in physics.entity_component_pairs() {
        let prev_pos = physics.lookup_prev_position(&component).to_vec();
        let current_pos = physics.lookup_position(&component).to_vec();

        let interpolated_pos = {
            // We have to work around the fact that cgmath does not implement .cast()
            // for Point3<S>, but only for Vector3<S>.
            let vector_form = fraction * current_pos + (1.0 - fraction) * prev_pos;
            Point3::from_vec(vector_form.cast::<f32>())
        };

        let transform = SceneTransform { position: interpolated_pos };

        // Note: This implicitly adds a SceneTransform component to any
        // component which has a Physics component, which we deem
        // to be desirable behavior.
        transforms.set_transform(entity, transform);
    }
}

fn initialize_scene(window: &Window, entity_manager: &mut EntityManager, stores: &mut ComponentStores) {
    use cgmath::{Point3, Vector3, EuclideanSpace};

    let blue = Color::rgb(0.0, 0.0, 1.0);
    let red = Color::rgb(1.0, 0.0, 0.0);
    let graybrown = Color::rgb(205.0 / 255.0, 133.0 / 255.0 ,63.0/255.0);

    {
        let sphere_entity = entity_manager.create();
        let sphere_position = Point3::new(0.0, 15.0, 0.0);
        let sphere_renderable = SceneRenderable { color: blue, .. unit_sphere_renderable(&window, 3) };
        let sphere_collision_model = CollisionModel::SphereModel { radius: 1.0 };
        stores.scene.set_renderable(sphere_entity, sphere_renderable);
        stores.physics.set_component_properties(sphere_entity,
            sphere_position,
            Vector3::new(0.0, 0.0, 0.0),
            1.0e11);
        stores.collision.set_component_model(sphere_entity, sphere_collision_model);
    }

    {
        // And a light unit sphere
        let sphere_entity = entity_manager.create();
        let sphere_position = Point3::new(0.0, 15.0, 5.0);
        let sphere_renderable = SceneRenderable{ color: graybrown, .. unit_sphere_renderable(&window, 3) };
        let sphere_collision_model = CollisionModel::SphereModel { radius: 1.0 };
        stores.scene.set_renderable(sphere_entity, sphere_renderable);
        stores.physics.set_component_properties(sphere_entity,
            sphere_position,
            Vector3::new(0.0, 2.5, 0.0),
            1.0);
        stores.collision.set_component_model(sphere_entity, sphere_collision_model);
    }

    {
        // And one more that collides with the heavy sphere
        let sphere_entity = entity_manager.create();
        let sphere_position = Point3::new(5.0, 15.0, 0.0);
        let sphere_renderable = SceneRenderable { color: red, .. unit_sphere_renderable(&window, 3) };
        let sphere_collision_model = CollisionModel::SphereModel { radius: 1.0 };
        stores.scene.set_renderable(sphere_entity, sphere_renderable);
        stores.physics.set_component_properties(sphere_entity,
            sphere_position,
            Vector3::new(0.0, 0.0, 1.5),
            1.0);
        stores.collision.set_component_model(sphere_entity, sphere_collision_model);
    }

    {
        // And one more that collides with the the other spheres
        let sphere_entity = entity_manager.create();
        let sphere_position = Point3::new(0.0, 15.0, -5.0);
        let sphere_renderable = unit_sphere_renderable(&window, 3);
        let sphere_collision_model = CollisionModel::SphereModel { radius: 1.0 };
        stores.scene.set_renderable(sphere_entity, sphere_renderable);
        stores.physics.set_component_properties(sphere_entity,
            sphere_position,
            Vector3::new(0.0, 1.0, 2.0),
            1.0);
        stores.collision.set_component_model(sphere_entity, sphere_collision_model);
    }

}
