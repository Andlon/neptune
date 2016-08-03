use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::backend::Facade;
use glium;
use render::*;

use cgmath::*;

fn perspective_matrix<S: Surface>(surface: &S) -> [[f32; 4]; 4] {
    // TODO: Move this into Camera, so that we can
    // adjust FOV etc. through adjusting the Camera's properties
    let (width, height) = surface.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f *   aspect_ratio   ,    0.0,              0.0              ,    0.0],
        [         0.0         ,     f ,              0.0              ,    0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(znear-zfar)    ,   -1.0],
        [         0.0         ,    0.0, (2.0*zfar*znear)/(znear-zfar) ,    0.0],
    ]
}

fn model_matrix(position: &Point3<f32>) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [position.x, position.y, position.z, 1.0f32]
    ]
}

pub struct SceneRenderer {
    program: glium::Program,
    pub camera: Camera,
}

impl SceneRenderer {
    pub fn new<F>(display: &F) -> SceneRenderer where F: Facade {
        let vertex_shader_src = r#"
            #version 140
            in vec3 pos;

            uniform mat4 perspective;
            uniform mat4 view;
            uniform mat4 model;

            void main() {
                gl_Position = perspective * view * model * vec4(pos, 1.0);
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
            program: program,
            camera: Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap()
        }
    }

    pub fn render<S: Surface>(&mut self,
        renderable_store: &SceneRenderableStore,
        transform_store: &SceneTransformStore,
        surface: &mut S)
    {

        let view: [[f32; 4]; 4] = self.camera.view_matrix().into();
        let perspective = perspective_matrix(surface);

        for (entity, renderable) in renderable_store.renderables().iter() {
            if let Some(transform) = transform_store.lookup(entity) {
                let model = model_matrix(&transform.position);
                let uniforms = uniform! {
                    model: model, view: view, perspective: perspective
                };

                surface.draw(
                    &renderable.vertices as &VertexBuffer<RenderVertex>,
                    &renderable.indices as &IndexBuffer<u32>,
                    &self.program,
                    &uniforms,
                    &Default::default()
                ).unwrap();
            }
        }
    }
}
