use cgmath::{Vector3, Quaternion};
use entity::Entity;
use std::collections::HashMap;

pub type CollisionComponentId = usize;

#[derive(Copy, Clone, Debug)]
pub struct SphereCollisionModel {
    pub radius: f64
}

#[derive(Copy, Clone, Debug)]
pub struct CuboidCollisionModel {
    pub half_size: Vector3<f64>,

    /// Rotation in the model coordinate frame, before world rotation is applied
    pub rotation: Quaternion<f64>
}


#[derive(Copy, Clone, Debug)]
pub enum CollisionModel {
    Sphere(SphereCollisionModel),
    Cuboid(CuboidCollisionModel)
}

impl CollisionModel {
    pub fn sphere(radius: f64) -> Self {
        CollisionModel::Sphere(SphereCollisionModel { radius: radius })
    }

    pub fn cuboid(half_size: Vector3<f64>, rotation: Quaternion<f64>) -> Self {
        CollisionModel::Cuboid(CuboidCollisionModel { half_size: half_size, rotation: rotation })
    }
}

pub struct CollisionComponentStore {
    models: Vec<CollisionModel>,
    entities: Vec<Entity>,

    entity_map: HashMap<Entity, CollisionComponentId>,
}

impl CollisionComponentStore {
    pub fn new() -> CollisionComponentStore {
        CollisionComponentStore {
            models: Vec::new(),
            entities: Vec::new(),
            entity_map: HashMap::new(),
        }
    }

    pub fn set_component_model(&mut self, entity: Entity, model: CollisionModel) -> CollisionComponentId {
        assert!(self.models.len() == self.entities.len());

        let next_available_index = self.num_components();
        let index = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index == next_available_index {
            self.models.push(model);
            self.entities.push(entity);
        } else {
            self.models[index] = model;
            self.entities[index] = entity;
        }
        index
    }

    pub fn num_components(&self) -> usize {
        assert!(self.models.len() == self.entities.len());
        self.models.len()
    }

    pub fn models<'a>(&'a self) -> &'a [CollisionModel] {
        self.models.as_slice()
    }

    pub fn entities<'a>(&'a self) -> &'a [Entity] {
        self.entities.as_slice()
    }
}