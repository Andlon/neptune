use physics::{CollisionComponentStore, CollisionModel, PhysicsComponentStore};
use geometry::{OverlapsWith, Sphere};
use message::Message;

pub struct CollisionEngine {

}

impl CollisionEngine {
    pub fn new() -> CollisionEngine {
        CollisionEngine { }
    }

    pub fn detect_collisions(&self,
        physics_store: &PhysicsComponentStore,
        collision_store: &CollisionComponentStore)
        -> Vec<Message>
    {
        let mut messages = Vec::new();

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

                type Sphere = CollisionModel::SphereModel;

                let collides = match (model_i, model_j) {
                    (&CollisionModel::SphereModel { radius: r_i },
                     &CollisionModel::SphereModel { radius: r_j })
                     => {
                        let sphere_i = Sphere { radius: r_i, center: pos_i };
                        let sphere_j = Sphere { radius: r_j, center: pos_j };
                        sphere_i.overlaps_with(&sphere_j)
                    }
                };

                if collides {
                    let entity1 = collision_store.entities()[i];
                    let entity2 = collision_store.entities()[j];
                    messages.push(Message::CollisionDetected(entity1, entity2));
                }
            }
        }

        messages
    }
}