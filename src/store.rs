use std::collections::HashMap;
use std::hash::Hash;
use entity::Entity;

pub trait Identifier : Copy + Hash + Eq + PartialEq {
    fn id(&self) -> u32;
    fn new(id: u32) -> Self;
}

/// A generic store which contains one-to-one mapping between entities and components
pub struct OneToOneStore<ComponentIdentifier, Component>
    where ComponentIdentifier: Identifier
{
    next_identifier: ComponentIdentifier,
    entity_map: HashMap<Entity, ComponentIdentifier>,
    pub components: HashMap<ComponentIdentifier, Component>
}

impl<Id, Component> OneToOneStore<Id, Component>
    where Id: Identifier
{
    pub fn new() -> OneToOneStore<Id, Component> {
        OneToOneStore {
            next_identifier: Id::new(0),
            entity_map: HashMap::new(),
            components: HashMap::new(),
        }
    }

    pub fn add_component(&mut self, entity: Entity, component: Component)
        -> Id
    {
        let identifier = self.next_identifier;
        self.next_identifier = Id::new(identifier.id() + 1);

        self.entity_map.insert(entity, identifier);
        self.components.insert(identifier, component);
        identifier
    }
}
