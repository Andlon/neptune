use std::collections::HashMap;
use entity::Entity;

pub struct LinearComponentStorage<C> {
    components: Vec<(C, Entity)>,
    entity_map: HashMap<Entity, usize>
}

impl<C> LinearComponentStorage<C> {
    pub fn new() -> Self {
        LinearComponentStorage {
            components: Vec::new(),
            entity_map: HashMap::new()
        }
    }

    pub fn num_components(&self) -> usize {
        self.components.len()
    }

    pub fn set_component_for_entity<'a>(&'a mut self, entity: Entity, component: C)
    {
        let next_available_index = self.components.len();
        let index: usize = self.entity_map.entry(entity).or_insert(next_available_index).clone();
        if index >= self.components.len() {
            self.components.push((component, entity));
        } else {
            self.components[index] = (component, entity);
        }
    }

    pub fn lookup_component_for_entity<'a>(&'a self, entity: Entity)
        -> Option<&'a C>
    {
        if let Some(index) = self.entity_map.get(&entity).cloned() {
            self.components.get(index).map(|&(ref component, e)| {
                debug_assert!(e == entity);
                component
            })
        } else {
            None
        }
    }

    pub fn lookup_component_for_entity_mut<'a>(&'a mut self, entity: Entity)
        -> Option<&'a mut C>
    {
        if let Some(index) = self.entity_map.get(&entity).cloned() {
            self.components.get_mut(index).map(|&mut (ref mut component, e)| {
                debug_assert!(e == entity);
                component
            })
        } else {
            None
        }
    }

    pub fn components<'a>(&'a self) -> &'a [(C, Entity)] {
        &self.components
    }

    pub fn components_mut<'a>(&'a mut self) -> &'a mut [(C, Entity)] {
        self.components.as_mut_slice()
    }

    pub fn clear(&mut self) {
        self.components.clear();
        self.entity_map.clear();
    }
}
