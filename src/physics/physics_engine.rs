use physics::PhysicsComponentStore;
use cgmath::{Point3, Vector3, InnerSpace, Zero};
use itertools;

pub struct PhysicsEngine {

}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {

        }
    }

    pub fn simulate(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        // Note: This is a very naive implementation with unnecessary allocations.
        assert!(dt >= 0.0);

        // Update previous positions with the data from what are currently
        // the current positions
        let current_positions = components.positions().to_vec();
        components.prev_positions_mut().copy_from_slice(&current_positions);


        let new_position: Vec<Point3<f64>> = {
            let xva_triplet = izip!(components.positions(),
                            components.velocities(),
                            components.accelerations());

            xva_triplet.map(|(x, v, a)| x + dt * v + 0.5 * dt * dt * a)
                       .collect()
        };

        let new_acceleration = self.compute_acceleration(&new_position, components.masses());
        let new_velocity: Vec<Vector3<f64>> = {
            let vaa_triplet = izip!(components.velocities(),
                                components.accelerations(),
                                &new_acceleration);
            vaa_triplet.map(|(v, a_curr, a_next)| v + 0.5 * dt * (a_curr + a_next))
                       .collect()
        };

        components.positions_mut().copy_from_slice(&new_position);
        components.velocities_mut().copy_from_slice(&new_velocity);
        components.accelerations_mut().copy_from_slice(&new_acceleration);
    }

    fn compute_acceleration(&self, positions: &[Point3<f64>], masses: &[f64]) -> Vec<Vector3<f64>> {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        let num_objects = positions.len();
        assert!(num_objects == masses.len());

        const G: f64 = 6.674e-11;
        let mut accel = Vec::new();
        accel.resize(num_objects, Vector3::zero());
        for i in 0 .. num_objects {
            for j in (i + 1) .. num_objects {
                let m_i = masses[i];
                let m_j = masses[j];
                let x_i = positions[i];
                let x_j = positions[j];
                let r = x_j - x_i;
                let r2 = r.magnitude2();
                let F = G * m_i * m_j / r2;
                accel[i] += (F / m_i) * r;
                accel[j] += - (F / m_j) * r;
            }
        }
        accel
    }
}