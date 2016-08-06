use cgmath::*;
use std::collections::HashMap;

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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Triangle<S> where S: BaseNum {
    pub a: Point3<S>,
    pub b: Point3<S>,
    pub c: Point3<S>
}

use std::slice::Iter;
pub struct TriangleIter<'a, S> where S: 'a + BaseNum {
    vertices: &'a [Point3<S>],
    index_iter: Iter<'a, TriangleIndices>
}

impl <'a, S> Iterator for TriangleIter<'a, S> where S: BaseNum {
    type Item = Triangle<S>;
    fn next(&mut self) -> Option<Self::Item> {
        self.index_iter.next()
            .map(|triangle_indices| {
                let indices = triangle_indices.indices;
                Triangle {
                    a: self.vertices[indices[0]],
                    b: self.vertices[indices[1]],
                    c: self.vertices[indices[2]]
                }
            })
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

    pub fn vertices(&'a self) -> &'a [Point3<S>] {
        &self.vertices[..]
    }

    pub fn triangle_indices(&'a self) -> &'a [TriangleIndices] {
        &self.triangles[..]
    }

    pub fn triangles(&'a self) -> TriangleIter<'a, S> {
        TriangleIter {
            vertices: self.vertices(),
            index_iter: self.triangle_indices().iter()
        }
    }

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_triangles(&self) -> usize {
        self.triangles.len()
    }

    /// Takes a mesh and constructs a new geometrically equivalent mesh
    /// where each vertex position is repeated such that every vertex is associated
    /// with exactly one triangle.
    pub fn replicate_vertices(&self) -> SurfaceMesh<S> {
        let vertices: Vec<Point3<S>> = self.triangle_indices().iter()
            .flat_map(|triangle| {
                triangle.indices.iter()
                        .map(|index| self.vertices()[index.clone()])
            }).collect();

        let triangle_indices: Vec<TriangleIndices> = (0 .. self.num_triangles())
            .map(|triangle_index| {
                let first_vertex = 3 * triangle_index as usize;
                TriangleIndices::new(first_vertex, first_vertex + 1, first_vertex + 2)
            }).collect();

        SurfaceMesh::from_indices(vertices, triangle_indices)
            .expect("Returned mesh should always be valid since it starts with a valid mesh.")
    }

    pub fn subdivide_once(&self) -> Self {
        let (new_vertices, midpoints) = extend_with_midpoints(self);

        // When adding the midpoint vertices, there are now
        // 6 vertices intersecting each triangle,
        // so we may form a total of 4 new triangles for each triangle.
        let triangles = self.triangle_indices();
        let new_triangles = triangles.iter()
            .flat_map(|triangle| {
                let (a, b, c) = (triangle.indices[0], triangle.indices[1], triangle.indices[2]);
                let ab = midpoints.get(&sort_tuple((a, b))).unwrap().clone();
                let ac = midpoints.get(&sort_tuple((a, c))).unwrap().clone();
                let bc = midpoints.get(&sort_tuple((b, c))).unwrap().clone();

                // It is quite inefficient to allocate a vector here,
                // however fixed size arrays do not seem to support into_iter()?
                // One could conceivably create an iterator that internally constructs
                // a fixed-size array.
                vec![
                    TriangleIndices::new(a, ab, ac),
                    TriangleIndices::new(ab, b, bc),
                    TriangleIndices::new(bc, c, ac),
                    TriangleIndices::new(ab, bc, ac)
                ].into_iter()
            }).collect();

        SurfaceMesh::from_indices(new_vertices, new_triangles)
            .expect("The subdivded mesh should always be valid.")
    }
}

fn extend_with_midpoints<S>(mesh: &SurfaceMesh<S>) -> (Vec<Point3<S>>, HashMap<(usize, usize), usize>) where S: BaseNum {
    let mut vertices = Vec::from(mesh.vertices());
    let mut midpoints: HashMap<(usize, usize), usize> = HashMap::new();

    {
        let mut insert_midpoint = |a: usize, b: usize| {
            let index_pair = sort_tuple((a, b));
            let entry = midpoints.entry(index_pair).or_insert(vertices.len());
            if entry == &vertices.len() {
                let midpoint = vertices[a].midpoint(vertices[b]);
                vertices.push(midpoint)
            }
        };

        for indices in mesh.triangle_indices().iter().map(|triangle| triangle.indices) {
            let (a, b, c) = (indices[0], indices[1], indices[2]);

            insert_midpoint(a, b);
            insert_midpoint(a, c);
            insert_midpoint(b, c);
        }
    }

    (vertices, midpoints)
}

#[inline]
fn sort_tuple<T>((a, b): (T, T)) -> (T, T) where T: Ord {
    if b < a {
        (b, a)
    } else {
        (a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::{SurfaceMesh, TriangleIndices};

    #[test]
    fn replicate_vertices_on_empty_mesh() {
        let mesh: SurfaceMesh<f32> = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let replicated = mesh.replicate_vertices();

        assert!(replicated.vertices().is_empty());
        assert!(replicated.triangle_indices().is_empty());
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

        let replicated = mesh.replicate_vertices();

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

        let replicated = mesh.replicate_vertices();

        assert_eq!(expected_mesh, replicated);
    }

    #[test]
    fn subdivide_once_on_empty_mesh() {
        let mesh: SurfaceMesh<f32> = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let subdivided = mesh.subdivide_once();

        assert!(subdivided.vertices().is_empty());
        assert!(subdivided.triangle_indices().is_empty());
    }
}
