use physics::*;
use nalgebra::{Matrix3, UnitQuaternion, Isometry3, Translation3};
use ncollide::world::{CollisionWorld3, CollisionGroups, GeometricQueryType};
use ncollide::shape::{ShapeHandle3, Ball, Cuboid};
use entity::{Entity, LinearComponentStorage};

pub struct CollisionEngine {
    world: CollisionWorld3<f64, Entity>
}

// As a quick hack, this is merely copy-pasted from physics_component.rs.
// Need to find a better way to deal with this
fn world_inverse_inertia(local_inertia_inv: &Matrix3<f64>, orientation: UnitQuaternion<f64>)
    -> Matrix3<f64> {
    let body_to_world = orientation.to_rotation_matrix();
    let world_to_body = orientation.inverse().to_rotation_matrix();
    body_to_world * (local_inertia_inv * world_to_body)
}

impl CollisionEngine {
    pub fn new() -> CollisionEngine {
        CollisionEngine {
            world: CollisionWorld3::new(0.02, false)
        }
    }

    pub fn detect_and_resolve(&mut self,
        rigid_bodies: &mut LinearComponentStorage<RigidBody>,
        collision_store: &CollisionComponentStore)
    {
        self.sync_shapes_and_positions(rigid_bodies, collision_store);
        // self.sync_positions(collision_store, rigid_bodies);
        self.world.update();
        self.resolve_collisions(rigid_bodies);
    }

    fn sync_shapes_and_positions(&mut self,
        bodies: &LinearComponentStorage<RigidBody>,
        collision_store: &CollisionComponentStore)
    {
        // TODO: This is very rudimentary and inefficient. Come up with
        // a better way to synchronize component shapes with ncollide
        // shapes
        let entities = collision_store.entities();
        let models = collision_store.models();
        for (entity, model) in izip!(entities, models) {
            let entity_uid: usize = entity.clone().into();

            let rb = bodies.lookup_component_for_entity(entity.clone());

            // At the moment we only allow collisions between rigid bodies,
            // so an associated rigid body component must belong to the entity
            if let Some(rb) = rb {
                let (center, rotation) = match model {
                    &CollisionModel::Sphere(sphere) =>
                        (sphere.center, UnitQuaternion::identity()),
                    &CollisionModel::Cuboid(cuboid) =>
                        (cuboid.center, cuboid.rotation)
                };
                let translation = Translation3::from_vector(center.coords + rb.position().coords);
                let rotation = rb.orientation() * rotation;
                let position = Isometry3::from_parts(translation, rotation);

                let shape_handle = match model {
                    &CollisionModel::Sphere(sphere) => {
                            let ball = Ball::new(sphere.radius);
                            ShapeHandle3::new(ball)
                        },
                        &CollisionModel::Cuboid(cuboid) => {
                            let half_extents = cuboid.half_size;
                            let cuboid = Cuboid::new(half_extents);
                            ShapeHandle3::new(cuboid)
                        }
                };

                if self.world.collision_object(entity_uid).is_none() {
                    self.world.deferred_add(entity_uid,
                        position,
                        shape_handle,
                        CollisionGroups::new(),
                        GeometricQueryType::Contacts(0.0),
                        entity.clone());
                } else {
                    self.world.deferred_set_position(entity_uid, position);
                }
            }

        }
    }

    pub fn resolve_collisions(&mut self,
        bodies: &mut LinearComponentStorage<RigidBody>)
    {
        self.resolve_velocities(bodies);
        self.resolve_interpenetrations(bodies);
    }

