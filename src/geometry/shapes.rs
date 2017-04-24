use alga::general::Real;
use nalgebra::{Point3, Vector3, UnitQuaternion, Scalar};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere<S> where S: Real + Scalar {
    pub radius: S,
    pub center: Point3<S>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cuboid<S> where S: Real + Scalar {
    pub center: Point3<S>,
    pub half_size: Vector3<S>,
    pub rotation: UnitQuaternion<S>
}
