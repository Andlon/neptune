use geometry::{SurfaceMesh, TriangleIndices};
use cgmath::Point3;

/// Takes a mesh and constructs a new geometrically equivalent mesh
/// where each vertex position is repeated such that every vertex is associated
/// with exactly one triangle.
pub fn replicate_vertices(mesh: &SurfaceMesh<f32>) -> SurfaceMesh<f32> {
    let vertices: Vec<Point3<f32>> = mesh.triangles().iter()
        .flat_map(|triangle| {
            triangle.indices.iter()
                    .map(|index| mesh.vertices()[index.clone()])
        }).collect();

    let triangle_indices: Vec<TriangleIndices> = (0 .. mesh.num_triangles())
        .map(|triangle_index| {
            let first_vertex = 3 * triangle_index as usize;
            TriangleIndices::new(first_vertex, first_vertex + 1, first_vertex + 2)
        }).collect();

    SurfaceMesh::from_indices(vertices, triangle_indices)
        .expect("Returned mesh should always be valid since it starts with a valid mesh.")
}

#[cfg(test)]
mod tests {
    use super::replicate_vertices;
    use geometry::{SurfaceMesh, TriangleIndices};

    #[test]
    fn replicate_vertices_on_empty_mesh() {
        let mesh: SurfaceMesh<f32> = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let replicated = replicate_vertices(&mesh);

        assert!(replicated.vertices().is_empty());
        assert!(replicated.triangles().is_empty());
    }

    #[test]
    fn replicate_vertices_on_single_triangle() {
        use cgmath::Point3;

        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(0.0, 0.0, 1.0);

        let vertices = vec![a, b, c];
        let indices = vec![TriangleIndices::new(0, 1, 2)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let replicated = replicate_vertices(&mesh);

        assert_eq!(mesh, replicated);
    }

    #[test]
    fn replicate_vertices_on_two_triangles() {
        use cgmath::Point3;

        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(1.0, 0.0, 0.0);
        let d = Point3::new(1.0, 1.0, 0.0);

        let vertices = vec![a, b, c, d];
        let indices = vec![TriangleIndices::new(0, 1, 2), TriangleIndices::new(1, 2, 3)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let expected_vertices = vec![a, b, c, b, c, d];
        let expected_indices = vec![TriangleIndices::new(0, 1, 2), TriangleIndices::new(3, 4, 5)];
        let expected_mesh = SurfaceMesh::from_indices(expected_vertices, expected_indices).unwrap();

        let replicated = replicate_vertices(&mesh);

        assert_eq!(expected_mesh, replicated);
    }
}
