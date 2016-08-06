use geometry::{SurfaceMesh, TriangleIndices};
use cgmath::*;

pub fn tetrahedron<S>(a: Point3<S>, b: Point3<S>, c: Point3<S>, d: Point3<S>) -> SurfaceMesh<S>
    where S: BaseNum
{
    let vertices = vec![a, b, c, d];
    let (a, b, c, d) = (0, 1, 2, 3);
    let indices = vec![
        TriangleIndices::new(c, b, a),
        TriangleIndices::new(a, b, d),
        TriangleIndices::new(a, d, c),
        TriangleIndices::new(b, c, d)
    ];

    SurfaceMesh::from_indices(vertices, indices)
        .expect("Triangle indices should all be valid.")
}

pub fn icosahedron() -> SurfaceMesh<f32> {
    // Let phi be the golden ratio
    let phi: f32 = (1.0 + (5.0 as f32).sqrt()) / 2.0;

    // The vertex coordinates are given by the cyclic permutations of (0, +-1, +- phi)
    // Here we have used the ordering of the vertices and the triangle indexing
    // provided by Andreas Kahler on his blog:
    // http://blog.andreaskahler.com/2009/06/creating-icosphere-mesh-in-code.html

    let vertices: Vec<Point3<f32>> = vec![
        Point3::new(-1.0, phi, 0.0),
        Point3::new(1.0, phi, 0.0),
        Point3::new(-1.0, -phi, 0.0),
        Point3::new(1.0, -phi, 0.0),

        Point3::new(0.0, -1.0, phi),
        Point3::new(0.0, 1.0, phi),
        Point3::new(0.0, -1.0, -phi),
        Point3::new(0.0, 1.0, -phi),

        Point3::new(phi, 0.0, -1.0),
        Point3::new(phi, 0.0, 1.0),
        Point3::new(-phi, 0.0, -1.0),
        Point3::new(-phi, 0.0, 1.0)
    ];

    let indices = vec![
        TriangleIndices::new(0, 11, 5),
        TriangleIndices::new(0, 5, 1),
        TriangleIndices::new(0, 1, 7),
        TriangleIndices::new(0, 7, 10),
        TriangleIndices::new(0, 10, 11),

        TriangleIndices::new(1, 5, 9),
        TriangleIndices::new(5, 11, 4),
        TriangleIndices::new(11, 10, 2),
        TriangleIndices::new(10, 7, 6),
        TriangleIndices::new(7, 1, 8),

        TriangleIndices::new(3, 9, 4),
        TriangleIndices::new(3, 4, 2),
        TriangleIndices::new(3, 2, 6),
        TriangleIndices::new(3, 6, 8),
        TriangleIndices::new(3, 8, 9),

        TriangleIndices::new(4, 9, 5),
        TriangleIndices::new(2, 4, 11),
        TriangleIndices::new(6, 2, 10),
        TriangleIndices::new(8, 6, 7),
        TriangleIndices::new(9, 8, 1)
    ];

    SurfaceMesh::from_indices(vertices, indices)
        .expect("Triangle indices should all be valid.")
}

pub fn unit_sphere(num_subdivisions: u32) -> SurfaceMesh<f32> {
    // Generate an icosphere
    let mesh = icosahedron().subdivide(num_subdivisions);

    // Normalize each vertex such that it lies on the unit sphere
    let normalized_vertices: Vec<Point3<f32>> = mesh.vertices().iter()
                                 .map(|v| Point3::from_vec(v.to_vec().normalize()))
                                 .collect();

    SurfaceMesh::from_indices(normalized_vertices, Vec::from(mesh.triangle_indices()))
        .expect("Triangle indices should all be valid")
}
