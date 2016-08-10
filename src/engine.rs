use entity::{Entity, EntityManager};
use render::*;
use physics::*;
use input_manager::InputManager;
use message::{Message, MessageReceiver};
use camera_controller::CameraController;
use time_keeper::TimeKeeper;
use time::*;

pub struct Engine {
    should_continue: bool
}

pub struct ComponentStores {
    pub scene: SceneRenderableStore,
    pub transform: SceneTransformStore,
    pub physics: PhysicsComponentStore
}

pub struct Systems {
    pub scene: SceneRenderer,
    pub input: InputManager,
    pub camera: CameraController,
    pub physics: PhysicsEngine
}

impl Engine {

    pub fn new() -> Engine {
        Engine { should_continue: true }
    }

    pub fn run(&mut self) {
        let window = Window::new();

        const TIMESTEP: f64 = 0.01;

        let mut entity_manager = EntityManager::new();
        let mut stores = prepare_component_stores();
        let mut systems = prepare_systems(&window);
        let mut time_keeper = TimeKeeper::new();

        initialize_scene(&window, &mut entity_manager, &mut stores);
        sync_transform_with_physics(&mut stores.transform, &stores.physics);

        let mut frame_timestamp = precise_time_s();

        while self.should_continue {
            let new_frame_timestamp = precise_time_s();
            let frame_time = new_frame_timestamp - frame_timestamp;
            frame_timestamp = new_frame_timestamp;
            time_keeper.produce(frame_time);

            while time_keeper.consume(TIMESTEP) {
                systems.physics.simulate(TIMESTEP, &mut stores.physics);
                sync_transform_with_physics(&mut stores.transform, &stores.physics);
            }

            let camera = systems.camera.update();

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
        physics: PhysicsComponentStore::new()
    }
}

fn prepare_systems(window: &Window) -> Systems {
    use cgmath::{Point3, Vector3, EuclideanSpace};
    let default_camera = Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap();
    Systems {
        scene: SceneRenderer::new(window),
        input: InputManager::new(),
        camera: CameraController::from(default_camera),
        physics: PhysicsEngine::new()
    }
}

fn sync_transform_with_physics(transforms: &mut SceneTransformStore, physics: &PhysicsComponentStore) {
    use cgmath::{Point3, EuclideanSpace};
    for (entity, ref mut transform) in transforms.transforms_mut() {
        if let Some(position) = physics.lookup_position(&entity) {
            // TODO: Implement .cast() for Point3 in cgmath?
            transform.position = Point3::from_vec(position.to_vec().cast::<f32>());
        }
    }
}

fn initialize_scene(window: &Window, entity_manager: &mut EntityManager, stores: &mut ComponentStores) {
    use cgmath::{Point3, Vector3};

    // Create a tetrahedron
    let (a, b, c, d) = (Point3::new(-0.5, 0.0, 0.0), Point3::new(0.5, 0.0, 0.0),
                        Point3::new(0.0, 0.5, 0.0), Point3::new(0.0, 0.25, 0.5));
    let triangle_entity = entity_manager.create();
    let triangle_renderable = tetrahedron_renderable(&window, a, b, c, d);
    let triangle_transform = SceneTransform {
        position: Point3 { x: 0.0, y: 5.0, z: 0.0 }
    };
    stores.scene.set_renderable(triangle_entity, triangle_renderable);
    stores.transform.set_transform(triangle_entity, triangle_transform);

    // Also create an icosahedron
    let ico_entity = entity_manager.create();
    let ico_renderable = icosahedron_renderable(&window);
    let ico_transform = SceneTransform {
        position: Point3 { x: 0.0, y: 15.0, z: 0.0 }
    };
    stores.scene.set_renderable(ico_entity, ico_renderable);
    stores.transform.set_transform(ico_entity, ico_transform);

    // And a unit sphere
    let sphere_entity = entity_manager.create();
    let sphere_renderable = unit_sphere_renderable(&window, 4);
    let sphere_transform = SceneTransform {
        position: Point3 { x: 0.0, y: 15.0, z: 5.0 }
    };
    stores.scene.set_renderable(sphere_entity, sphere_renderable);
    stores.transform.set_transform(sphere_entity, sphere_transform);
    stores.physics.set_component_properties(sphere_entity,
        Point3::new(0.0, 15.0, 5.0),
        Vector3::new(0.0, 0.0, 0.5),
        1.0);
}
