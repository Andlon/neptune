use physics::*;
use geometry::{Sphere, Cuboid};
use cgmath::{InnerSpace, Matrix3, Quaternion, Matrix, SquareMatrix};

pub struct CollisionEngine;

// As a quick hack, this is merely copy-pasted from physics_component.rs.
// Need to find a better way to deal with this
fn world_inverse_inertia(local_inertia_inv: &Matrix3<f64>, orientation: Quaternion<f64>)
    -> Matrix3<f64> {
    let body_to_world = Matrix3::from(orientation);
    let world_to_body = body_to_world.transpose();
    body_to_world * local_inertia_inv * world_to_body
}

impl CollisionEngine {
    pub fn new() -> CollisionEngine {
        CollisionEngine { }
    }

    pub fn detect_collisions(&self,
        physics_store: &PhysicsComponentStore,
        collision_store: &CollisionComponentStore,
        contacts: &mut ContactCollection)
    {
        contacts.clear_contacts();
        for i in 0 .. collision_store.num_components() {
            for j in (i + 1) .. collision_store.num_components() {
                let entity_i = collision_store.entities()[i];
                let entity_j = collision_store.entities()[j];

                let model_i = collision_store.models()[i];
                let model_j = collision_store.models()[j];

                // TODO: Can't really use unwrap here,
                // as we cannot assume that a physics component actually exists
                // Find a better design to deal with this.
                let phys_id_i = physics_store.lookup_component(&entity_i).unwrap();
                let phys_id_j = physics_store.lookup_component(&entity_j).unwrap();

                let pos_i = physics_store.lookup_position(&phys_id_i);
                let pos_j = physics_store.lookup_position(&phys_id_j);

                let orient_i = physics_store.lookup_orientation(&phys_id_i);
                let orient_j = physics_store.lookup_orientation(&phys_id_j);

                use physics::CollisionModel as Model;
                let possible_contact = match (model_i, model_j) {
                    // Sphere-sphere
                    (Model::Sphere(sphere1_model), Model::Sphere(sphere2_model))
                     => {
                        let sphere_i = Sphere { radius: sphere1_model.radius, center: pos_i };
                        let sphere_j = Sphere { radius: sphere2_model.radius, center: pos_j };
                        contact_sphere_sphere(sphere_i, sphere_j)
                            .map(|data| Contact { 
                                objects: (entity_i, entity_j),
                                physics_components: (phys_id_i, phys_id_j),
                                data: data
                            })
                    },
                    // Cuboid-cuboid
                    (Model::Cuboid(_), Model::Cuboid(_))
                    => {
                        // TODO: Implement Cuboid-cuboid collisions
                        None
                    },
                    // Cuboid-sphere
                    (Model::Sphere(sphere_model), Model::Cuboid(cuboid_model))
                    => {
                        let sphere = Sphere { radius: sphere_model.radius, center: pos_i };
                        let cuboid = Cuboid { half_size: cuboid_model.half_size, rotation: orient_j * cuboid_model.rotation, center: pos_j };

                        // TODO: Fix this. This may be ordering the objects in the wrong way! (I.e. we expect normal to point from i to j etc.)
                        contact_sphere_cuboid(sphere, cuboid)
                            .map(|data| Contact {
                                objects: (entity_i, entity_j),
                                physics_components: (phys_id_i, phys_id_j),
                                data: data
                            })
                    }
                    (Model::Cuboid(cuboid_model), Model::Sphere(sphere_model))
                    => {
                        let cuboid = Cuboid { half_size: cuboid_model.half_size, rotation: orient_i * cuboid_model.rotation, center: pos_i };
                        let sphere = Sphere { radius: sphere_model.radius, center: pos_j };
                        // TODO: Fix this. This may be ordering the objects in the wrong way! (I.e. we expect normal to point from i to j etc.)
                        contact_sphere_cuboid(sphere, cuboid)
                            .map(|data| Contact {
                                objects: (entity_i, entity_j),
                                physics_components: (phys_id_i, phys_id_j),
                                data: data
                            })
                    }
                };

                if let Some(contact) = possible_contact {
                    contacts.push_contact(contact);
                }
            }
        }
    }

