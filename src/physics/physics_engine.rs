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
        assert!(dt >= 0.0);

        // The following is an implementation of Velocity Verlet.
        // See https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet

        components.swap_buffers();

        let num_components = components.num_components();
        let mut view = components.mutable_view();

        // Update positions
        for i in 0 .. num_components {
            let x = view.prev_position[i];
            let v = view.velocity[i];
            let a = view.prev_acceleration[i];
            view.position[i] = x + dt * v + 0.5 * dt * dt * a;
        }

        // Update acceleration
        self.compute_acceleration(
            &mut view.acceleration,
            &view.position,
            &view.mass);

        // Update velocities
        for i in 0 .. num_components {
            let a_prev = &view.prev_acceleration[i];
            let a = &view.acceleration[i];
            let v = &mut view.velocity[i];
            *v += 0.5 * dt * (a_prev + a);
        }
    }

    fn compute_acceleration(&self,
        acceleration: &mut [Vector3<f64>],
        position: &[Point3<f64>],
        mass: &[f64])
    {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        let num_objects = acceleration.len();
        assert!(num_objects == position.len());
        assert!(num_objects == mass.len());

        // Reset the acceleration to zero before summation
        for accel in acceleration.iter_mut() {
            *accel = Vector3::zero();
        }

        const G: f64 = 6.674e-11;
        for i in 0 .. num_objects {
            for j in (i + 1) .. num_objects {
                let m_i = mass[i];
                let m_j = mass[j];
                let x_i = position[i];
                let x_j = position[j];
                let r = x_j - x_i;
                let r2 = r.magnitude2();
                let F = G * m_i * m_j / r2;
                acceleration[i] += (F / m_i) * r;
                acceleration[j] += - (F / m_j) * r;
            }
        }
    }
}