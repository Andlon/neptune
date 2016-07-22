use entity::Entity;
use std::rc::Rc;
use glium::{VertexBuffer, IndexBuffer};
use store::{Identifier, OneToOneStore};

use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct RenderVertex {
    pub pos: [f32; 3]
}

implement_vertex!(RenderVertex, pos);

// TODO: Ideally we'd like to abstract away glium-specific things
pub struct SceneRenderable {
    // TODO: Should probably just store vertices directly here,
    // and then let the renderer cache vertices in GPU buffers
    // as it sees fit.
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

impl Identifier for SceneRenderableIdentifier {
    fn new(id: u32) -> Self { SceneRenderableIdentifier { id: id } }
    fn id(&self) -> u32 { self.id }
}

pub struct SceneRenderableStore {
    store: OneToOneStore<SceneRenderable>,
}

impl SceneRenderableStore {
    pub fn new() -> SceneRenderableStore {
        SceneRenderableStore {
            store: OneToOneStore::new()
        }
    }

    pub fn set_renderable(&mut self, entity: Entity, renderable: SceneRenderable) {
        self.store.set_component(entity, renderable)
    }

    pub fn renderables(&self) -> &HashMap<Entity, SceneRenderable> {
        &self.store.components
    }
}
