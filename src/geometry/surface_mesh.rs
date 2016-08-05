use cgmath::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct TriangleIndices {
    pub indices: [usize; 3]
}

impl TriangleIndices {
    pub fn new(a: usize, b: usize, c: usize) -> Self {
        TriangleIndices {
            indices: [a, b, c]
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct SurfaceMesh<S> where S: BaseNum {
    vertices: Vec<Point3<S>>,
    triangles: Vec<TriangleIndices>
}

impl<'a, S> SurfaceMesh<S> where S: BaseNum {
    pub fn from_indices(vertices: Vec<Point3<S>>, triangles: Vec<TriangleIndices>) -> Option<SurfaceMesh<S>> {
        let num_vertices = vertices.len();
        let indices_are_valid = triangles.iter().all(|t| t.indices.iter().all(|i| i < &num_vertices));

        if indices_are_valid {
            Some(SurfaceMesh {
                vertices: vertices,
                triangles: triangles
            })
        } else {
            None
        }
    }

    pub fn vertices(&'a self) -> &'a Vec<Point3<S>> {
        &self.vertices
    }

    pub fn triangles(&'a self) -> &'a Vec<TriangleIndices> {
        &self.triangles
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_triangles(&self) -> usize {
        self.triangles.len()
    }
}

