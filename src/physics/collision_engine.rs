use physics::*;
use geometry::{OverlapsWith, Sphere, Cuboid};
use message::Message;
use entity::Entity;
use cgmath::{InnerSpace, MetricSpace};

pub struct CollisionEngine;

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
                    (Model::Cuboid(cuboid1), Model::Cuboid(cuboid2))
                    => {
                        // TODO: Implement Cuboid-cuboid collisions
                        None
                    },
                    // Cuboid-sphere
                    (Model::Sphere(sphere_model), Model::Cuboid(cuboid_model))
                    => {
                        let sphere = Sphere { radius: sphere_model.radius, center: pos_i };
                        let cuboid = Cuboid { halfSize: cuboid_model.halfSize, rotation: orient_j * cuboid_model.rotation, center: pos_j };
                        contact_sphere_cuboid(sphere, cuboid)
                            .map(|data| Contact {
                                objects: (entity_i, entity_j),
                                physics_components: (phys_id_i, phys_id_j),
                                data: data
                            })
                    }
                    (Model::Cuboid(cuboid_model), Model::Sphere(sphere_model))
                    => {
                        let cuboid = Cuboid { halfSize: cuboid_model.halfSize, rotation: orient_i * cuboid_model.rotation, center: pos_i };
                        let sphere = Sphere { radius: sphere_model.radius, center: pos_j };
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
        let (physics1, physics2) = contact.physics_components;
        let v1 = view.velocity[physics1];
        let v2 = view.velocity[physics2];
        let m1 = view.mass[physics1];
        let m2 = view.mass[physics2];
        let v_closing = (v1 - v2).dot(contact.data.normal);

        // We only need to apply an impulse if the objects
        // are actually on a collision course
        if v_closing > 0.0 {
            let j_r = (2.0 * v_closing / (1.0 / m1 + 1.0 / m2)) * contact.data.normal;
            let v1_post = v1 - j_r / m1;
            let v2_post = v2 + j_r / m2;
            view.velocity[physics1] = v1_post;
            view.velocity[physics2] = v2_post;
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
        let p1 = view.position[physics1];
        let p2 = view.position[physics2];
        let m1 = view.mass[physics1];
        let m2 = view.mass[physics2];
        let total_mass = m1 + m2;

        // Move the two objects linearly away from each other along the contact normal.
        // The distance to move is determined by the relative masses of the two objects,
        // and the penetration depth.
        let obj1_move_dist = (m1 / total_mass) * contact.data.penetration_depth;
        let obj2_move_dist = (m2 / total_mass) * contact.data.penetration_depth;

        // TODO: Implement -= for cgmath Point3?
        view.position[physics1] += - obj1_move_dist * contact.data.normal;
        view.position[physics2] += obj2_move_dist * contact.data.normal;
    }
}