use glium::backend::Facade;
use glium;
use render::*;
use cgmath::*;
use geometry::*;

pub fn weighted_vertex_normals(mesh: &SurfaceMesh<f32>) -> Vec<RenderNormal> {
    let mut vertex_normals: Vec<Vector3<f32>> = Vec::new();
    vertex_normals.resize(mesh.num_vertices(), Vector3::zero());

    let vertices = mesh.vertices();

    for triangle in mesh.triangle_indices() {
        let a_index = triangle.indices[0];
        let b_index = triangle.indices[1];
        let c_index = triangle.indices[2];

        let (a, b, c) = (vertices[a_index], vertices[b_index], vertices[c_index]);
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
        .map(|v| RenderNormal::from(v.clone()))
        .collect()
}

pub fn build_renderable(window: &Window,
    mesh: &SurfaceMesh<f32>,
    normals: &[RenderNormal])
    -> SceneRenderable {

    let display = &window.display;
    let vertices: Vec<RenderVertex> = mesh.vertices().iter()
        .map(|v| RenderVertex::from(v.clone()))
        .collect();
    let indices: Vec<u32> = mesh.triangle_indices().iter()
        .flat_map(|t| t.indices.iter())
        .map(|i| i.clone() as u32)
        .collect();

    let vertex_buffer = glium::VertexBuffer::new(display, &vertices).unwrap();
    let normal_buffer = glium::VertexBuffer::new(display, &normals).unwrap();
    let index_buffer = glium::IndexBuffer::new(display,
        glium::index::PrimitiveType::TrianglesList,
        &indices).unwrap();

    use std::rc::Rc;
    SceneRenderable {
        vertices: Rc::new(vertex_buffer),
        normals: Rc::new(normal_buffer),
        indices: Rc::new(index_buffer),
        color: Color { r: 0.5, g: 0.5, b: 0.5 }
    }
}

pub fn tetrahedron_renderable(window: &Window,
    a: Point3<f32>, b: Point3<f32>, c: Point3<f32>, d: Point3<f32>)
     -> SceneRenderable {

    let mesh = tetrahedron(a, b, c, d).replicate_vertices();
    let normals = weighted_vertex_normals(&mesh);

    build_renderable(window, &mesh, &normals)
}

pub fn icosahedron_renderable(window: &Window) -> SceneRenderable {
    use geometry::icosahedron;
    let mesh = icosahedron().replicate_vertices();
    let normals = weighted_vertex_normals(&mesh);

    build_renderable(window, &mesh, &normals)
}

pub fn unit_sphere_renderable(window: &Window, num_subdivisions: u32)
    -> SceneRenderable {

    let mesh = unit_sphere(num_subdivisions);
    let normals: Vec<RenderNormal> = mesh.vertices().iter()
                                 .map(|v| v.to_vec())
                                 .map(|v| RenderNormal::from(v))
                                 .collect();
    build_renderable(window, &mesh, &normals)
}

pub fn box_renderable(window: &Window, halfx: f32, halfy: f32, halfz: f32)
    -> SceneRenderable {
    let mesh = box_mesh(halfx, halfy, halfz).replicate_vertices();
    let normals = weighted_vertex_normals(&mesh);

    build_renderable(window, &mesh, &normals)
}

#[cfg(test)]
mod tests {

    use super::weighted_vertex_normals;
    use geometry::{SurfaceMesh, TriangleIndices};
    use cgmath::{Point3, Vector3, ApproxEq};
    use render::{RenderVertex, RenderNormal};

    #[test]
    fn weighted_vertex_normals_on_empty_mesh() {
        let mesh = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let normals = weighted_vertex_normals(&mesh);

        assert!(normals.is_empty());
    }

    #[test]
    fn weighted_vertex_normals_on_single_triangle() {
        let vertices = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0)
        ];
        let indices = vec![TriangleIndices::new(0, 1, 2)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();
        let normals = weighted_vertex_normals(&mesh);

        let expected_normal = RenderNormal::new(0.0, 0.0, 1.0);

        assert_eq!(3, normals.len());
        assert_approx_eq!(expected_normal, normals[0]);
        assert_approx_eq!(expected_normal, normals[1]);
        assert_approx_eq!(expected_normal, normals[2]);
    }

    #[test]
    fn weighted_vertex_normals_on_two_triangles_with_repeated_vertices() {
        let vertices = vec![
            // First triangle
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),

            // Second triangle
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(1.0, 0.0, 1.0)
        ];
        let indices = vec![TriangleIndices::new(0, 1, 2), TriangleIndices::new(3, 4, 5)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();
        let normals = weighted_vertex_normals(&mesh);

        let expected_normal1 = RenderNormal::new(0.0, 0.0, 1.0);
        let expected_normal2 = RenderNormal::new(0.0, 1.0, 0.0);

        assert_eq!(6, normals.len());
        assert_approx_eq!(expected_normal1, normals[0]);
        assert_approx_eq!(expected_normal1, normals[1]);
        assert_approx_eq!(expected_normal1, normals[2]);
        assert_approx_eq!(expected_normal2, normals[3]);
        assert_approx_eq!(expected_normal2, normals[4]);
        assert_approx_eq!(expected_normal2, normals[5]);
    }

    // TODO: Test weighted_vertex_normals when vertices are not repeated
}