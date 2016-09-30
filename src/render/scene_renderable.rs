use entity::Entity;

use std::collections::HashMap;
use cgmath::{Point3, Vector3};
use render::Color;
use geometry::{Sphere, Cuboid};

pub struct MeshRenderable {
    pub vertices: Vec<Point3<f32>>,
    pub normals: Vec<Vector3<f32>>,
    pub indices: Vec<u32>
}

pub enum RenderData {
    Mesh(MeshRenderable),
    Sphere(Sphere<f32>),
    Cuboid(Cuboid<f32>)
}

pub struct SceneRenderable {
    // TODO: Make all data in SceneRenderable private and
    // assumme immutability
    pub render_data: RenderData,

    // For now we only have a concept of a single color for the entire
    // renderable. TODO: Split this into a Material struct
    pub color: Color
}

pub struct SceneRenderableStore {
    renderables: HashMap<Entity, SceneRenderable>
}

impl SceneRenderableStore {
    pub fn new() -> SceneRenderableStore {
        SceneRenderableStore {
            renderables: HashMap::new()
        }
    }

    pub fn set_renderable(&mut self, entity: Entity, renderable: SceneRenderable) {
        self.renderables.insert(entity, renderable);
    }

    pub fn renderables(&self) -> &HashMap<Entity, SceneRenderable> {
        &self.renderables
    }
}
