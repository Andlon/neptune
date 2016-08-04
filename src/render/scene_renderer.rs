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

    let fov: f32 = 3.141592 / 2.0;
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
            in vec3 normal;

            uniform mat4 perspective;
            uniform mat4 view;
            uniform mat4 model;

            out vec3 vertex_normal;

            void main() {
                mat4 modelview = view * model;
                vertex_normal = transpose(inverse(mat3(modelview))) * normal;
                gl_Position = perspective * modelview * vec4(pos, 1.0);
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec3 vertex_normal;

            out vec4 color;

            uniform vec3 light_direction;

            void main() {
                float brightness = dot(normalize(vertex_normal), normalize(light_direction));
                vec3 dark_color = vec3(0.5, 0.0, 0.0);
                vec3 regular_color = vec3(1.0, 0.0, 0.0);
                color = vec4(mix(dark_color, regular_color, brightness), 1.0);
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
                    model: model,
                    view: view,
                    perspective: perspective,
                    light_direction: [1.0f32, 1.0f32, 0.0]
                };

                surface.draw(
                    (&renderable.vertices as &VertexBuffer<RenderVertex>,
                        &renderable.normals as &VertexBuffer<RenderNormal>),
                    &renderable.indices as &IndexBuffer<u32>,
                    &self.program,
                    &uniforms,
                    &Default::default()
                ).unwrap();
            }
        }
    }
}
