use physics::PhysicsComponentStore;
use cgmath::{Point3, Vector3, InnerSpace, Zero, Matrix3, Quaternion, Matrix};

pub struct PhysicsEngine {

}

fn world_inverse_inertia(local_inertia_inv: &Matrix3<f64>, orientation: Quaternion<f64>)
    -> Matrix3<f64> {
    let body_to_world = Matrix3::from(orientation);
    let world_to_body = body_to_world.transpose();
    body_to_world * local_inertia_inv * world_to_body
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {

        }
    }

    pub fn simulate(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        assert!(dt >= 0.0);
        components.swap_buffers();

        self.integrate_linear_motion(dt, components);
        self.integrate_angular_motion(dt, components);
    }

    fn integrate_linear_motion(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        let num_components = components.num_components();
        let mut view = components.mutable_view();

        // The following is an implementation of Velocity Verlet.
        // See https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet

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

    fn integrate_angular_motion(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        let num_components = components.num_components();
        let mut view = components.mutable_view();

        // The integration for angular motion is a lot more complicated in general,
        // so we can't easily apply something similar to the Velocity Verlet algorithm
        // for linear motion. For now, we just use simple euler integrators instead.

        // Update angular momentum
        for i in 0 .. num_components {
            // TODO: Implement torque accumulators
            let torque = Vector3::zero();
            view.angular_momentum[i] = view.angular_momentum[i] + dt * torque;
        }

        // Update orientation
        for i in 0 .. num_components {
            let orientation = view.prev_orientation[i];
            let inv_inertia_body = view.inv_inertia_body[i];
            let inverse_world_inertia = world_inverse_inertia(&inv_inertia_body, orientation);
            let angular_momentum = view.angular_momentum[i];
            let angular_velocity = inverse_world_inertia * angular_momentum;
            let angular_velocity_quat = Quaternion::from_sv(0.0, angular_velocity);
            let new_orientation = orientation + 0.5 * dt * angular_velocity_quat * orientation;
            view.orientation[i] = new_orientation.normalize();
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
                let f = G * m_i * m_j / r2;
                acceleration[i] += (f / m_i) * r;
                acceleration[j] += - (f / m_j) * r;
            }
        }
    }
}