use entity::Entity;
use value_types::{Vec3, Quaternion};
use std::rc::Rc;
use glium::{VertexBuffer, IndexBuffer};

use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct RenderVertex {
    pub pos: [f32; 3]
}

implement_vertex!(RenderVertex, pos);

// TODO: Ideally we'd like to abstract away glium-specific things
pub struct SceneRenderable {
    pub vertices: Rc<VertexBuffer<RenderVertex>>,
    pub indices: Rc<IndexBuffer<u32>>,

    // Currently we have no concept of local transform,
    // but this is natural to implement later
    // pub localPosition: Vec3<f32>,
    //pub localRotation: Quaternion;
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct SceneRenderableIdentifier {
    id: u32
}

pub struct SceneRenderableStore {
    next_identifier: SceneRenderableIdentifier,
    entity_map: HashMap<Entity, SceneRenderableIdentifier>,
    pub renderables: HashMap<SceneRenderableIdentifier, SceneRenderable>
}

impl SceneRenderableStore {
    pub fn new() -> SceneRenderableStore {
        SceneRenderableStore {
            next_identifier: SceneRenderableIdentifier { id: 0 },
            entity_map: HashMap::new(),
            renderables: HashMap::new()
        }
    }

    pub fn add_renderable(&mut self, entity: Entity, renderable: SceneRenderable)
        -> SceneRenderableIdentifier {
        let identifier = self.next_identifier;
        self.next_identifier = SceneRenderableIdentifier { id: identifier.id + 1 };

        self.entity_map.insert(entity, identifier);
        self.renderables.insert(identifier, renderable);
        identifier
    }
}
