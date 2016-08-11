use cgmath::{Point3, Vector3, Zero};
use store::{Identifier, OneToOneStore};
use std::collections::HashMap;
use entity::Entity;
use itertools::Zip;

type PhysicsComponentId = usize;

pub struct PhysicsComponentStore {
    position: Vec<Point3<f64>>,
    velocity: Vec<Vector3<f64>>,
    acceleration: Vec<Vector3<f64>>,
    mass: Vec<f64>,

    // Used for interpolating the state between physics frames
    prev_position: Vec<Point3<f64>>,

    entity_map: HashMap<Entity, PhysicsComponentId>,
}

impl PhysicsComponentStore {

    pub fn new() -> Self {
        PhysicsComponentStore {
            position: Vec::new(),
            velocity: Vec::new(),
            acceleration: Vec::new(),
            mass: Vec::new(),
            prev_position: Vec::new(),
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

        // Note that we set acceleration to zero, because it will be
        // computed by the physics engine.

        let next_available_index = self.num_components();
        let index: usize = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index >= self.num_components() {
            self.position.push(position);
            self.velocity.push(velocity);
            self.acceleration.push(Vector3::zero());
            self.mass.push(mass);
            self.prev_position.push(position)
        } else {
            self.position[index] = position;
            self.velocity[index] = velocity;
            self.acceleration[index] = Vector3::zero();
            self.mass[index] = mass;
            // Setting prev position as well will avoid strange effects
            // as the object position is interpolated between physics frames
            self.prev_position[index] = position;
        }
        index
    }

    pub fn lookup_position(&self, entity: &Entity) -> Option<Point3<f64>> {
        self.entity_map.get(entity)
                       .map(|index| self.position[index.clone()])
    }

    pub fn lookup_prev_position(&self, entity: &Entity) -> Option<Point3<f64>> {
        self.entity_map.get(entity)
                       .map(|index| self.prev_position[index.clone()])
    }

    pub fn num_components(&self) -> usize {
        assert!(self.position.len() == self.velocity.len() && self.velocity.len() == self.mass.len());
        self.position.len()
    }

    pub fn prev_positions<'a>(&'a self) -> &'a [Point3<f64>] {
        self.prev_position.as_slice()
    }

    pub fn prev_positions_mut<'a>(&'a mut self) -> &'a mut [Point3<f64>] {
        self.prev_position.as_mut_slice()
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

    pub fn accelerations<'a>(&'a self) -> &'a [Vector3<f64>] {
        self.acceleration.as_slice()
    }

    pub fn accelerations_mut<'a>(&'a mut self) -> &'a mut [Vector3<f64>] {
        self.acceleration.as_mut_slice()
    }

    pub fn masses<'a>(&'a self) -> &'a [f64] {
        self.mass.as_slice()
    }
}