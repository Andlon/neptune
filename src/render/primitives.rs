use glium::backend::Facade;
use glium;
use render::*;
use cgmath::*;



pub fn weighted_vertex_normals(vertices: &[RenderVertex], triangle_indices: &[u32]) -> Vec<RenderNormal> {
    // TODO: Implement a more formal SurfaceMesh struct to hold surface triangulations.
    assert!(triangle_indices.len() % 3 == 0);

    let num_triangles = triangle_indices.len() / 3;

    let mut vertex_normals: Vec<Vector3<f32>> = Vec::new();
    vertex_normals.resize(num_triangles, Vector3::zero());

    for k in 0..num_triangles {
        // Vertex indices
        let a_index = triangle_indices[3 * k] as usize;
        let b_index = triangle_indices[3 * k + 1] as usize;
        let c_index = triangle_indices[3 * k + 2] as usize;

        // Convert RenderVertices into Point3 instances
        let (a, b, c) = (vertices[a_index], vertices[b_index], vertices[c_index]);
        let (a, b, c) = (Point3::from(a.pos), Point3::from(b.pos), Point3::from(c.pos));

        let ab = b - a;
        let ac = c - a;

        // Note that the normal here is implicitly weighted with the
        // measure/area of the triangle, so that after final normalization,
        // the neighboring triangles whose area is greater is weighted
        // more in the resulting vertex normal.
        let normal = ab.cross(ac);
        vertex_normals[a_index] += normal;
        vertex_normals[b_index] += normal;
        vertex_normals[c_index] += normal;
    }

    vertex_normals.iter()
        .map(|v| v.normalize())
        .map(|v| RenderNormal::from(v))
        .collect()
}

pub fn build_renderable<F>(display: &F,
    vertices: &[RenderVertex],
    normals: &[RenderNormal],
    triangle_indices: &[u32],)
    -> SceneRenderable where F: Facade {

    let vertex_buffer = glium::VertexBuffer::new(display, vertices).unwrap();
    let normal_buffer = glium::VertexBuffer::new(display, normals).unwrap();
    let index_buffer = glium::IndexBuffer::new(display,
        glium::index::PrimitiveType::TrianglesList,
        triangle_indices).unwrap();

    use std::rc::Rc;
    SceneRenderable {
        vertices: Rc::new(vertex_buffer),
        normals: Rc::new(normal_buffer),
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
    let normals = weighted_vertex_normals(&vertices, &indices);
    build_renderable(display, &vertices, &normals, &indices)
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
    let normals = weighted_vertex_normals(&vertices, &indices);
    build_renderable(display, &vertices, &normals, &indices)
}
