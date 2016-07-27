use glium::backend::Facade;
use glium;
use render::*;
use cgmath::Point3;

pub fn build_triangle_renderable<F>(display: &F, a: Point3<f32>, b: Point3<f32>, c: Point3<f32>)
    -> SceneRenderable where F: Facade {

    let a = RenderVertex::from(a);
    let b = RenderVertex::from(b);
    let c = RenderVertex::from(c);

    let vertices = vec!(a, b, c);

    let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
    let indices = glium::IndexBuffer::new(display,
        glium::index::PrimitiveType::TrianglesList,
        &[0, 1, 2]).unwrap();

    use std::rc::Rc;
    SceneRenderable {
        vertices: Rc::new(vertex_buffer),
        indices: Rc::new(indices)
    }
}

pub fn build_tetrahedron_renderable<F>(display: &F) -> SceneRenderable where F: Facade {
    unimplemented!();
}
