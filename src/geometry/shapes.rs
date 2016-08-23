use cgmath::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sphere<S> where S: BaseNum {
    pub radius: S,
    pub center: Point3<S>
}

pub trait OverlapsWith<Shape> {
    fn overlaps_with(&self, shape: &Shape) -> bool;
}

impl<S> OverlapsWith<Sphere<S>> for Sphere<S> where S: BaseFloat {
    fn overlaps_with(&self, other: &Sphere<S>) -> bool {
        let r = other.center - self.center;
        r.magnitude2() <= (self.radius + other.radius).powi(2)
    }
}

mod tests {
    use super::{Sphere, OverlapsWith};
    use cgmath::{Vector3, Point3};

    #[test]
    fn sphere_overlaps_with_sphere() {
        {
            let a = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
            let b = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 3.0) };
            assert_eq!(false, a.overlaps_with(&b));
            assert_eq!(false, b.overlaps_with(&a));
        }

        {
            let a = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
            let b = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 2.0) };
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }

        {
            let a = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
            let b = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }

        {
            let a = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
            let b = Sphere { radius: 1.5, center: Point3::new(0.0, 0.0, 2.0) };
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }
    }
}