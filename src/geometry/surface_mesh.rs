use cgmath::*;
use std::collections::HashMap;
use std::cmp::Ordering;

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

impl<S> ApproxEq for Triangle<S> where S: BaseFloat + ApproxEq {
    type Epsilon = S::Epsilon;

    fn default_epsilon() -> Self::Epsilon {
        S::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        S::default_max_relative()
    }

    fn default_max_ulps() -> u32 {
        S::default_max_ulps()
    }

    fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon) -> bool {
        self.a.relative_eq(&other.a, epsilon, max_relative)
        && self.b.relative_eq(&other.b, epsilon, max_relative)
        && self.c.relative_eq(&other.c, epsilon, max_relative)
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        self.a.ulps_eq(&other.a, epsilon, max_ulps)
        && self.b.ulps_eq(&other.b, epsilon, max_ulps)
        && self.c.ulps_eq(&other.c, epsilon, max_ulps)
    }
}

impl<S> Triangle<S> where S: BaseNum {
    pub fn new(a: Point3<S>, b: Point3<S>, c: Point3<S>) -> Triangle<S> {
        Triangle { a: a, b: b, c: c }
    }
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

/// For any given geometry, there are many ways to represent it
/// using SurfaceMesh, since the vertices and triangles can be
/// numbered in many different ways. The purpose of NormalizedSurfaceMesh
/// is to provide a representation that is unambiguous, in the sense that
/// any two equivalent geometrical configurations will have the same
/// normalized representation, independent of vertex and triangle numbering.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct NormalizedSurfaceMesh<S> where S: BaseNum {
    triangles: Vec<Triangle<S>>
}

impl<S> ApproxEq for NormalizedSurfaceMesh<S> where S: BaseFloat + ApproxEq {
    type Epsilon = S::Epsilon;

    fn default_epsilon() ->Self::Epsilon {
        S::default_epsilon()
    }

    fn default_max_relative() -> Self::Epsilon {
        S::default_max_relative()
    }

    fn default_max_ulps() -> u32 {
        S::default_max_ulps()
    }

    fn relative_eq(&self, other: &Self, epsilon: Self::Epsilon, max_relative: Self::Epsilon) -> bool {
        if self.triangles.len() == other.triangles.len() {
            let mut pairs = self.triangles.iter().zip(other.triangles.iter());
            pairs.all(|(&tri1, &tri2)| tri1.relative_eq(&tri2, epsilon, max_relative))
        } else {
            false
        }
    }

    fn ulps_eq(&self, other: &Self, epsilon: Self::Epsilon, max_ulps: u32) -> bool {
        if self.triangles.len() == other.triangles.len() {
            let mut pairs = self.triangles.iter().zip(other.triangles.iter());
            pairs.all(|(&tri1, &tri2)| tri1.ulps_eq(&tri2, epsilon, max_ulps))
        } else {
            false
        }
    }
}

// TODO: A better approach to this floating point sorting mess would be something like
// implementing a floating point wrapper type that guarantees that the float is finite,
// in which case sorting is well-defined.
fn partially_compare_points<S>(a: Point3<S>, b: Point3<S>) -> Option<Ordering>
    where S: BaseNum + PartialOrd {

    let a: [S; 3] = a.into();
    let b: [S; 3] = b.into();

    // While components are pairwise equal, or they fail to compare,
    // compare the next pair. If any are Less or Greater, stop and use
    // that ordering. If all three components are Equal, return an Ordering::Equal
    // (wrapped in an Option).
    let pairs = a.iter().zip(b.iter());
    pairs.map(|(x_a, x_b)| x_a.partial_cmp(x_b))
         .skip_while(|ordering| ordering == &Some(Ordering::Equal))
         .next()
         .unwrap_or(Some(Ordering::Equal))
}