    fn resolve_velocities(&mut self,
        bodies: &mut LinearComponentStorage<RigidBody>)
    {
        for (obj1, obj2, contact) in self.world.contacts() {
            // TODO: Move restituion into contact
            let restitution = 1.0;

            // Use the following terminology (suffixed by 1 or 2):
            // v: linear velocity (i.e. velocity of mass center)
            // m: mass
            // w: angular velocity
            // r: contact point relative to center of mass
            // i_inv: inverse inertia tensor (in world coordinates)
            // v_p: velocity at point of contact (includes angular contribution)
            //
            // The mathematics here are based on the following Wikipedia article:
            // https://en.wikipedia.org/wiki/Collision_response#Impulse-based_reaction_model

            let contact_point = contact.world1;
            let (entity1, entity2) = (obj1.data, obj2.data);
            let rb1 = bodies.lookup_component_for_entity(entity1).cloned();
            let rb2 = bodies.lookup_component_for_entity(entity2).cloned();

            // Currently we only support dynamic-dynamic collisions.
            // TODO: Support static-dynamic collisions (and ignore static-static)
            if let (Some(RigidBody::Dynamic(mut rb1)), Some(RigidBody::Dynamic(mut rb2)))
                = (rb1, rb2)
            {
                let orientation1 = rb1.state.orientation;
                let orientation2 = rb2.state.orientation;
                let v1 = rb1.state.velocity;
                let v2 = rb2.state.velocity;
                let m1 = rb1.mass.value();
                let m2 = rb2.mass.value();
                let r1 = contact_point - rb1.state.position;
                let r2 = contact_point - rb2.state.position;
                let i_inv1 = world_inverse_inertia(&rb1.inv_inertia_body, orientation1);
                let i_inv2 = world_inverse_inertia(&rb2.inv_inertia_body, orientation2);
                let w1 = i_inv1 * rb1.state.angular_momentum;
                let w2 = i_inv2 * rb2.state.angular_momentum;
                let v_p1 = v1 + w1.cross(&r1);
                let v_p2 = v2 + w2.cross(&r2);

                // Let n denote the contact normal
                let n = contact.normal;

                // Define the "relative velocity" at the point of impact
                let v_r = v_p2 - v_p1;

                // The separating velocity is the projection of the relative velocity
                // onto the contact normal.
                let v_separating = v_r.dot(&n);

                // If v_separating is non-negative, the objects are not moving
                // towards each other, and we do not need to add any corrective impulse.
                if v_separating < 0.0 {
                    // j_r denotes the relative (reaction) impulse
                    let j_r = {
                        let linear_denominator = 1.0 / m1 + 1.0 / m2;
                        let angular_denominator1 = i_inv1 * r1.cross(&n).cross(&r1);
                        let angular_denominator2 = i_inv2 * r2.cross(&n).cross(&r2);
                        let angular_denominator = (angular_denominator1 + angular_denominator2).dot(&n);
                        let numerator = -(1.0 + restitution) * v_separating;
                        numerator / (linear_denominator + angular_denominator)
                    };

                    // Compute post-collision velocities
                    let v1_post = v1 - j_r / m1 * n;
                    let v2_post = v2 + j_r / m2 * n;
                    let w1_post = w1 - j_r * i_inv1 * r1.cross(&n);
                    let w2_post = w2 + j_r * i_inv2 * r2.cross(&n);
                    rb1.state.velocity = v1_post;
                    rb2.state.velocity = v2_post;

                    // TODO: Avoid the inversions here
                    use interop::try_3x3_inverse;
                    rb1.state.angular_momentum = try_3x3_inverse(i_inv1).unwrap() * w1_post;
                    rb2.state.angular_momentum = try_3x3_inverse(i_inv2).unwrap() * w2_post;
                }

                bodies.set_component_for_entity(entity1, RigidBody::Dynamic(rb1));
                bodies.set_component_for_entity(entity2, RigidBody::Dynamic(rb2));
            }
        }
    }

    fn resolve_interpenetrations(&mut self,
        bodies: &mut LinearComponentStorage<RigidBody>)
    {
        for (obj1, obj2, contact) in self.world.contacts() {
            let (entity1, entity2) = (obj1.data, obj2.data);
            let rb1 = bodies.lookup_component_for_entity(entity1).cloned();
            let rb2 = bodies.lookup_component_for_entity(entity2).cloned();
            if let (Some(RigidBody::Dynamic(mut rb1)), Some(RigidBody::Dynamic(mut rb2)))
                = (rb1, rb2)
            {
                let m1 = rb1.mass.value();
                let m2 = rb2.mass.value();
                let total_mass = m1 + m2;

                // Move the two objects linearly away from each other along the contact normal.
                // The distance to move is determined by the relative masses of the two objects,
                // and the penetration depth.
                let obj1_move_dist = (m2 / total_mass) * contact.depth;
                let obj2_move_dist = (m1 / total_mass) * contact.depth;

                rb1.state.position -= obj1_move_dist * contact.normal;
                rb2.state.position += obj2_move_dist * contact.normal;

                bodies.set_component_for_entity(entity1, RigidBody::Dynamic(rb1));
                bodies.set_component_for_entity(entity2, RigidBody::Dynamic(rb2));
            }
        }
    }

}
