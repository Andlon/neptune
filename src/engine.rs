use entity::{EntityManager, EntityBlueprint, Entity};
use render::*;
use physics::{PhysicsEngine, CollisionComponentStore,
    PhysicsComponentStore, CollisionEngine, ContactCollection};
use input_manager::InputManager;
use message::{Message, MessageReceiver};
use camera::{Camera, CameraController};
use time_keeper::TimeKeeper;
use std;

pub struct Engine<Initializer: SceneInitializer> {
    initializer: Initializer,
    should_continue: bool,
    systems: Systems,
    stores: ComponentStores,
    entity_manager: EntityManager
}

struct ComponentStores {
    pub scene: SceneRenderableStore,
    pub transform: SceneTransformStore,
    pub physics: PhysicsComponentStore,
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

    pub fn clear(&mut self) {
        self.scene.clear();
        self.transform.clear();
        self.physics.clear();
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
                self.systems.physics.simulate(TIMESTEP, &mut self.stores.physics);
                self.systems.collision.detect_collisions(&self.stores.physics, &self.stores.collision, &mut contacts);
                self.systems.collision.resolve_collisions(&mut self.stores.physics, &contacts);
            }

            let progress = time_keeper.accumulated() / TIMESTEP;
            interpolate_transforms(&mut self.stores.transform, &self.stores.physics, progress);

            self.stores.camera = self.systems.camera.update(self.stores.camera, frame_time);

            self.systems.scene.update_buffers(&window, &self.stores.scene);

            // Render
            let mut frame = window.begin_frame();
            self.systems.scene.render(&mut frame, self.stores.camera.clone(), &self.stores.scene, &self.stores.transform);
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
        transform: SceneTransformStore::new(),
        physics: PhysicsComponentStore::new(),
        collision: CollisionComponentStore::new(),
        camera: Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap()
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

fn reassemble_scene(entity_manager: &mut EntityManager,
                    stores: &mut ComponentStores,
                    scene: SceneBlueprint) {
    stores.camera = scene.camera;

    stores.clear();
    for blueprint in scene.blueprints {
        stores.assemble_blueprint(entity_manager.create(), blueprint);
    }
}