impl<'a, S> From<&'a SurfaceMesh<S>> for NormalizedSurfaceMesh<S> where S: BaseNum + PartialOrd {
    fn from(mesh: &'a SurfaceMesh<S>) -> Self {
        let mut triangles: Vec<Triangle<S>> = mesh.triangles().collect();
        triangles.sort_by(|tri1, tri2| {
            let a_ordering = partially_compare_points(tri1.a, tri2.a);
            let b_ordering = partially_compare_points(tri1.b, tri2.b);
            let c_ordering = partially_compare_points(tri1.c, tri2.c);

            let orderings: [Option<Ordering>; 3] = [a_ordering, b_ordering, c_ordering];
            orderings.iter()
                     .skip_while(|&ordering| ordering == &Some(Ordering::Equal))
                     .next()
                     .unwrap_or(&Some(Ordering::Equal))
                     .expect("Coordinates must be finite.")
        });

        NormalizedSurfaceMesh {
            triangles: triangles
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

    pub fn subdivide(&self, times: u32) -> Self {
        let mut mesh = self.clone();

        for k in 0 .. times {
            mesh = mesh.subdivide_once();
        }

        mesh
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
                    TriangleIndices::new(b, bc, ab),
                    TriangleIndices::new(c, ac, bc),
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
    use super::{SurfaceMesh, TriangleIndices, NormalizedSurfaceMesh, Triangle};
    use cgmath::Point3;
    use cgmath::ApproxEq;

    #[test]
    fn normalized_empty_mesh() {
        let mesh: SurfaceMesh<f32> = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let normalized = NormalizedSurfaceMesh::from(&mesh);

        assert!(normalized.triangles.is_empty());
    }

    #[test]
    fn normalized_single_triangle() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(0.0, 0.0, 1.0);

        let vertices = vec![a, b, c];
        let indices = vec![TriangleIndices::new(0, 1, 2)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let normalized = NormalizedSurfaceMesh::from(&mesh);

        let expected_triangles = vec![Triangle::new(a, b, c)];

        assert_eq!(expected_triangles, normalized.triangles);
    }

    #[test]
    fn normalized_two_triangles_already_ordered() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(1.0, 0.0, 0.0);
        let d = Point3::new(1.0, 1.0, 0.0);

        let vertices = vec![a, b, c, d];
        let indices = vec![TriangleIndices::new(0, 1, 2), TriangleIndices::new(1, 2, 3)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let normalized = NormalizedSurfaceMesh::from(&mesh);
        let expected_triangles = vec![Triangle::new(a, b, c), Triangle::new(b, c, d)];

        assert_eq!(expected_triangles, normalized.triangles);
    }

    #[test]
    fn normalized_two_triangles_unordered() {
        // In this case, the normalization procedure is expected
        // to swap the ordering of the triangles
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(1.0, 0.0, 0.0);
        let d = Point3::new(1.0, 1.0, 0.0);

        let vertices = vec![a, b, c, d];
        let indices = vec![TriangleIndices::new(1, 2, 3), TriangleIndices::new(0, 1, 2)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let normalized = NormalizedSurfaceMesh::from(&mesh);
        let expected_triangles = vec![Triangle::new(a, b, c), Triangle::new(b, c, d)];

        assert_eq!(expected_triangles, normalized.triangles);
    }

    #[test]
    fn replicate_vertices_on_empty_mesh() {
        let mesh: SurfaceMesh<f32> = SurfaceMesh::from_indices(Vec::new(), Vec::new()).unwrap();
        let replicated = mesh.replicate_vertices();

        assert!(replicated.vertices().is_empty());
        assert!(replicated.triangle_indices().is_empty());
    }

    #[test]
    fn replicate_vertices_on_single_triangle() {
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

    #[test]
    fn subdivide_once_on_single_triangle() {
        let a = Point3::new(0.0, 0.0, 0.0);
        let b = Point3::new(0.0, 1.0, 0.0);
        let c = Point3::new(0.0, 0.0, 1.0);

        let vertices = vec![a, b, c];
        let indices = vec![TriangleIndices::new(0, 1, 2)];
        let mesh = SurfaceMesh::from_indices(vertices, indices).unwrap();

        let subdivided = mesh.subdivide_once();
        let normalized = NormalizedSurfaceMesh::from(&subdivided);

        let ab = Point3::new(0.0, 0.5, 0.0);
        let ac = Point3::new(0.0, 0.0, 0.5);
        let bc = Point3::new(0.0, 0.5, 0.5);

        // Note: We need to preserve orientation of each triangle.
        // NormalizedSurfaceMesh does not change the order of the vertices within
        // each triangle, so we need to make sure we get the order right.
        // Currently we rely on the internals of subdivide_once to figure out
        // the correct order. A better approach would be to implement routines
        // that would let us compare SurfaceMeshes where orientation is taken into
        // account, without requiring exact, but this is rather a lot of work
        // in its own right.

        let expected_triangles = vec![
            Triangle::new(a, ab, ac),
            Triangle::new(c, ac, bc),
            Triangle::new(ab, bc, ac),
            Triangle::new(b, bc, ab),
        ];

        // Assert each individual triangle so that it is easier to debug
        assert_eq!(4, normalized.triangles.len());
        assert_ulps_eq!(expected_triangles[0], normalized.triangles[0]);
        assert_ulps_eq!(expected_triangles[1], normalized.triangles[1]);
        assert_ulps_eq!(expected_triangles[2], normalized.triangles[2]);
        assert_ulps_eq!(expected_triangles[3], normalized.triangles[3]);
    }

    // TODO: Need more tests for almost everything here. In particular,
    // a better way to compare triangulations would be nice. Also,
    // need more tests for subdivide.
}
