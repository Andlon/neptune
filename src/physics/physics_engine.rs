use physics::{Mass, RigidBody, CollisionEngine, CollisionComponentStore};
use nalgebra::{zero, norm_squared, Point3, Vector3, Matrix3, Quaternion, UnitQuaternion};
use entity::LinearComponentStorage;

pub struct PhysicsEngine {
    // Buffers for intermediate computations
    // TODO: Move into structs with specialized responsibility,
    // i.e. LinearMotionIntegrator
    x: Vec<Point3<f64>>,
    v: Vec<Vector3<f64>>,
    a: Vec<Vector3<f64>>,
    a_next: Vec<Vector3<f64>>,
    m: Vec<f64>,

    collision_engine: CollisionEngine,
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
            m: Vec::new(),

            collision_engine: CollisionEngine::new(),
        }
    }

    pub fn simulate(&mut self,
                    dt: f64,
                    rigid_bodies: &mut LinearComponentStorage<RigidBody>,
                    collision_store: &CollisionComponentStore)
    {
        assert!(dt >= 0.0);
        self.populate_buffers(rigid_bodies);
        self.integrate_linear_motion(dt);
        self.integrate_angular_motion(dt, rigid_bodies);
        self.sync_components_from_buffers(rigid_bodies);

        self.collision_engine.detect_and_resolve(rigid_bodies, collision_store);
    }

    fn populate_buffers(&mut self, rigid_bodies: &LinearComponentStorage<RigidBody>)
    {
        self.x.clear();
        self.v.clear();
        self.a.clear();
        self.a_next.clear();
        self.m.clear();

        let dynamic_iter = rigid_bodies.components()
                                  .iter()
                                  .filter_map(|&(ref rb, _)| rb.as_dynamic());

        for rb in dynamic_iter {
            self.x.push(rb.state.position);
            self.v.push(rb.state.velocity);
            self.a.push(rb.state.acceleration);
            self.m.push(rb.mass.value());
        }

        self.a_next.resize(self.a.len(), zero::<Vector3<f64>>());
    }

    fn sync_components_from_buffers(&self,
        rigid_bodies: &mut LinearComponentStorage<RigidBody>)
    {
        let dynamic_iter = rigid_bodies.components_mut()
                                  .iter_mut()
                                  .filter_map(|&mut (ref mut rb, _)| rb.as_dynamic_mut());

        let iter = izip!(dynamic_iter, &self.x, &self.v, &self.a_next, &self.m);

        let mut count = 0;
        for (rb, x, v, a_next, m) in iter {
            rb.prev_state.position = rb.state.position;
            rb.state.position = x.clone();

            rb.prev_state.velocity = rb.state.velocity;
            rb.state.velocity = v.clone();

            rb.prev_state.acceleration = rb.state.acceleration;
            rb.state.acceleration = a_next.clone();

            rb.mass = Mass::new(m.clone());

            count += 1;
        }

        // Sanity check
        assert!(self.x.len() == count);
        assert!(self.v.len() == count);
        assert!(self.a.len() == count);
        assert!(self.a_next.len() == count);
        assert!(self.m.len() == count);
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
            if let &mut RigidBody::Dynamic(ref mut rb) = rb {
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
    }

    fn compute_acceleration(&mut self)
    {
        // TODO: This only takes into account gravity, so perhaps move into a gravity-only function.
        let num_objects = self.a.len();
        self.a_next.clear();

        // Reset the acceleration to zero before summation
        self.a_next.resize(num_objects, zero::<Vector3<f64>>());

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
}
