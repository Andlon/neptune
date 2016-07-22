use value_types::{Vec3, Quaternion};
use store::OneToOneStore;
use entity::Entity;
use std::collections::HashMap;

pub struct SceneTransform {
    pub position: Vec3<u32>

    // TODO: Support rotation
}

pub struct SceneTransformStore {
    store: OneToOneStore<SceneTransform>,
}

impl SceneTransformStore {
    pub fn new() -> SceneTransformStore {
        SceneTransformStore {
            store: OneToOneStore::new()
        }
    }

    pub fn set_transform(&mut self, entity: Entity, transform: SceneTransform) {
        self.store.set_component(entity, transform);
    }

    pub fn renderables(&self) -> &HashMap<Entity, SceneTransform> {
        &self.store.components
    }
}
