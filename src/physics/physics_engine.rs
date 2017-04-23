use physics::{Mass, RigidBody};
use nalgebra::{zero, norm_squared, Point3, Vector3, Matrix3, Quaternion, UnitQuaternion};
use core::{TransformPair, TransformStore};
use interop;
use entity::LinearComponentStorage;

pub struct PhysicsEngine {
    // Buffers for intermediate computations
    x: Vec<Point3<f64>>,
    v: Vec<Vector3<f64>>,
    a: Vec<Vector3<f64>>,
    a_next: Vec<Vector3<f64>>,
    m: Vec<f64>
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
            x: Vec::new(),
            v: Vec::new(),
            a: Vec::new(),
            a_next: Vec::new(),
            m: Vec::new()
        }
    }

    pub fn simulate(&mut self,
                    dt: f64,
                    rigid_bodies: &mut LinearComponentStorage<RigidBody>,
                    transforms: &mut TransformStore) {
        assert!(dt >= 0.0);
        // TODO: Eliminate transforms altogether
        self.update_bodies_from_transforms(rigid_bodies, transforms);
        self.populate_buffers(rigid_bodies);
        self.integrate_linear_motion(dt);
        self.integrate_angular_motion(dt, rigid_bodies);
        self.sync_components_from_buffers(rigid_bodies);
        self.update_transforms_from_bodies(rigid_bodies, transforms);
    }

    fn populate_buffers(&mut self, rigid_bodies: &LinearComponentStorage<RigidBody>)
    {
        let n = rigid_bodies.num_components();
        self.x.resize(n, Point3::origin());
        self.v.resize(n, zero::<Vector3<_>>());
        self.a.resize(n, zero::<Vector3<_>>());
        self.a_next.resize(n, zero::<Vector3<_>>());
        self.m.resize(n, 0.0);
        let rb_iter = rigid_bodies.components().iter().map(|&(ref rb, _)| rb);
        let iter = izip!(rb_iter,
                         self.x.iter_mut(),
                         self.v.iter_mut(),
                         self.a.iter_mut(),
                         self.m.iter_mut());
        for (rb, x, v, a, m) in iter {
            *x = rb.state.position;
            *v = rb.state.velocity;
            *a = rb.state.acceleration;
            *m = rb.mass.value();
        }
    }

    fn sync_components_from_buffers(&self,
        rigid_bodies: &mut LinearComponentStorage<RigidBody>)
    {
        let n = rigid_bodies.num_components();
        assert!(self.x.len() == n);
        assert!(self.v.len() == n);
        assert!(self.a_next.len() == n);
        assert!(self.m.len() == n);

        let rb_iter = rigid_bodies.components_mut()
                                  .iter_mut()
                                  .map(|&mut (ref mut rb, _)| rb);
        let iter = izip!(rb_iter,
                         self.x.iter(),
                         self.v.iter(),
                         self.a_next.iter(),
                         self.m.iter());
        for (rb, x, v, a_next, m) in iter {
            rb.prev_state.position = rb.state.position;
            rb.state.position = x.clone();

            rb.prev_state.velocity = rb.state.velocity;
            rb.state.velocity = v.clone();

            rb.prev_state.acceleration = rb.state.acceleration;
            rb.state.acceleration = a_next.clone();

            rb.mass = Mass::new(m.clone());
        }
    }

    fn integrate_linear_motion(&mut self, dt: f64)
    {
        // The following is an implementation of Velocity Verlet.
        // See https://en.wikipedia.org/wiki/Verlet_integration#Velocity_Verlet

        assert!(self.x.len() == self.v.len()
            && self.v.len() == self.a.len()
            && self.a.len() == self.a_next.len()
            && self.a_next.len() == self.m.len());

        let num_components = self.x.len();

        // Update positions
        for i in 0 .. num_components {
            let ref mut x = self.x[i];
            let v = self.v[i];
            let a = self.a[i];
            *x += dt * v + 0.5 * dt * dt * a;
        }

        // Update acceleration
        self.compute_acceleration();

        // Update velocities
        for i in 0 .. num_components {
            let ref mut v = self.v[i];
            let a = self.a[i];
            let a_next = self.a_next[i];
            *v += 0.5 * dt * (a + a_next);
        }
    }

    fn integrate_angular_motion(&mut self,
        dt: f64,
        rigid_bodies: &mut LinearComponentStorage<RigidBody>)
    {

        // The integration for angular motion is a lot more complicated in general,
        // so we can't easily apply something similar to the Velocity Verlet algorithm
        // for linear motion. For now, we just use simple euler integrators instead.

        // TODO: Implement torque accumulators

        for &mut (ref mut rb, _) in rigid_bodies.components_mut() {
            rb.prev_state.orientation = rb.state.orientation;

            let orientation = rb.state.orientation;
            let inv_inertia_body = rb.inv_inertia_body;
            let inverse_world_inertia = world_inverse_inertia(&inv_inertia_body, orientation);
            let angular_momentum = rb.state.angular_momentum;
            let angular_velocity = inverse_world_inertia * angular_momentum;
            let angular_velocity_quat = Quaternion::from_parts(0.0, angular_velocity);

            // The orientation update first makes the quaternion non-unit.
            // This means that we need to:
            // 1. Turn the UnitQuaternion into Quaternion by unwrapping
            // 2. Update the Quaternion instance
            // 3. Normalize the updated Quaternion into a new UnitQuaternion
            let orientation = orientation.unwrap();
            let new_orientation = orientation + 0.5 * dt * angular_velocity_quat * orientation;
            rb.state.orientation = UnitQuaternion::new_normalize(new_orientation);
        }
    }

    fn compute_acceleration(&mut self)
    {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        let num_objects = self.a_next.len();

        // Reset the acceleration to zero before summation
        for accel in self.a_next.iter_mut() {
            *accel = zero::<Vector3<f64>>();
        }

        const G: f64 = 6.674e-11;
        for i in 0 .. num_objects {
            for j in (i + 1) .. num_objects {
                let m_i = self.m[i];
                let m_j = self.m[j];
                let x_i = self.x[i];
                let x_j = self.x[j];
                let r = x_j - x_i;
                let r2 = norm_squared(&r);
                let f = G * m_i * m_j / r2;
                // TODO: Since r isn't normalized, doesn't this give the wrong
                // values for the acceleration? Investigate!
                self.a_next[i] += (f / m_i) * r;
                self.a_next[j] += - (f / m_j) * r;
            }
        }
    }

    fn update_bodies_from_transforms(&mut self,
        bodies: &mut LinearComponentStorage<RigidBody>,
        transforms: &TransformStore)
    {
        for &mut (ref mut rb, entity) in bodies.components_mut() {
            let pair = transforms.lookup(&entity)
                                 .expect("All entities with a Physics component must have a Transform component!");
            let &TransformPair { current, prev } = pair;
            rb.state.position = interop::cgmath_point3_to_nalgebra(&current.position);
            rb.state.orientation = UnitQuaternion::new_normalize(interop::cgmath_quat_to_nalgebra(&current.orientation));
            rb.prev_state.position = interop::cgmath_point3_to_nalgebra(&prev.position);
            rb.prev_state.orientation = UnitQuaternion::new_normalize(
                interop::cgmath_quat_to_nalgebra(&prev.orientation));
        }
    }

    fn update_transforms_from_bodies(&self,
        bodies: &LinearComponentStorage<RigidBody>,
        transforms: &mut TransformStore)
    {
        debug_assert!(self.x.len() == self.v.len());
        debug_assert!(self.v.len() == self.a.len());
        debug_assert!(self.a.len() == self.a_next.len());

        for &(ref rb, entity) in bodies.components() {
            let transform_pair = transforms.lookup_mut(&entity)
                                           .expect("Physics component is expected to have a transform component!");
            transform_pair.current.position =
                interop::nalgebra_point3_to_cgmath(&rb.state.position);
            transform_pair.current.orientation =
                interop::nalgebra_unit_quat_to_cgmath(&rb.state.orientation);
            transform_pair.prev.position =
                interop::nalgebra_point3_to_cgmath(&rb.prev_state.position);
            transform_pair.prev.orientation =
                interop::nalgebra_unit_quat_to_cgmath(&rb.prev_state.orientation);
        }
    }
}
