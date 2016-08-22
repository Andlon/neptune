use std::collections::HashSet;

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
pub struct Entity {
    id: u32,
}

pub struct EntityManager {
    next: Entity,
    entities: HashSet<Entity>
}

impl EntityManager {
    pub fn create(&mut self) -> Entity {
        // TODO: Make manager reuse entity IDs
        let entity = self.next;
        self.next = Entity { id: entity.id + 1 };
        self.entities.insert(entity);
        entity
    }

    pub fn alive(&self, entity: &Entity) -> bool {
        self.entities.contains(entity)
    }

    pub fn destroy(&mut self, entity: &Entity) -> bool {
        self.entities.remove(&entity)
    }

    pub fn new() -> EntityManager {
        EntityManager {
            next: Entity { id: 0 },
            entities: HashSet::new()
        }
    }
}

#[test]
fn identity_manager_create_counts_sequentially() {
    let mut ent_man = EntityManager::new();
    assert!(ent_man.create().id == 0);
    assert!(ent_man.create().id == 1);
    assert!(ent_man.create().id == 2);
}

#[test]
fn identity_manager_destroy_kills_entities() {
    let mut ent_man = EntityManager::new();
    let entities = (0..3).map(|_| ent_man.create()).collect::<Vec<Entity>>();
    ent_man.destroy(&entities[1]);

    assert_eq!(ent_man.alive(&entities[0]), true);
    assert_eq!(ent_man.alive(&entities[1]), false);
    assert_eq!(ent_man.alive(&entities[2]), true);
}
