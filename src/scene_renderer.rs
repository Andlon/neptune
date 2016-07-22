use value_types::Vec3;
use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::backend::Facade;
use glium;
use ecs::Entity;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Copy, Clone)]
pub struct RenderVertex {
    pub pos: [f32; 3]
}

implement_vertex!(RenderVertex, pos);

pub struct SceneRenderable {
    pub vertices: Rc<VertexBuffer<RenderVertex>>,
    pub indices: Rc<IndexBuffer<u32>>
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct SceneRenderableIdentifier {
    id: u32
}

struct RenderableWithPosition {
    position: Vec3,
    renderable: SceneRenderable
}

pub struct SceneRenderer {
    next_identifier: SceneRenderableIdentifier,
    entity_map: HashMap<Entity, Vec<SceneRenderableIdentifier>>,
    renderables: HashMap<SceneRenderableIdentifier, RenderableWithPosition>,
    program: glium::Program
}

impl SceneRenderer {
    pub fn new<F>(display: &F) -> SceneRenderer where F: Facade {
        let vertex_shader_src = r#"
            #version 140
            in vec3 pos;
            void main() {
                gl_Position = vec4(pos, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140
            out vec4 color;
            void main() {
                color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        "#;

        let program = glium::Program::from_source(display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();

        SceneRenderer {
            next_identifier: SceneRenderableIdentifier { id: 0 },
            entity_map: HashMap::new(),
            renderables: HashMap::new(),
            program: program
        }
    }

    pub fn render<S: Surface>(&mut self, surface: &mut S) {
        for renderable_with_pos in self.renderables.values_mut() {
            let pos = &renderable_with_pos.position;
            let renderable = &renderable_with_pos.renderable;

            surface.draw(
                &renderable.vertices as &VertexBuffer<RenderVertex>,
                &renderable.indices as &IndexBuffer<u32>,
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &Default::default()
            ).unwrap();
        }
    }

    pub fn add_renderable(&mut self, entity: Entity, renderable: SceneRenderable)
        -> SceneRenderableIdentifier {
        let identifier = self.next_identifier;
        self.next_identifier = SceneRenderableIdentifier { id: identifier.id + 1 };

        let scene_entities = self.entity_map.entry(entity).or_insert_with(|| Vec::new());
        scene_entities.push(identifier);

        let renderable_with_pos = RenderableWithPosition {
            position: Vec3 { x: 0.0, y: 0.0, z: 0.0 },
            renderable: renderable
        };

        self.renderables.insert(identifier, renderable_with_pos);
        identifier
    }
}
