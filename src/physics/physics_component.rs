use cgmath::{Point3, Vector3, Zero};
use store::{Identifier, OneToOneStore};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use entity::Entity;
use itertools::Zip;

pub type PhysicsComponentId = usize;

pub struct PhysicsComponentsView<'a> {
    pub position: &'a [Point3<f64>],
    pub velocity: &'a [Vector3<f64>],
    pub acceleration: &'a [Vector3<f64>],
    pub mass: &'a [f64],

    pub prev_position: &'a [Point3<f64>],
    pub prev_acceleration: &'a [Vector3<f64>]
}

pub struct MutablePhysicsComponentsView<'a> {
    pub position: &'a mut [Point3<f64>],
    pub velocity: &'a mut [Vector3<f64>],
    pub acceleration: &'a mut [Vector3<f64>],
    pub mass: &'a mut [f64],

    pub prev_position: &'a mut [Point3<f64>],
    pub prev_acceleration: &'a mut [Vector3<f64>]
}

pub struct PhysicsComponentStore {
    position: Vec<Point3<f64>>,
    velocity: Vec<Vector3<f64>>,
    acceleration: Vec<Vector3<f64>>,
    mass: Vec<f64>,

    // Used for interpolating the state between physics frames
    prev_position: Vec<Point3<f64>>,
    prev_acceleration: Vec<Vector3<f64>>,

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
            prev_acceleration: Vec::new(),
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
            self.prev_position.push(position);
            self.prev_acceleration.push(Vector3::zero());
        } else {
            self.position[index] = position;
            self.velocity[index] = velocity;
            self.acceleration[index] = Vector3::zero();
            self.mass[index] = mass;
            // Setting prev position as well will avoid strange effects
            // as the object position is interpolated between physics frames
            self.prev_position[index] = position;
            self.prev_acceleration[index] = Vector3::zero();
        }
        index
    }

    // When 'impl Trait' lands in stable, we can return something like impl Iterator instead
    pub fn entity_component_pairs<'a>(&'a self) -> Iter<'a, Entity, PhysicsComponentId> {
        self.entity_map.iter()
    }

    pub fn lookup_component(&self, entity: &Entity) -> Option<PhysicsComponentId> {
        self.entity_map.get(entity).map(|x| x.to_owned())
    }

    pub fn lookup_position(&self, component: &PhysicsComponentId) -> Point3<f64> {
        self.position[component.to_owned()]
    }

    pub fn lookup_prev_position(&self, component: &PhysicsComponentId) -> Point3<f64> {
        self.prev_position[component.to_owned()]
    }

    pub fn num_components(&self) -> usize {
        assert!(self.position.len() == self.velocity.len() && self.velocity.len() == self.mass.len());
        self.position.len()
    }

    pub fn swap_buffers(&mut self) {
        use std::mem::swap;
        swap(&mut self.position, &mut self.prev_position);
        swap(&mut self.acceleration, &mut self.prev_acceleration);
    }

    pub fn view<'a>(&'a self) -> PhysicsComponentsView<'a> {
        PhysicsComponentsView {
            position: &self.position,
            velocity: &self.velocity,
            acceleration: &self.acceleration,
            mass: &self.mass,
            prev_position: &self.prev_position,
            prev_acceleration: &self.prev_acceleration
        }
    }

    pub fn mutable_view<'a>(&'a mut self) -> MutablePhysicsComponentsView<'a> {
        MutablePhysicsComponentsView {
            position: &mut self.position,
            velocity: &mut self.velocity,
            acceleration: &mut self.acceleration,
            mass: &mut self.mass,
            prev_position: &mut self.prev_position,
            prev_acceleration: &mut self.prev_acceleration
        }
    }
}