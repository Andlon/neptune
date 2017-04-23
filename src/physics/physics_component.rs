use nalgebra;
use nalgebra::{Vector3, Matrix3};
use std::collections::HashMap;
use entity::Entity;

pub type PhysicsComponentId = usize;

pub struct PhysicsComponentsView<'a> {
    // Dynamic properties
    pub velocity: &'a [Vector3<f64>],
    pub angular_momentum: &'a [Vector3<f64>],

    // Static properties
    pub mass: &'a [f64],
    pub inv_inertia_body: &'a [Matrix3<f64>],
    pub entity: &'a [Entity]
}

pub struct MutablePhysicsComponentsView<'a> {
    // Dynamic properties
    pub velocity: &'a mut [Vector3<f64>],
    pub angular_momentum: &'a mut [Vector3<f64>],

    // Static properties
    pub mass: &'a mut [f64],
    pub inv_inertia_body: &'a mut [Matrix3<f64>],
    pub entity: &'a [Entity]
}

pub struct PhysicsComponentStore {
    // Dynamic properties
    velocity: Vec<Vector3<f64>>,
    angular_momentum: Vec<Vector3<f64>>,

    // Static properties
    mass: Vec<f64>,
    inv_inertia_body: Vec<Matrix3<f64>>,

    entity_map: HashMap<Entity, PhysicsComponentId>,
    entity: Vec<Entity>
}

pub struct PhysicsComponent {
    pub velocity: Vector3<f64>,
    pub angular_velocity: Vector3<f64>,
    pub mass: f64,
    pub inertia_body: Matrix3<f64>
}

impl Default for PhysicsComponent {
    fn default() -> Self {
        PhysicsComponent {
            velocity: nalgebra::zero::<Vector3<_>>(),
            angular_velocity: nalgebra::zero::<Vector3<_>>(),
            mass: 0.0,
            inertia_body: Matrix3::identity()
        }
    }
}

impl PhysicsComponentStore {
    pub fn new() -> Self {
        PhysicsComponentStore {
            velocity: Vec::new(),
            angular_momentum: Vec::new(),
            mass: Vec::new(),
            inv_inertia_body: Vec::new(),
            entity_map: HashMap::new(),
            entity: Vec::new(),
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
        let inv_inertia_body = component.inertia_body.try_inverse()
                                        .expect("Must be invertible. Replace with Result");
        let angular_momentum = component.inertia_body * component.angular_velocity;

        // Note that we set acceleration to zero, because it will be
        // computed by the physics engine.
        let next_available_index = self.num_components();
        let index: usize = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index >= self.num_components() {
            self.velocity.push(component.velocity);
            self.angular_momentum.push(angular_momentum);
            self.mass.push(component.mass);
            self.inv_inertia_body.push(inv_inertia_body);
            self.entity.push(entity);
        } else {
            self.velocity[index] = component.velocity;
            self.angular_momentum[index] = angular_momentum;
            self.mass[index] = component.mass;
            self.inv_inertia_body[index] = inv_inertia_body;
            self.entity[index] = entity;
        }
        index
    }

    pub fn lookup_component(&self, entity: &Entity) -> Option<PhysicsComponentId> {
        self.entity_map.get(entity).map(|x| x.to_owned())
    }

    pub fn num_components(&self) -> usize {
        debug_assert!(self.velocity.len() == self.angular_momentum.len());
        debug_assert!(self.angular_momentum.len() == self.mass.len());
        debug_assert!(self.mass.len() == self.inv_inertia_body.len());
        self.velocity.len()
    }

    #[allow(dead_code)]
    pub fn view<'a>(&'a self) -> PhysicsComponentsView<'a> {
        PhysicsComponentsView {
            velocity: &self.velocity,
            angular_momentum: &self.angular_momentum,
            mass: &self.mass,
            inv_inertia_body: &self.inv_inertia_body,
            entity: &self.entity
        }
    }

    pub fn mutable_view<'a>(&'a mut self) -> MutablePhysicsComponentsView<'a> {
        MutablePhysicsComponentsView {
            velocity: &mut self.velocity,
            angular_momentum: &mut self.angular_momentum,
            mass: &mut self.mass,
            inv_inertia_body: &mut self.inv_inertia_body,
            entity: &self.entity
        }
    }

    pub fn clear(&mut self) {
        self.velocity.clear();
        self.angular_momentum.clear();
        self.mass.clear();
        self.inv_inertia_body.clear();
        self.entity_map.clear();
        self.entity.clear();
    }
}
