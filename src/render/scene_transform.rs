use store::OneToOneStore;
use entity::Entity;

use cgmath::{Point3, Vector3, Matrix4, EuclideanSpace, Quaternion};

#[derive(Copy, Clone, Debug)]
pub struct SceneTransform {
    pub position: Point3<f32>,
    pub scale: Vector3<f32>,
    pub orientation: Quaternion<f32>
}

impl Default for SceneTransform {
    fn default() -> Self {
        SceneTransform {
            position: Point3::origin(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        }
    }
}

impl SceneTransform {
    pub fn model_matrix(&self) -> Matrix4<f32> {
        // This is a very expensive way to do it, but it's easy and straightforward.
        // Fix when it becomes a bottleneck.
        let translate = Matrix4::from_translation(self.position.to_vec());
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        let rot = Matrix4::from(self.orientation);
        translate * rot * scale
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
}
