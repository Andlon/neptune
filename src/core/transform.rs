use entity::Entity;
use cgmath::{Point3, Vector3, Matrix4, EuclideanSpace, Quaternion, InnerSpace};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Point3<f64>,
    pub scale: Vector3<f64>,
    pub orientation: Quaternion<f64>
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: Point3::origin(),
            scale: Vector3::new(1.0, 1.0, 1.0),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        }
    }
}

impl Transform {
    pub fn model_matrix(&self) -> Matrix4<f64> {
        // This is a very expensive way to do it, but it's easy and straightforward.
        // Fix when it becomes a bottleneck.
        let translate = Matrix4::from_translation(self.position.to_vec());
        let scale = Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z);
        let rot = Matrix4::from(self.orientation);
        translate * rot * scale
    }
}

pub struct TransformPair {
    pub prev: Transform,
    pub current: Transform
}

pub struct TransformStore {
    // Stores (previous, current) transforms
    transforms: HashMap<Entity, TransformPair>,
}

impl TransformStore {
    pub fn new() -> TransformStore {
        TransformStore {
            transforms: HashMap::new()
        }
    }

    pub fn set_transform(&mut self, entity: Entity, transforms: TransformPair) {
        self.transforms.insert(entity, transforms);;
    }

    /// Returns previous and current transform.
    pub fn lookup(&self, entity: &Entity) -> Option<&TransformPair> {
        self.transforms.get(entity)
    }

    pub fn lookup_mut(&mut self, entity: &Entity) -> Option<&mut TransformPair> {
        self.transforms.get_mut(entity)
    }

    /// Clears all transforms from the store.
    pub fn clear(&mut self) {
        self.transforms.clear();
    }
}

impl TransformPair {
    pub fn interpolate(&self, progress: f64) -> Transform {
        let interpolated_pos = Point3::from_vec(self.prev.position.to_vec().lerp(self.current.position.to_vec(), progress));
        let interpolated_orientation = self.prev.orientation.nlerp(self.current.orientation, progress);
        let interpolated_scale = self.prev.scale.lerp(self.current.scale, progress);
        Transform {
            position: interpolated_pos,
            orientation: interpolated_orientation,
            scale: interpolated_scale
        }
    } 
}
