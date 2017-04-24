use entity::{EntityManager, EntityBlueprint, Entity, LinearComponentStorage};
use render::*;
use physics::{PhysicsEngine, CollisionComponentStore,
    CollisionEngine, ContactCollection, RigidBody};
use input_manager::InputManager;
use message::{Message, MessageReceiver};
use camera::{Camera, CameraController};
use time_keeper::TimeKeeper;
use core::{Transform, TransformPair, TransformStore};
use std;
use interop;

pub struct Engine<Initializer: SceneInitializer> {
    initializer: Initializer,
    should_continue: bool,
    systems: Systems,
    stores: ComponentStores,
    entity_manager: EntityManager
}

struct ComponentStores {
    pub scene: SceneRenderableStore,
    pub transform: TransformStore,
    pub rigid_bodies: LinearComponentStorage<RigidBody>,
    pub collision: CollisionComponentStore,
    pub camera: Camera
}

struct Systems {
    pub scene: SceneRenderer,
    pub input: InputManager,
    pub camera: CameraController,
    pub physics: PhysicsEngine,
    pub collision: CollisionEngine
}

impl Systems {
    pub fn new() -> Self {
        Systems {
            scene: SceneRenderer::new(),
            input: InputManager::new(),
            camera: CameraController::new(),
            physics: PhysicsEngine::new(),
            collision: CollisionEngine::new()
        }
    }
}

impl ComponentStores {
    pub fn assemble_blueprint(&mut self, entity: Entity, blueprint: EntityBlueprint) {
        if let Some(rb) = blueprint.rigid_body {
            self.rigid_bodies.set_component_for_entity(entity, rb);
        }
        if let Some(collision) = blueprint.collision {
            self.collision.set_component_model(entity, collision);
        }
        if let Some(transform) = blueprint.transform {
            self.transform.set_transform(entity, TransformPair {
                current: transform,
                prev: transform
            });
        }
        if let Some(renderable) = blueprint.renderable {
            self.scene.set_renderable(entity, renderable);
        }
    }

    pub fn clear(&mut self) {
        self.scene.clear();
        self.transform.clear();
        self.rigid_bodies.clear();
        self.collision.clear();
    }
}

pub struct SceneBlueprint {
    pub blueprints: Vec<EntityBlueprint>,
    pub camera: Camera
}

pub trait SceneInitializer {
    fn create_scene(&self, index: usize) -> Option<SceneBlueprint>;
}

impl<I> Engine<I> where I: SceneInitializer {

    pub fn new(initializer: I) -> Engine<I> {
        Engine {
            initializer: initializer,
            should_continue: true,
            systems: Systems::new(),
            stores: prepare_component_stores(),
            entity_manager: EntityManager::new()
        }
    }

    pub fn run(&mut self) {
        let window = Window::new();

        const TIMESTEP: f64 = 0.02;

        let mut contacts = ContactCollection::new();
        let mut time_keeper = TimeKeeper::new();

        self.systems.scene.compile_shaders(&window);

        let scene = self.initializer.create_scene(0).expect("Initializer must provide scene 0!");
        reassemble_scene(&mut self.entity_manager, &mut self.stores, scene);

        while self.should_continue {
            let frame_time = time_keeper.produce_frame();

            while time_keeper.consume(TIMESTEP) {
                self.systems.physics.simulate(TIMESTEP, &mut self.stores.rigid_bodies);
                self.systems.collision.detect_collisions(&self.stores.rigid_bodies,
                                                         &self.stores.collision,
                                                         &mut contacts);
                self.systems.collision.resolve_collisions(&mut self.stores.rigid_bodies, &contacts);
                sync_transforms(&self.stores.rigid_bodies, &mut self.stores.transform);
            }

            let progress = time_keeper.accumulated() / TIMESTEP;

            self.stores.camera = self.systems.camera.update(self.stores.camera, frame_time);
            self.systems.scene.update_buffers(&window, &self.stores.scene);

            // Render
            let mut frame = window.begin_frame();
            self.systems.scene.render(&mut frame, progress, self.stores.camera.clone(), &self.stores.scene, &self.stores.transform);
            frame.finish();

            let messages = window.check_events();
            self.dispatch_messages(messages);
        }
    }

    fn dispatch_messages(&mut self, messages: Vec<Message>) {
        let mut messages = messages;
        let mut response = Vec::new();

        while !messages.is_empty() {
            response.clear();
            response.extend(self.systems.input.process_messages(&messages));
            response.extend(self.systems.camera.process_messages(&messages));
            response.extend(self.process_messages(&messages));

            std::mem::swap(&mut messages, &mut response);
        }
    }
}

impl<I> MessageReceiver for Engine<I> where I: SceneInitializer {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message> {
        for message in messages {
            match message.clone() {
                Message::WindowClosed => self.should_continue = false,
                Message::ReloadScene { index } => {
                    let new_scene = self.initializer.create_scene(index);
                    if let Some(new_scene) = new_scene {
                        self.stores.camera = new_scene.camera;
                        reassemble_scene(&mut self.entity_manager, &mut self.stores, new_scene);
                    }
                }
                _ => ()
            };
        }
        Vec::new()
    }
}

fn prepare_component_stores() -> ComponentStores {
    use cgmath::{Point3, Vector3, EuclideanSpace};
    ComponentStores {
        scene: SceneRenderableStore::new(),
        transform: TransformStore::new(),
        rigid_bodies: LinearComponentStorage::new(),
        collision: CollisionComponentStore::new(),
        camera: Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap()
    }
}

fn reassemble_scene(entity_manager: &mut EntityManager,
                    stores: &mut ComponentStores,
                    scene: SceneBlueprint) {
    stores.camera = scene.camera;

    stores.clear();
    for blueprint in scene.blueprints {
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }
}

fn sync_transforms(bodies: &LinearComponentStorage<RigidBody>,
                   transforms: &mut TransformStore)
{
    // For now, we require every physics object to also have a transform.
    // In the future we should remove the concept of Transform altogether,
    // and instead just let physics objects have discretized positions,
    // while SceneRenderables have interpolated positions, with
    // no common notion of Transform
    for &(ref rb, entity) in bodies.components() {
        let old_pair = transforms.lookup(&entity)
                                 .cloned()
                                 .unwrap_or_default();
        let new_pair = TransformPair {
            prev: Transform {
                position: interop::nalgebra_point3_to_cgmath(&rb.prev_state.position),
                orientation: interop::nalgebra_unit_quat_to_cgmath(&rb.prev_state.orientation),
                .. old_pair.prev
            },
            current: Transform {
                position: interop::nalgebra_point3_to_cgmath(&rb.state.position),
                orientation: interop::nalgebra_unit_quat_to_cgmath(&rb.state.orientation),
                .. old_pair.current
            }
        };
        transforms.set_transform(entity, new_pair);
    }
}
