use glium::{Surface, VertexBuffer, IndexBuffer};
use glium;
use cgmath::*;
use camera::Camera;
use render::*;
use std::collections::HashMap;
use entity::Entity;
use cgmath;

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

#[derive(Copy, Clone, Debug)]
struct RenderVertex {
    pub pos: [f32; 3]
}

#[derive(Copy, Clone, Debug)]
struct RenderNormal {
    pub normal: [f32; 3]
}

implement_vertex!(RenderVertex, pos);
implement_vertex!(RenderNormal, normal);

struct ComponentBufferData {
    pub vertices: glium::VertexBuffer<RenderVertex>,
    pub normals: glium::VertexBuffer<RenderNormal>,
    pub indices: glium::IndexBuffer<u32>
}

pub struct SceneRenderer {
    program: glium::Program,
    buffer_cache: HashMap<Entity, ComponentBufferData>
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
            program: program,
            buffer_cache: HashMap::new()
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

                let component_data = self.buffer_cache.get(entity)
                                                      .expect("Buffers should have been updated before rendering!");

                surface.draw(
                    (&component_data.vertices as &VertexBuffer<RenderVertex>,
                     &component_data.normals as &VertexBuffer<RenderNormal>),
                    &component_data.indices as &IndexBuffer<u32>,
                    &self.program,
                    &uniforms,
                    &params
                ).unwrap();
            }
        }
    }

    pub fn update_buffers(&mut self, window: &Window, renderable_store: &SceneRenderableStore) {
        // Note: This is a stopgap solution!
        for (entity, renderable) in renderable_store.renderables().iter() {
            if !self.buffer_cache.contains_key(entity) {
                match renderable.render_data {
                    RenderData::Mesh(ref mesh) => {
                        let vertices: Vec<_> = mesh.vertices.iter()
                                                    .map(|v| RenderVertex::from(v))
                                                    .collect();
                        let normals: Vec<_> = mesh.normals.iter()
                                                  .map(|n| RenderNormal::from(n))
                                                  .collect();

                        let vertex_buffer = glium::VertexBuffer::new(&window.display, &vertices).unwrap();
                        let normal_buffer = glium::VertexBuffer::new(&window.display, &normals).unwrap();
                        let index_buffer = glium::IndexBuffer::new(&window.display,
                            glium::index::PrimitiveType::TrianglesList,
                            &mesh.indices).unwrap();

                        self.buffer_cache.insert(entity.clone(), ComponentBufferData {
                            vertices: vertex_buffer,
                            normals: normal_buffer,
                            indices: index_buffer
                        });
                    },
                    _ => ()
                }
            }
        }
    }
}

impl RenderVertex {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RenderVertex {
            pos: [x, y, z]
        }
    }
}

impl<'a> From<&'a cgmath::Point3<f32>> for RenderVertex {
    fn from(point: &'a cgmath::Point3<f32>) -> Self {
        RenderVertex { pos: [ point.x, point.y, point.z ]}
    }
}

impl RenderNormal {
    #[allow(dead_code)]
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        RenderNormal {
            normal: [x, y, z]
        }
    }
}

impl<'a> From<&'a cgmath::Vector3<f32>> for RenderNormal {
    fn from(vector: &'a cgmath::Vector3<f32>) -> Self {
        RenderNormal { normal: [ vector.x, vector.y, vector.z ]}
    }
}