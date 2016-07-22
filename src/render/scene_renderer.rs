use glium::{Surface, VertexBuffer, IndexBuffer};
use glium::backend::Facade;
use glium;
use value_types::Vec3;
use render::*;

fn perspective_matrix<S: Surface>(surface: &S) -> [[f32; 4]; 4] {
    let (width, height) = surface.get_dimensions();
    let aspect_ratio = height as f32 / width as f32;

    let fov: f32 = 3.141592 / 3.0;
    let zfar = 1024.0;
    let znear = 0.1;

    let f = 1.0 / (fov / 2.0).tan();

    [
        [f *   aspect_ratio   ,    0.0,              0.0              ,   0.0],
        [         0.0         ,     f ,              0.0              ,   0.0],
        [         0.0         ,    0.0,  (zfar+znear)/(zfar-znear)    ,   1.0],
        [         0.0         ,    0.0, -(2.0*zfar*znear)/(zfar-znear),   0.0],
    ]
}

//fn view_matrix(position: &[f32; 3], direction: &[f32; 3], up: &[f32; 3]) -> [[f32; 4]; 4] {
fn view_matrix(position: &Vec3<f32>, direction: &Vec3<f32>, up: &Vec3<f32>) -> [[f32; 4]; 4] {
    let up = [up.x, up.y, up.z];
    let position = [position.x, position.y, position.z];
    let direction = [direction.x, direction.y, direction.z];

    let f = {
        let f = direction;
        let len = f[0] * f[0] + f[1] * f[1] + f[2] * f[2];
        let len = len.sqrt();
        [f[0] / len, f[1] / len, f[2] / len]
    };

    let s = [up[1] * f[2] - up[2] * f[1],
             up[2] * f[0] - up[0] * f[2],
             up[0] * f[1] - up[1] * f[0]];

    let s_norm = {
        let len = s[0] * s[0] + s[1] * s[1] + s[2] * s[2];
        let len = len.sqrt();
        [s[0] / len, s[1] / len, s[2] / len]
    };

    let u = [f[1] * s_norm[2] - f[2] * s_norm[1],
             f[2] * s_norm[0] - f[0] * s_norm[2],
             f[0] * s_norm[1] - f[1] * s_norm[0]];

    let p = [-position[0] * s_norm[0] - position[1] * s_norm[1] - position[2] * s_norm[2],
             -position[0] * u[0] - position[1] * u[1] - position[2] * u[2],
             -position[0] * f[0] - position[1] * f[1] - position[2] * f[2]];

    [
        [s[0], u[0], f[0], 0.0],
        [s[1], u[1], f[1], 0.0],
        [s[2], u[2], f[2], 0.0],
        [p[0], p[1], p[2], 1.0],
    ]
}

fn model_matrix(position: &Vec3<f32>) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [position.x, position.y, position.z, 1.0f32]
    ]
}

#[derive(Copy, Clone)]
pub struct Camera {
    pub pos: Vec3<f32>,
    pub direction: Vec3<f32>,
    pub up:  Vec3<f32>,
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
            camera: Camera {
                pos: Vec3 { x: 0.0, y: 0.0, z: 0.0f32 },
                direction: Vec3 { x: 0.0, y: 1.0, z: 0.0f32 },
                up:  Vec3 { x: 0.0, y: 0.0, z: 1.0f32 }
            }
        }
    }

    pub fn render<S: Surface>(&mut self,
        renderable_store: &SceneRenderableStore,
        transform_store: &SceneTransformStore,
        surface: &mut S)
    {
        let camera_pos = &self.camera.pos;
        let camera_direction = &self.camera.direction;
        let up = &self.camera.up;

        let view = view_matrix(&camera_pos, &camera_direction, &up);
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
