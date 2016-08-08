use glium::{Surface, VertexBuffer, IndexBuffer};
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
    pub fn new(window: &Window) -> SceneRenderer {
        let vertex_shader_src = r#"
            #version 140
            in vec3 pos;
            in vec3 normal;

            uniform mat4 perspective;
            uniform mat4 view;
            uniform mat4 model;

            out vec3 vertex_normal;
            out vec3 vertex_position;

            void main() {
                mat4 modelview = view * model;
                vertex_normal = transpose(inverse(mat3(modelview))) * normal;
                gl_Position = perspective * modelview * vec4(pos, 1.0);
                vertex_position = gl_Position.xyz / gl_Position.w;
            }
        "#;

        let fragment_shader_src = r#"
            #version 140

            in vec3 vertex_normal;
            in vec3 vertex_position;

            out vec4 color;

            uniform vec3 light_direction;

            const vec3 ambient_color = vec3(0.05, 0.0, 0.0);
            const vec3 diffuse_color = vec3(0.6, 0.0, 0.0);
            const vec3 specular_color = vec3(1.0, 1.0, 1.0);

            void main() {
                float diffuse = max(dot(normalize(vertex_normal), normalize(light_direction)), 0.0);

                vec3 camera_dir = normalize(-vertex_position);
                vec3 half_direction = normalize(light_direction + camera_dir);
                float specular = pow(max(dot(half_direction, normalize(vertex_normal)), 0.0), 16.0);

                color = vec4(ambient_color + diffuse * diffuse_color + specular * specular_color, 1.0);
            }
        "#;

        let display = &window.display;
        let program = glium::Program::from_source(display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();

        SceneRenderer {
            program: program,
            camera: Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap()
        }
    }

    pub fn render(&mut self,
        frame: &mut Frame,
        renderable_store: &SceneRenderableStore,
        transform_store: &SceneTransformStore)
    {
        let surface = &mut frame.internal_frame;
        let params = glium::DrawParameters {
            depth: glium::Depth {
                test: glium::draw_parameters::DepthTest::IfLess,
                write: true,
                .. Default::default()
            },
            .. Default::default()
        };

        let view_matrix = self.camera.view_matrix();
        let view: [[f32; 4]; 4] = view_matrix.into();
        let perspective = perspective_matrix(surface);

        // Transform the light direction by the view transform,
        // so that the direction of the light does not change as
        // the camera orientation changes.
        let light_direction: [f32; 3] = {
            let dir4 = view_matrix * Vector4::new(1.0f32, 1.0f32, 0.0, 0.0);
            dir4.truncate().into()
        };

        for (entity, renderable) in renderable_store.renderables().iter() {
            if let Some(transform) = transform_store.lookup(entity) {
                let model = model_matrix(&transform.position);
                let uniforms = uniform! {
                    model: model,
                    view: view,
                    perspective: perspective,
                    light_direction: light_direction
                };

                surface.draw(
                    (&renderable.vertices as &VertexBuffer<RenderVertex>,
                        &renderable.normals as &VertexBuffer<RenderNormal>),
                    &renderable.indices as &IndexBuffer<u32>,
                    &self.program,
                    &uniforms,
                    &params
                ).unwrap();
            }
        }
    }
}
