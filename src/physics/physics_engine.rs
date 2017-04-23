use physics::PhysicsComponentStore;
use nalgebra::{zero, norm_squared, Point3, Vector3, Matrix3, Quaternion, UnitQuaternion};
use core::{TransformPair, TransformStore};
use interop;

pub struct PhysicsEngine {
    position: Vec<Point3<f64>>,
    orientation: Vec<UnitQuaternion<f64>>,
    acceleration: Vec<Vector3<f64>>,
    prev_position: Vec<Point3<f64>>,
    prev_orientation: Vec<UnitQuaternion<f64>>,
    prev_acceleration: Vec<Vector3<f64>>
}

fn world_inverse_inertia(local_inertia_inv: &Matrix3<f64>, orientation: UnitQuaternion<f64>)
    -> Matrix3<f64> {
    let body_to_world = orientation.to_rotation_matrix();
    let world_to_body = orientation.inverse().to_rotation_matrix();
    body_to_world * (local_inertia_inv * world_to_body)
}

impl PhysicsEngine {
    pub fn new() -> Self {
        PhysicsEngine {
            position: Vec::new(),
            orientation: Vec::new(),
            acceleration: Vec::new(),
            prev_position: Vec::new(),
            prev_orientation: Vec::new(),
            prev_acceleration: Vec::new()
        }
    }

    pub fn simulate(&mut self,
                    dt: f64,
                    components: &mut PhysicsComponentStore,
                    transforms: &mut TransformStore) {
        assert!(dt >= 0.0);
        self.update_buffers_from_transforms(components, transforms);
        self.swap_buffers();
        self.integrate_linear_motion(dt, components);
        self.integrate_angular_motion(dt, components);
        self.update_transforms_from_buffers(components, transforms);
    }

    fn integrate_linear_motion(&mut self, dt: f64, components: &mut PhysicsComponentStore) {
        let num_components = components.num_components();
        let mut view = components.mutable_view();

        // The following is an implementation of Velocity Verlet.
        // See https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet

        // Update positions
        for i in 0 .. num_components {
            let x = self.prev_position[i];
            let v = view.velocity[i];
            let a = self.prev_acceleration[i];
            self.position[i] = x + dt * v + 0.5 * dt * dt * a;
        }

        // Update acceleration
        self.compute_acceleration(&view.mass);

        // Update velocities
        for i in 0 .. num_components {
            let a_prev = &self.prev_acceleration[i];
            let a = &self.acceleration[i];
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
            let torque = zero::<Vector3<f64>>();
            view.angular_momentum[i] = view.angular_momentum[i] + dt * torque;
        }

        // Update orientation
        for i in 0 .. num_components {
            let orientation = self.prev_orientation[i];
            let inv_inertia_body = view.inv_inertia_body[i];
            let inverse_world_inertia = world_inverse_inertia(&inv_inertia_body, orientation);
            let angular_momentum = view.angular_momentum[i];
            let angular_velocity = inverse_world_inertia * angular_momentum;
            let angular_velocity_quat = Quaternion::from_parts(0.0, angular_velocity);

            // The orientation update first makes the quaternion non-unit.
            // This means that we need to:
            // 1. Turn the UnitQuaternion into Quaternion by unwrapping
            // 2. Update the Quaternion instance
            // 3. Normalize the updated Quaternion into a new UnitQuaternion
            let orientation = orientation.unwrap();
            let new_orientation = orientation + 0.5 * dt * angular_velocity_quat * orientation;
            self.orientation[i] = UnitQuaternion::new_normalize(new_orientation);
        }
    }

    fn compute_acceleration(&mut self,
        mass: &[f64])
    {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        let num_objects = self.acceleration.len();

        // Reset the acceleration to zero before summation
        for accel in self.acceleration.iter_mut() {
            *accel = zero::<Vector3<f64>>();
        }

        const G: f64 = 6.674e-11;
        for i in 0 .. num_objects {
            for j in (i + 1) .. num_objects {
                let m_i = mass[i];
                let m_j = mass[j];
                let x_i = self.position[i];
                let x_j = self.position[j];
                let r = x_j - x_i;
                let r2 = norm_squared(&r);
                let f = G * m_i * m_j / r2;
                self.acceleration[i] += (f / m_i) * r;
                self.acceleration[j] += - (f / m_j) * r;
            }
        }
    }

    fn update_buffers_from_transforms(&mut self, components: &PhysicsComponentStore, transforms: &TransformStore) {
        let num_components = components.num_components();
        self.position.resize(num_components, Point3::origin());
        self.orientation.resize(num_components, UnitQuaternion::identity());
        self.acceleration.resize(num_components, zero::<Vector3<f64>>());
        self.prev_position.resize(num_components, Point3::origin());
        self.prev_orientation.resize(num_components, UnitQuaternion::identity());
        self.prev_acceleration.resize(num_components, zero::<Vector3<f64>>());

        let view = components.view();

        for i in 0 .. num_components {
            let entity = view.entity[i];
            let pair = transforms.lookup(&entity)
                                 .expect("All entities with a Physics component must have a Transform component!");
            let &TransformPair { current, prev } = pair;

            self.position[i] = interop::cgmath_point3_to_nalgebra(&current.position);
            self.orientation[i] = UnitQuaternion::new_normalize(interop::cgmath_quat_to_nalgebra(&current.orientation));
            self.prev_position[i] = interop::cgmath_point3_to_nalgebra(&prev.position);
            self.prev_orientation[i] = UnitQuaternion::new_normalize(interop::cgmath_quat_to_nalgebra(&prev.orientation));
        }
    }

    fn swap_buffers(&mut self) {
        use std::mem;
        mem::swap(&mut self.position, &mut self.prev_position);
        mem::swap(&mut self.orientation, &mut self.prev_orientation);
        mem::swap(&mut self.acceleration, &mut self.prev_acceleration);
    }

    fn update_transforms_from_buffers(&self, components: &PhysicsComponentStore, transforms: &mut TransformStore) {
        debug_assert!(components.num_components() == self.position.len());
        debug_assert!(self.position.len() == self.prev_position.len());
        debug_assert!(self.prev_position.len() == self.orientation.len());
        debug_assert!(self.orientation.len() == self.prev_orientation.len());
        debug_assert!(self.prev_orientation.len() == self.acceleration.len());
        debug_assert!(self.acceleration.len() == self.prev_acceleration.len());

        let view = components.view();
        for i in 0 .. components.num_components() {
            let entity = view.entity[i];
            let transform_pair = transforms.lookup_mut(&entity)
                                           .expect("Physics component is expected to have a transform component!");
            transform_pair.current.position = interop::nalgebra_point3_to_cgmath(&self.position[i]);
            transform_pair.current.orientation = interop::nalgebra_unit_quat_to_cgmath(&self.orientation[i]);
            transform_pair.prev.position = interop::nalgebra_point3_to_cgmath(&self.prev_position[i]);
            transform_pair.prev.orientation = interop::nalgebra_unit_quat_to_cgmath(&self.prev_orientation[i]);
        }
    }
}
