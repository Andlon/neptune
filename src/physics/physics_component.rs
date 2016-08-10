use cgmath::{Point3, Vector3};
use store::{Identifier, OneToOneStore};
use std::collections::HashMap;
use entity::Entity;

type PhysicsComponentId = usize;

pub struct PhysicsComponentStore {
    position: Vec<Point3<f64>>,
    velocity: Vec<Vector3<f64>>,
    mass: Vec<f64>,

    entity_map: HashMap<Entity, PhysicsComponentId>,
}

impl PhysicsComponentStore {

    pub fn new() -> Self {
        PhysicsComponentStore {
            position: Vec::new(),
            velocity: Vec::new(),
            mass: Vec::new(),
            entity_map: HashMap::new()
        }
    }

    pub fn set_component_properties(&mut self,
        entity: Entity,
        position: Point3<f64>,
        velocity: Vector3<f64>,
        mass: f64) -> PhysicsComponentId
    {
        assert!(mass >= 0.0, "Mass must be non-negative.");

        let next_available_index = self.num_components();
        let index: usize = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index >= self.num_components() {
            self.position.push(position);
            self.velocity.push(velocity);
            self.mass.push(mass);
        } else {
            self.position[index] = position;
            self.velocity[index] = velocity;
            self.mass[index] = mass;
        }
        index
    }

    pub fn lookup_position(&self, entity: &Entity) -> Option<Point3<f64>> {
        self.entity_map.get(entity)
                       .map(|index| self.position[index.clone()])
    }

    pub fn num_components(&self) -> usize {
        assert!(self.position.len() == self.velocity.len() && self.velocity.len() == self.mass.len());
        self.position.len()
    }

    pub fn positions<'a>(&'a self) -> &'a [Point3<f64>] {
        self.position.as_slice()
    }

    pub fn positions_mut<'a>(&'a mut self) -> &'a mut [Point3<f64>] {
        self.position.as_mut_slice()
    }

    pub fn velocities<'a>(&'a self) -> &'a [Vector3<f64>] {
        self.velocity.as_slice()
    }

    pub fn velocities_mut<'a>(&'a mut self) -> &'a mut [Vector3<f64>] {
        self.velocity.as_mut_slice()
    }

    pub fn masses<'a>(&'a self) -> &'a [f64] {
        self.mass.as_slice()
    }
}