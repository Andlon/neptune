use entity::Entity;
use std::rc::Rc;
use glium::{VertexBuffer, IndexBuffer};
use store::{Identifier, OneToOneStore};

use std::collections::HashMap;
use cgmath;
use render::Color;

#[derive(Copy, Clone, Debug)]
pub struct RenderVertex {
    pub pos: [f32; 3]
}

#[derive(Copy, Clone, Debug)]
pub struct RenderNormal {
    pub normal: [f32; 3]
}

impl RenderVertex {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RenderVertex {
            pos: [x, y, z]
        }
    }
}

impl From<cgmath::Point3<f32>> for RenderVertex {
    fn from(point: cgmath::Point3<f32>) -> Self {
        RenderVertex { pos: [ point.x, point.y, point.z ]}
    }
}

impl RenderNormal {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RenderNormal {
            normal: [x, y, z]
        }
    }
}

impl From<cgmath::Vector3<f32>> for RenderNormal {
    fn from(vector: cgmath::Vector3<f32>) -> Self {
        RenderNormal { normal: [ vector.x, vector.y, vector.z ]}
    }
}

use approx::ApproxEq;

impl ApproxEq for RenderVertex {
    type Epsilon = <f32 as ApproxEq>::Epsilon;

    fn default_epsilon() -> <f32 as ApproxEq>::Epsilon {
        f32::default_epsilon()
    }

    fn default_max_relative() -> <f32 as ApproxEq>::Epsilon {
        f32::default_max_relative()
    }

    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn relative_eq(&self, other: &Self, epsilon: <f32 as ApproxEq>::Epsilon, max_relative: <f32 as ApproxEq>::Epsilon) -> bool {
        self.pos[0].relative_eq(&other.pos[0], epsilon, max_relative)
        && self.pos[1].relative_eq(&other.pos[1], epsilon, max_relative)
        && self.pos[2].relative_eq(&other.pos[2], epsilon, max_relative)
    }

    fn ulps_eq(&self, other: &Self, epsilon: <f32 as ApproxEq>::Epsilon, max_ulps: u32) -> bool {
        self.pos[0].ulps_eq(&other.pos[0], epsilon, max_ulps)
        && self.pos[1].ulps_eq(&other.pos[1], epsilon, max_ulps)
        && self.pos[2].ulps_eq(&other.pos[2], epsilon, max_ulps)
    }
}

impl ApproxEq for RenderNormal {
    type Epsilon = <f32 as ApproxEq>::Epsilon;

    fn default_epsilon() -> <f32 as ApproxEq>::Epsilon {
        f32::default_epsilon()
    }

    fn default_max_relative() -> <f32 as ApproxEq>::Epsilon {
        f32::default_max_relative()
    }

    fn default_max_ulps() -> u32 {
        f32::default_max_ulps()
    }

    fn relative_eq(&self, other: &Self, epsilon: <f32 as ApproxEq>::Epsilon, max_relative: <f32 as ApproxEq>::Epsilon) -> bool {
        self.normal[0].relative_eq(&other.normal[0], epsilon, max_relative)
        && self.normal[1].relative_eq(&other.normal[1], epsilon, max_relative)
        && self.normal[2].relative_eq(&other.normal[2], epsilon, max_relative)
    }

    fn ulps_eq(&self, other: &Self, epsilon: <f32 as ApproxEq>::Epsilon, max_ulps: u32) -> bool {
        self.normal[0].ulps_eq(&other.normal[0], epsilon, max_ulps)
        && self.normal[1].ulps_eq(&other.normal[1], epsilon, max_ulps)
        && self.normal[2].ulps_eq(&other.normal[2], epsilon, max_ulps)
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

    // For now we only have a concept of a single color for the entire
    // renderable
    pub color: Color
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
