use glium::backend::Facade;
use glium;
use render::*;
use cgmath::Point3;

pub fn build_renderable<F>(display: &F,
    vertices: &[RenderVertex],
    triangle_indices: &[u32])
    -> SceneRenderable where F: Facade {

    let vertex_buffer = glium::VertexBuffer::new(display, vertices).unwrap();
    let index_buffer = glium::IndexBuffer::new(display,
        glium::index::PrimitiveType::TrianglesList,
        triangle_indices).unwrap();

    use std::rc::Rc;
    SceneRenderable {
        vertices: Rc::new(vertex_buffer),
        indices: Rc::new(index_buffer)
    }
}

pub fn build_triangle_renderable<F>(display: &F, a: Point3<f32>, b: Point3<f32>, c: Point3<f32>)
    -> SceneRenderable where F: Facade {

    let a = RenderVertex::from(a);
    let b = RenderVertex::from(b);
    let c = RenderVertex::from(c);

    let vertices = vec!(a, b, c);
    let indices = [0, 1, 2];
    build_renderable(display, &vertices, &indices)
}

pub fn build_tetrahedron_renderable<F>(display: &F,
    a: Point3<f32>, b: Point3<f32>, c: Point3<f32>, d: Point3<f32>)
     -> SceneRenderable where F: Facade {

    let a = RenderVertex::from(a);
    let b = RenderVertex::from(b);
    let c = RenderVertex::from(c);
    let d = RenderVertex::from(d);

    let vertices = vec!(a, b, c, d);
    let indices = [ 0, 1, 2,
                    0, 1, 3,
                    1, 2, 3,
                    0, 2, 3 ];
    build_renderable(display, &vertices, &indices)
}
