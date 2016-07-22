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
}

/// A generic store which contains one-to-many mappings between entities and components
pub struct OneToManyStore<ComponentIdentifier, Component>
    where ComponentIdentifier: Identifier
{
    next_identifier: ComponentIdentifier,
    entity_map: HashMap<Entity, ComponentIdentifier>,
    pub components: HashMap<ComponentIdentifier, Component>
}

impl<Id, Component> OneToManyStore<Id, Component>
    where Id: Identifier
{
    pub fn new() -> OneToManyStore<Id, Component> {
        OneToManyStore {
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