    pub fn resolve_collisions(&self,
        physics_store: &mut PhysicsComponentStore,
        contacts: &ContactCollection)
    {
        resolve_interpenetrations(physics_store, contacts);
        resolve_velocities(physics_store, contacts);
    }
}

fn resolve_velocities(
    physics_store: &mut PhysicsComponentStore,
    contacts: &ContactCollection)
{
    let mut view = physics_store.mutable_view();
    for contact in contacts.contacts() {
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

        let (physics1, physics2) = contact.physics_components;
        let orientation1 = view.orientation[physics1];
        let orientation2 = view.orientation[physics2];
        let v1 = view.velocity[physics1];
        let v2 = view.velocity[physics2];
        let m1 = view.mass[physics1];
        let m2 = view.mass[physics2];
        let r1 = contact.data.point - view.position[physics1];
        let r2 = contact.data.point - view.position[physics2];
        let i_inv1 = world_inverse_inertia(&view.inv_inertia_body[physics1], orientation1);
        let i_inv2 = world_inverse_inertia(&view.inv_inertia_body[physics2], orientation2);
        let w1 = i_inv1 * view.angular_momentum[physics1];
        let w2 = i_inv2 * view.angular_momentum[physics2];
        let v_p1 = v1 + w1.cross(r1);
        let v_p2 = v2 + w1.cross(r2);

        // Let n denote the contact normal
        let n = contact.data.normal;

        // Define the "relative velocity" at the point of impact
        let v_r = v_p2 - v_p1;

        // The separating velocity is the projection of the relative velocity
        // onto the contact normal.
        let v_separating = v_r.dot(n);

        // If v_separating is non-negative, the objects are not moving
        // towards each other, and we do not need to add any corrective impulse.
        if v_separating < 0.0 {
            // j_r denotes the relative (reaction) impulse
            let j_r = {
                let linear_denominator = 1.0 / m1 + 1.0 / m2;
                let angular_denominator1 = i_inv1 * r1.cross(n).cross(r1);
                let angular_denominator2 = i_inv2 * r2.cross(n).cross(r2);
                let angular_denominator = (angular_denominator1 + angular_denominator2).dot(n);
                let numerator = -(1.0 + restitution) * v_separating;
                numerator / (linear_denominator + angular_denominator)
            };

            // Compute post-collision velocities
            let v1_post = v1 - j_r / m1 * n;
            let v2_post = v2 + j_r / m2 * n;
            let w1_post = w1 - j_r * i_inv1 * r1.cross(n);
            let w2_post = w2 + j_r * i_inv2 * r2.cross(n);
            view.velocity[physics1] = v1_post;
            view.velocity[physics2] = v2_post;

            // TODO: Avoid the inversions here
            view.angular_momentum[physics1] = i_inv1.invert().unwrap() * w1_post;
            view.angular_momentum[physics2] = i_inv2.invert().unwrap() * w2_post;
        }
    }
}

fn resolve_interpenetrations(
    physics_store: &mut PhysicsComponentStore,
    contacts: &ContactCollection)
{
    let mut view = physics_store.mutable_view();
    for contact in contacts.contacts() {
        let (physics1, physics2) = contact.physics_components;
        let m1 = view.mass[physics1];
        let m2 = view.mass[physics2];
        let total_mass = m1 + m2;

        // Move the two objects linearly away from each other along the contact normal.
        // The distance to move is determined by the relative masses of the two objects,
        // and the penetration depth.
        let obj1_move_dist = (m2 / total_mass) * contact.data.penetration_depth;
        let obj2_move_dist = (m1 / total_mass) * contact.data.penetration_depth;

        // TODO: Implement -= for cgmath Point3?
        view.position[physics1] += - obj1_move_dist * contact.data.normal;
        view.position[physics2] += obj2_move_dist * contact.data.normal;
    }
}