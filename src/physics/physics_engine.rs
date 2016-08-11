use physics::PhysicsComponentStore;
use cgmath::{Point3, Vector3, InnerSpace, Zero};

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

    fn compute_current_acceleration(&self, components: &PhysicsComponentStore) -> Vec<Vector3<f64>> {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        const G: f64 = 6.674e-11;
        let mut accel = Vec::new();
        accel.resize(components.num_components(), Vector3::zero());
        for i in 0 .. components.num_components() {
            for j in (i + 1) .. components.num_components() {
                let m_i = components.masses()[i];
                let m_j = components.masses()[j];
                let x_i = components.positions()[i];
                let x_j = components.positions()[j];
                let r = x_j - x_i;
                // Since we don't have collision detection at the moment,
                let mut r2 = r.magnitude2();
                if r2 < 0.05 { r2 = 0.2; }
                let F = G * m_i * m_j / r2;
                accel[i] += (F / m_i) * r;
                accel[j] += - (F / m_j) * r;
            }
        }
        accel
    }

    fn compute_new_velocities(&self, dt: f64, components: &PhysicsComponentStore) -> Vec<Vector3<f64>> {
        let accel = self.compute_current_acceleration(components);

        self.prev_velocity.iter().zip(accel)
                                 .map(|(v_prev, a_current)| 2.0 * dt * a_current + v_prev)
                                 .collect()
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