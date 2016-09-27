use geometry::{SurfaceMesh, TriangleIndices};
use cgmath::*;

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

pub fn box_mesh(halfx: f32, halfy: f32, halfz: f32) -> SurfaceMesh<f32> {
    assert!(halfx > 0.0);
    assert!(halfy > 0.0);
    assert!(halfz > 0.0);

    let vertices: Vec<Point3<f32>> = vec![
        // Negative y
        Point3::new(-halfx, -halfy, halfz),  // 0, top-south-west
        Point3::new(-halfx, -halfy, -halfz), // 1, bottom-south-west
        Point3::new(halfx, -halfy, -halfz),  // 2, bottom-south-east
        Point3::new(halfx, -halfy, halfz),   // 3, top-south-east

        // Positive y
        Point3::new(-halfx, halfy, halfz),   // 4, top-north-west
        Point3::new(-halfx, halfy, -halfz),  // 5, bottom-north-west
        Point3::new(halfx, halfy, -halfz),   // 6, bottom-north-east
        Point3::new(halfx, halfy, halfz),    // 7, top-north-east
    ];

    // TODO: Could us a more systematic pattern for the below,
    // but it shouldn't really matter in the end.
    let indices = vec![
        // Southern face
        TriangleIndices::new(0, 1, 2),
        TriangleIndices::new(2, 3, 0),

        // Eastern face
        TriangleIndices::new(3, 2, 7),
        TriangleIndices::new(2, 6, 7),

        // Northern face
        TriangleIndices::new(5, 4, 7),
        TriangleIndices::new(7, 6, 5),

        // Western face
        TriangleIndices::new(1, 0, 4),
        TriangleIndices::new(4, 5, 1),

        // Bottom face
        TriangleIndices::new(6, 2, 1),
        TriangleIndices::new(6, 1, 5),

        // Top face
        TriangleIndices::new(0, 3, 7),
        TriangleIndices::new(7, 4, 0)
    ];

    SurfaceMesh::from_indices(vertices, indices)
        .expect("The mesh generated should always be valid.")
}
