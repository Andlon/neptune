use cgmath::{Point3, Vector3, Zero, Matrix3, Quaternion, EuclideanSpace, SquareMatrix};
use store::{Identifier, OneToOneStore};
use std::collections::HashMap;
use std::collections::hash_map::Iter;
use entity::Entity;
use itertools::Zip;

pub type PhysicsComponentId = usize;

pub struct PhysicsComponentsView<'a> {
    // Dynamic properties
    pub position: &'a [Point3<f64>],
    pub velocity: &'a [Vector3<f64>],
    pub orientation: &'a [Quaternion<f64>],
    pub angular_momentum: &'a [Vector3<f64>],

    // Static properties
    pub mass: &'a [f64],
    pub inv_inertia_body: &'a [Matrix3<f64>],

    // Intermediate properties for integration and interpolation
    pub acceleration: &'a [Vector3<f64>],
    pub prev_position: &'a [Point3<f64>],
    pub prev_orientation: &'a [Quaternion<f64>],
    pub prev_acceleration: &'a [Vector3<f64>],
}

pub struct MutablePhysicsComponentsView<'a> {
    // Dynamic properties
    pub position: &'a mut [Point3<f64>],
    pub velocity: &'a mut [Vector3<f64>],
    pub orientation: &'a mut [Quaternion<f64>],
    pub angular_momentum: &'a mut [Vector3<f64>],

    // Static properties
    pub mass: &'a mut [f64],
    pub inv_inertia_body: &'a mut [Matrix3<f64>],

    // Intermediate properties for integration and interpolation
    pub acceleration: &'a mut [Vector3<f64>],
    pub prev_position: &'a mut [Point3<f64>],
    pub prev_orientation: &'a mut [Quaternion<f64>],
    pub prev_acceleration: &'a mut [Vector3<f64>],
}

pub struct PhysicsComponentStore {
    // Dynamic properties
    position: Vec<Point3<f64>>,
    velocity: Vec<Vector3<f64>>,
    orientation: Vec<Quaternion<f64>>,
    angular_momentum: Vec<Vector3<f64>>,

    // Static properties
    mass: Vec<f64>,
    inv_inertia_body: Vec<Matrix3<f64>>,

    // Intermediate properties for integration and interpolation
    acceleration: Vec<Vector3<f64>>,
    prev_position: Vec<Point3<f64>>,
    prev_orientation: Vec<Quaternion<f64>>,
    prev_acceleration: Vec<Vector3<f64>>,

    entity_map: HashMap<Entity, PhysicsComponentId>,
}

pub struct PhysicsComponent {
    pub position: Point3<f64>,
    pub velocity: Vector3<f64>,
    pub orientation: Quaternion<f64>,
    pub angular_velocity: Vector3<f64>,
    pub mass: f64,
    pub inertia_body: Matrix3<f64>
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        PhysicsComponent {
            position: Point3::origin(),
            velocity: Vector3::zero(),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            angular_velocity: Vector3::zero(),
            mass: 0.0,
            inertia_body: Matrix3::identity()
        }
    }
}

impl PhysicsComponentStore {
    pub fn new() -> Self {
        PhysicsComponentStore {
            position: Vec::new(),
            velocity: Vec::new(),
            orientation: Vec::new(),
            angular_momentum: Vec::new(),
            mass: Vec::new(),
            inv_inertia_body: Vec::new(),
            acceleration: Vec::new(),
            prev_position: Vec::new(),
            prev_orientation: Vec::new(),
            prev_acceleration: Vec::new(),
            entity_map: HashMap::new()
        }
    }

    pub fn set_component_properties(&mut self,
        entity: Entity,
        component: PhysicsComponent) -> PhysicsComponentId
    {
        assert!(component.mass >= 0.0, "Mass must be non-negative.");

        // It's far more user friendly to let the user supply angular
        // velocity and inertia tensor instead of angular momentum
        // and the inverse inertia tensor.
        let inv_inertia_body = component.inertia_body.invert().expect("Must be invertible. Replace with Result");
        let angular_momentum = component.inertia_body * component.angular_velocity;

        // Note that we set acceleration to zero, because it will be
        // computed by the physics engine.
        let next_available_index = self.num_components();
        let index: usize = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index >= self.num_components() {
            self.position.push(component.position);
            self.velocity.push(component.velocity);
            self.orientation.push(component.orientation);
            self.angular_momentum.push(angular_momentum);
            self.mass.push(component.mass);
            self.inv_inertia_body.push(inv_inertia_body);
            self.acceleration.push(Vector3::zero());
            self.prev_position.push(component.position);
            self.prev_orientation.push(component.orientation);
            self.prev_acceleration.push(Vector3::zero());
        } else {
            self.position[index] = component.position;
            self.velocity[index] = component.velocity;
            self.orientation[index] = component.orientation;
            self.angular_momentum[index] = angular_momentum;
            self.mass[index] = component.mass;
            self.inv_inertia_body[index] = inv_inertia_body;
            self.acceleration[index] = Vector3::zero();
            self.prev_position[index] = component.position;
            self.prev_orientation[index] = component.orientation;
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

    pub fn lookup_orientation(&self, component: &PhysicsComponentId) -> Quaternion<f64> {
        self.orientation[component.to_owned()]
    }

    pub fn lookup_prev_orientation(&self, component: &PhysicsComponentId) -> Quaternion<f64> {
        self.prev_orientation[component.to_owned()]
    }

    pub fn num_components(&self) -> usize {
        assert!(self.position.len() == self.velocity.len() && self.velocity.len() == self.mass.len());
        self.position.len()
    }

    pub fn swap_buffers(&mut self) {
        use std::mem::swap;
        swap(&mut self.position, &mut self.prev_position);
        swap(&mut self.acceleration, &mut self.prev_acceleration);
        swap(&mut self.orientation, &mut self.prev_orientation);
    }

    pub fn view<'a>(&'a self) -> PhysicsComponentsView<'a> {
        PhysicsComponentsView {
            position: &self.position,
            velocity: &self.velocity,
            orientation: &self.orientation,
            angular_momentum: &self.angular_momentum,
            mass: &self.mass,
            inv_inertia_body: &self.inv_inertia_body,
            acceleration: &self.acceleration,
            prev_position: &self.prev_position,
            prev_orientation: &self.prev_orientation,
            prev_acceleration: &self.prev_acceleration,
        }
    }

    pub fn mutable_view<'a>(&'a mut self) -> MutablePhysicsComponentsView<'a> {
        MutablePhysicsComponentsView {
            position: &mut self.position,
            velocity: &mut self.velocity,
            orientation: &mut self.orientation,
            angular_momentum: &mut self.angular_momentum,
            mass: &mut self.mass,
            inv_inertia_body: &mut self.inv_inertia_body,
            acceleration: &mut self.acceleration,
            prev_position: &mut self.prev_position,
            prev_orientation: &mut self.prev_orientation,
            prev_acceleration: &mut self.prev_acceleration,
        }
    }
}