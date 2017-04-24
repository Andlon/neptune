use cgmath::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sphere<S> where S: BaseNum {
    pub radius: S,
    pub center: Point3<S>
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cuboid<S> where S: BaseNum {
    pub center: Point3<S>,
    pub half_size: Vector3<S>,
    pub rotation: Quaternion<S>
}
