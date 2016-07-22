use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::backend::Facade;
use glium;
use render::*;

pub struct SceneRenderer {
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
            program: program
        }
    }

    pub fn render<S: Surface>(&mut self, store: &SceneRenderableStore, surface: &mut S) {
        for renderable in store.renderables.values() {
            surface.draw(
                &renderable.vertices as &VertexBuffer<RenderVertex>,
                &renderable.indices as &IndexBuffer<u32>,
                &self.program,
                &glium::uniforms::EmptyUniforms,
                &Default::default()
            ).unwrap();
        }
    }
}
