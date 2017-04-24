use entity::Entity;
use std::collections::HashMap;
use geometry::{Sphere, Cuboid};

pub type CollisionComponentId = usize;

#[derive(Copy, Clone, Debug)]
pub enum CollisionModel {
    Sphere(Sphere<f64>),
    Cuboid(Cuboid<f64>)
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

    pub fn clear(&mut self) {
        self.models.clear();
        self.entity_map.clear();
        self.entities.clear();
    }
}
