use entity::Entity;
use std::rc::Rc;
use glium::{VertexBuffer, IndexBuffer};
use store::{Identifier, OneToOneStore};

use std::collections::HashMap;
use cgmath;

#[derive(Copy, Clone)]
pub struct RenderVertex {
    pub pos: [f32; 3]
}

#[derive(Copy, Clone)]
pub struct RenderNormal {
    pub normal: [f32; 3]
}

impl From<cgmath::Point3<f32>> for RenderVertex {
    fn from(point: cgmath::Point3<f32>) -> Self {
        RenderVertex { pos: [ point.x, point.y, point.z ]}
    }
}

impl From<cgmath::Vector3<f32>> for RenderNormal {
    fn from(vector: cgmath::Vector3<f32>) -> Self {
        RenderNormal { normal: [ vector.x, vector.y, vector.z ]}
    }
}

implement_vertex!(RenderVertex, pos);
implement_vertex!(RenderNormal, normal);

// TODO: Ideally we'd like to abstract away glium-specific things
pub struct SceneRenderable {
    // TODO: Should probably just store vertices directly here,
    // and then let the renderer cache vertices in GPU buffers
    // as it sees fit.
    pub vertices: Rc<VertexBuffer<RenderVertex>>,
    pub normals: Rc<VertexBuffer<RenderNormal>>,
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
