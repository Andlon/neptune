use physics::PhysicsComponentStore;
use cgmath::{Point3, Vector3};

pub struct PhysicsEngine {
    prev_position: Vec<Point3<f64>>,
    prev_velocity: Vec<Vector3<f64>>,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            prev_position: Vec::new(),
            prev_velocity: Vec::new()
        }
    }

    pub fn simulate(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        assert!(dt >= 0.0);
        // Note: This is a super-naive implementation. We'll want to optimize a bit
        // in terms of avoiding new allocations etc.
        self.add_prev_for_new_components(components);

        let new_velocities = self.compute_new_velocities(dt, components);
        let new_positions = self.compute_new_positions(dt, components);;

        self.prev_position.as_mut_slice().copy_from_slice(components.positions());
        self.prev_velocity.as_mut_slice().copy_from_slice(components.velocities());
        components.positions_mut().copy_from_slice(&new_positions[..]);
        components.velocities_mut().copy_from_slice(&new_velocities[..]);
    }

    fn compute_new_velocities(&self, dt: f64, components: &PhysicsComponentStore) -> Vec<Vector3<f64>> {
        // TODO: Implement support for acceleration
        self.prev_velocity.clone()
    }

    fn compute_new_positions(&self, dt: f64, components: &PhysicsComponentStore) -> Vec<Point3<f64>> {
        self.prev_position.iter().zip(components.velocities())
                                 .map(|(x_prev, v_current)| x_prev + 2.0 * dt * v_current)
                                 .map(|vector| Point3::from(vector))
                                 .collect()
    }

    fn add_prev_for_new_components(&mut self, components: &PhysicsComponentStore) {
        let new_positions = &components.positions()[self.num_prev_components() ..];
        let new_velocities = &components.velocities()[self.num_prev_components() ..];
        self.prev_position.extend_from_slice(new_positions);
        self.prev_velocity.extend_from_slice(new_velocities);
    }

    fn num_prev_components(&self) -> usize {
        assert!(self.prev_position.len() == self.prev_velocity.len());
        self.prev_position.len()
    }
}