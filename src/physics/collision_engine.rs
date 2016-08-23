use physics::{Contact, ContactCollection, CollisionComponentStore, CollisionModel, PhysicsComponentStore};
use geometry::{OverlapsWith, Sphere};
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
                let entity_i = &collision_store.entities()[i];
                let entity_j = &collision_store.entities()[j];

                let model_i = &collision_store.models()[i];
                let model_j = &collision_store.models()[j];

                // TODO: Can't really use unwrap here,
                // as we cannot assume that a physics component actually exists
                // Find a better design to deal with this.
                let phys_id_i = physics_store.lookup_component(entity_i).unwrap();
                let phys_id_j = physics_store.lookup_component(entity_j).unwrap();

                let pos_i = physics_store.lookup_position(&phys_id_i);
                let pos_j = physics_store.lookup_position(&phys_id_j);

                let possible_contact = match (model_i, model_j) {
                    (&CollisionModel::SphereModel { radius: r_i },
                     &CollisionModel::SphereModel { radius: r_j })
                     => {
                        let sphere_i = Sphere { radius: r_i, center: pos_i };
                        let sphere_j = Sphere { radius: r_j, center: pos_j };
                        contact_for_spheres(entity_i, entity_j, sphere_i, sphere_j)
                    }
                };

                if let Some(contact) = possible_contact {
                    contacts.push_contact(contact);
                }
            }
        }
    }
}

fn contact_for_spheres(
        entity1: &Entity, entity2: &Entity,
        sphere1: Sphere<f64>, sphere2: Sphere<f64>)
     -> Option<Contact>
{
    if sphere1.overlaps_with(&sphere2) {
        let r = sphere2.center - sphere1.center;
        let normal = r.normalize();
        // TODO: Implement Sub<Vector3<S>> for Point3<S> in cgmath?
        let point = sphere2.center + (- sphere2.radius * normal);
        let point2 = sphere1.center + sphere1.radius * normal;
        let depth = point.distance(point2);
        Some(Contact {
            objects: (entity1.to_owned(), entity2.to_owned()),
            point: point,
            normal: normal,
            penetration_depth: depth
        })
    } else {
        None
    }
}

mod tests {
    use super::contact_for_spheres;
    use geometry::Sphere;
    use entity::{Entity, EntityManager};
    use cgmath::{Point3, Vector3};

    #[test]
    pub fn contact_for_spheres_no_collision() {
        let mut entity_man = EntityManager::new();
        let entity1 = entity_man.create();
        let entity2 = entity_man.create();

        let sphere1 = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
        let sphere2 = Sphere { radius: 1.0, center: Point3::new(3.0, 0.0, 0.0) };

        let contact = contact_for_spheres(&entity1, &entity2, sphere1, sphere2);
        assert!(contact.is_none());
    }

    #[test]
    pub fn contact_for_spheres_collision() {
        let mut entity_man = EntityManager::new();
        let entity1 = entity_man.create();
        let entity2 = entity_man.create();

        let sphere1 = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
        let sphere2 = Sphere { radius: 1.0, center: Point3::new(1.5, 0.0, 0.0) };

        let contact = contact_for_spheres(&entity1, &entity2, sphere1, sphere2);
        assert!(contact.is_some());

        let contact = contact.unwrap();
        assert_eq!((entity1, entity2), contact.objects);
        assert_eq!(Vector3::new(1.0, 0.0, 0.0), contact.normal);
        assert_eq!(Point3::new(0.5, 0.0, 0.0), contact.point);
        assert_eq!(0.5, contact.penetration_depth);
    }

}