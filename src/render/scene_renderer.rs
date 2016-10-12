use glium::{Surface, VertexBuffer, IndexBuffer};
use glium;
use camera::Camera;
use render::*;
use std::collections::HashMap;
use entity::Entity;
use core::{TransformStore};
use cgmath::{Vector4, InnerSpace, Point3, Vector3};

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
    program: Option<glium::Program>,
    buffer_cache: HashMap<Entity, ComponentBufferData>
}

impl SceneRenderer {
    pub fn new() -> SceneRenderer {
        SceneRenderer {
            program: None,
            buffer_cache: HashMap::new()
        }
    }

    pub fn render(&mut self,
        frame: &mut Frame,
        frame_progress: f64,
        camera: Camera,
        renderable_store: &SceneRenderableStore,
        transform_store: &TransformStore)
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
                let transform = transform.interpolate(frame_progress);
                // TODO: Fix this ugly mess
                let model: [[f64; 4]; 4] = transform.model_matrix().into();
                let model = {
                    let mut new_model: [[f32; 4]; 4] = [[0.0; 4]; 4];
                    for i in 0 .. 4 {
                        for j in 0 .. 4 {
                            new_model[i][j] = model[i][j] as f32;
                        }
                    }
                    new_model
                };

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
                    self.program.as_ref().expect("Shader must be compiled before rendering!"),
                    &uniforms,
                    &params
                ).unwrap();
            }
        }
    }

    pub fn compile_shaders(&mut self, window: &Window) {
        let vertex_shader_src = include_str!("shaders/default_vertex.glsl");
        let fragment_shader_src = include_str!("shaders/default_fragment.glsl");

        let display = &window.display;
        let program = glium::Program::from_source(display,
            vertex_shader_src,
            fragment_shader_src,
            None).unwrap();
        self.program = Some(program);
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

impl<'a> From<&'a Point3<f32>> for RenderVertex {
    fn from(point: &'a Point3<f32>) -> Self {
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

impl<'a> From<&'a Vector3<f32>> for RenderNormal {
    fn from(vector: &'a Vector3<f32>) -> Self {
        RenderNormal { normal: [ vector.x, vector.y, vector.z ]}
    }
}
