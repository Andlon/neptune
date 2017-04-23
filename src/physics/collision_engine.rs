use physics::*;
use geometry::{Sphere, Cuboid};
use nalgebra::{Matrix3, UnitQuaternion};
use core::{TransformStore};
use cgmath::{InnerSpace, EuclideanSpace};
use interop;
use entity::LinearComponentStorage;

pub struct CollisionEngine;

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
        CollisionEngine { }
    }

    pub fn detect_collisions(&self,
        transforms: &TransformStore,
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

                let transform_i = transforms.lookup(&entity_i)
                                            .expect("All collision components must have a Transform component.")
                                            .current;
                let transform_j = transforms.lookup(&entity_j)
                                            .expect("All collision components must have a Transform component.")
                                            .current;

                let pos_i = transform_i.position;
                let pos_j = transform_j.position;

                let orient_i = transform_i.orientation;
                let orient_j = transform_j.orientation;

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
                                data: data
                            })
                    },
                    // Cuboid-cuboid
                    (Model::Cuboid(cuboid_i), Model::Cuboid(cuboid_j))
                    => {
                        let cuboid_i = Cuboid {
                            rotation: (orient_i * cuboid_i.rotation).normalize(),
                            center: pos_i + cuboid_i.center.to_vec(),
                            .. cuboid_i
                        };
                        let cuboid_j = Cuboid {
                            rotation: (orient_j * cuboid_j.rotation).normalize(),
                            center: pos_j + cuboid_j.center.to_vec(),
                            .. cuboid_j
                        };
                        contact_cuboid_cuboid(cuboid_i, cuboid_j)
                            .map(|data| Contact {
                                objects: (entity_i, entity_j),
                                data: data
                            })
                    },
                    // Cuboid-sphere
                    (Model::Sphere(sphere), Model::Cuboid(cuboid))
                    => {
                        let sphere = Sphere { radius: sphere.radius, center: pos_i + sphere.center.to_vec() };
                        let cuboid = Cuboid { half_size: cuboid.half_size, rotation: orient_j * cuboid.rotation, center: pos_j + cuboid.center.to_vec() };
                        contact_sphere_cuboid(sphere, cuboid)
                            .map(|data| Contact {
                                objects: (entity_i, entity_j),
                                data: data
                            })
                    },
                    (Model::Cuboid(cuboid), Model::Sphere(sphere))
                    => {
                        let cuboid = Cuboid { half_size: cuboid.half_size, rotation: orient_i * cuboid.rotation, center: pos_i + cuboid.center.to_vec() };
                        let sphere = Sphere { radius: sphere.radius, center: pos_j + sphere.center.to_vec() };
                        contact_sphere_cuboid(sphere, cuboid)
                            .map(|data| Contact {
                                objects: (entity_j, entity_i),
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
        bodies: &mut LinearComponentStorage<RigidBody>,
        transforms: &mut TransformStore,
        contacts: &ContactCollection)
    {
        resolve_velocities(bodies, transforms, contacts);
        resolve_interpenetrations(bodies, transforms, contacts);
    }
}

fn resolve_velocities(
    bodies: &mut LinearComponentStorage<RigidBody>,
    transforms: &TransformStore,
    contacts: &ContactCollection)
{
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

        let (entity1, entity2) = contact.objects;
        let rb1 = bodies.lookup_component_for_entity(entity1).cloned();
        let rb2 = bodies.lookup_component_for_entity(entity2).cloned();
        let transform1 = transforms.lookup(&entity1)
                                   .expect("All Collision components must have a Transform component.")
                                   .current;
        let transform2 = transforms.lookup(&entity2)
                                   .expect("All Collision components must have a Transform component.")
                                   .current;
        if let (Some(mut rb1), Some(mut rb2)) = (rb1, rb2) {
            let orientation1 = UnitQuaternion::new_normalize(
                interop::cgmath_quat_to_nalgebra(&transform1.orientation));
            let orientation2 = UnitQuaternion::new_normalize(
                interop::cgmath_quat_to_nalgebra(&transform2.orientation));
            let v1 = rb1.state.velocity;
            let v2 = rb2.state.velocity;
            let m1 = rb1.mass.value();
            let m2 = rb2.mass.value();
            let r1 = contact.data.point - transform1.position;
            let r2 = contact.data.point - transform2.position;
            let r1 = interop::cgmath_vector3_to_nalgebra(&r1);
            let r2 = interop::cgmath_vector3_to_nalgebra(&r2);
            let i_inv1 = world_inverse_inertia(&rb1.inv_inertia_body, orientation1);
            let i_inv2 = world_inverse_inertia(&rb2.inv_inertia_body, orientation2);
            let w1 = i_inv1 * rb1.state.angular_momentum;
            let w2 = i_inv2 * rb2.state.angular_momentum;
            let v_p1 = v1 + w1.cross(&r1);
            let v_p2 = v2 + w1.cross(&r2);

            // Let n denote the contact normal
            let n = contact.data.normal;
            let n = interop::cgmath_vector3_to_nalgebra(&n);

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

            bodies.set_component_for_entity(entity1, rb1);
            bodies.set_component_for_entity(entity2, rb2);
        }
    }
}

fn resolve_interpenetrations(
    bodies: &mut LinearComponentStorage<RigidBody>,
    transforms: &mut TransformStore,
    contacts: &ContactCollection)
{
    for contact in contacts.contacts() {
        let (entity1, entity2) = contact.objects;
        let rb1 = bodies.lookup_component_for_entity(entity1).cloned();
        let rb2 = bodies.lookup_component_for_entity(entity2).cloned();
        if let (Some(mut rb1), Some(mut rb2)) = (rb1, rb2) {
            let m1 = rb1.mass.value();
            let m2 = rb2.mass.value();
            let total_mass = m1 + m2;

            // Move the two objects linearly away from each other along the contact normal.
            // The distance to move is determined by the relative masses of the two objects,
            // and the penetration depth.
            let obj1_move_dist = (m2 / total_mass) * contact.data.penetration_depth;
            let obj2_move_dist = (m1 / total_mass) * contact.data.penetration_depth;

            rb1.state.position -= interop::cgmath_vector3_to_nalgebra(
                    &(obj1_move_dist * contact.data.normal));
            rb2.state.position += interop::cgmath_vector3_to_nalgebra(
                    &(obj2_move_dist * contact.data.normal));

            // We update the transforms as well here,
            // but this is a stop-gap solution. In fact, we'd
            // like to remove transforms altogether.
            {
                let t1 = transforms.lookup_mut(&entity1).expect("Temporary hack");
                t1.current.position = interop::nalgebra_point3_to_cgmath(
                    &rb1.state.position);
            }

            {
                let t2 = transforms.lookup_mut(&entity2).expect("Temporary hack");
                t2.current.position = interop::nalgebra_point3_to_cgmath(
                    &rb2.state.position);
            }

            bodies.set_component_for_entity(entity1, rb1);
            bodies.set_component_for_entity(entity2, rb2);
        }
    }
}
