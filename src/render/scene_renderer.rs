use glium::{Surface, VertexBuffer, IndexBuffer};
use glium;
use cgmath::*;
use camera::Camera;
use render::*;

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

pub struct SceneRenderer {
    program: glium::Program,
}

impl SceneRenderer {
    pub fn new(window: &Window) -> SceneRenderer {
        let vertex_shader_src = r#"
            #version 330
            in vec3 pos;
            in vec3 normal;

            uniform mat4 perspective;
            uniform mat4 view;
            uniform mat4 model;

            out vec3 vertex_normal;
            out vec3 vertex_position;

            void main() {
                mat4 modelview = view * model;
                gl_Position = perspective * modelview * vec4(pos, 1.0);
                vertex_normal = transpose(inverse(mat3(modelview))) * normal;
                vertex_position = vec3(modelview * vec4(pos, 1.0));
            }
        "#;

        let fragment_shader_src = r#"
            #version 330

            in vec3 vertex_normal;
            in vec3 vertex_position;

            out vec4 color;

            uniform vec3 light_direction;
            uniform vec3 diffuse_color;

            const vec3 u_ambient_intensity = vec3(0.05, 0.05, 0.05);
            const vec3 u_diffuse_intensity = vec3(0.6, 0.6, 0.6);
            const vec3 u_specular_intensity = vec3(0.90, 0.90, 0.90);
            const float shininess = 32.0;

            vec3 specular_lighting(vec3 v_normal, vec3 light_dir, vec3 camera_dir) {
                vec3 half_direction = normalize(light_dir + camera_dir);

                float specular_weight = 0;
                if (dot(v_normal, light_dir) > 0) {
                    specular_weight = pow(max(dot(half_direction, v_normal), 0.0), shininess);
                }

                return specular_weight * u_specular_intensity;
            }

            vec3 diffuse_lighting(vec3 v_normal, vec3 light_dir) {
                float diffuse_weight = max(dot(v_normal, light_dir), 0.0);
                return diffuse_weight * u_diffuse_intensity * diffuse_color;
            }

            vec3 ambient_lighting() {
                return u_ambient_intensity * u_diffuse_intensity * diffuse_color;
            }

            void main() {
                vec3 v_normal = normalize(vertex_normal);
                vec3 l_dir = normalize(light_direction);
                vec3 camera_dir = normalize(-vertex_position);

                vec3 ambient = ambient_lighting();
                vec3 diffuse = diffuse_lighting(v_normal, l_dir);
                vec3 specular = specular_lighting(v_normal, l_dir, camera_dir);

                color = vec4(ambient + diffuse + specular, 1.0);
            }
        "#;

        let display = &window.display;
        let program = glium::Program::from_source(display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();

        SceneRenderer {
            program: program
        }
    }

    pub fn render(&mut self,
        frame: &mut Frame,
        camera: Camera,
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

        let view_matrix = camera.view_matrix();
        let view: [[f32; 4]; 4] = view_matrix.into();
        let perspective = perspective_matrix(surface);

        // Transform the light direction by the view transform,
        // so that the direction of the light does not change as
        // the camera orientation changes.
        let light_direction: [f32; 3] = {
            let dir4 = view_matrix * Vector4::new(1.0f32, 1.5f32, 0.2, 0.0).normalize();
            dir4.truncate().into()
        };

        for (entity, renderable) in renderable_store.renderables().iter() {
            if let Some(transform) = transform_store.lookup(entity) {
                let model: [[f32; 4]; 4] = transform.model_matrix().into();
                let uniforms = uniform! {
                    model: model,
                    view: view,
                    perspective: perspective,
                    light_direction: light_direction,
                    diffuse_color: renderable.color
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