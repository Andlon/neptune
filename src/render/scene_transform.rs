use store::OneToOneStore;
use entity::Entity;
use std::collections::HashMap;

use cgmath::{Point3, Vector3, Matrix4, EuclideanSpace};

#[derive(Copy, Clone, Debug)]
pub struct SceneTransform {
    pub position: Point3<f32>,
    pub scale: Vector3<f32>

    // TODO: Support rotation
}

impl Default for SceneTransform {
    fn default() -> Self {
        SceneTransform {
            position: Point3::origin(),
            scale: Vector3::new(1.0, 1.0, 1.0)
        }
    }
}

impl SceneTransform {
    pub fn model_matrix(&self) -> Matrix4<f32> {
        let pos = &self.position;
        let scale = &self.scale;
        Matrix4::from([
            [scale.x, 0.0,     0.0,     0.0],
            [0.0,     scale.y, 0.0,     0.0],
            [0.0,     0.0,     scale.z, 0.0],
            [pos.x,   pos.y,   pos.z,   1.0]
        ])
    }
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

    pub fn lookup(&self, entity: &Entity) -> Option<&SceneTransform> {
        self.store.lookup(entity)
    }

    pub fn transforms(&self) -> &HashMap<Entity, SceneTransform> {
        &self.store.components
    }

    pub fn transforms_mut(&mut self) -> &mut HashMap<Entity, SceneTransform> {
        &mut self.store.components
    }
}
