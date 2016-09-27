use std::collections::HashMap;
use std::hash::Hash;
use entity::Entity;

pub trait Identifier : Copy + Hash + Eq + PartialEq {
    fn id(&self) -> u32;
    fn new(id: u32) -> Self;
}

/// A generic store which contains one-to-one mappings between entities and components
pub struct OneToOneStore<Component> {
    pub components: HashMap<Entity, Component>
}

impl<Component> OneToOneStore<Component>
{
    pub fn new() -> OneToOneStore<Component> {
        OneToOneStore {
            components: HashMap::new(),
        }
    }

    pub fn set_component(&mut self, entity: Entity, component: Component)
    {
        self.components.insert(entity, component);
    }

    pub fn lookup(&self, entity: &Entity) -> Option<&Component> {
        self.components.get(entity)
    }
}
