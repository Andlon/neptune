use cgmath::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Sphere<S> where S: BaseNum {
    pub radius: S,
    pub center: Point3<S>
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Interval<S: BaseNum> ( S, S );

impl<S> Interval<S> where S: BaseNum {
    pub fn new(a: S, b: S) -> Interval<S> {
        if a <= b {
            Interval(a, b)
        } else {
            Interval(b, a)
        }
    }

    pub fn contains(&self, num: S) -> bool {
        let &Interval(a, b) = self;
        a <= num && num <= b
    }
}

trait OverlapsWith<Shape> {
    fn overlaps_with(&self, shape: &Shape) -> bool;
}

impl<S> OverlapsWith<Sphere<S>> for Sphere<S> where S: BaseFloat {
    fn overlaps_with(&self, other: &Sphere<S>) -> bool {
        let r = other.center - self.center;

        if r.is_zero() {
            true
        } else {
            let separating_axis = r.normalize();

            let self_projected = project_sphere_onto_axis(self, separating_axis);
            let other_projected = project_sphere_onto_axis(other, separating_axis);

            self_projected.overlaps_with(&other_projected)
        }
    }
}

impl<S> OverlapsWith<Interval<S>> for Interval<S> where S: BaseNum {
    fn overlaps_with(&self, interval: &Interval<S>) -> bool {
        // Given two intervals (a1, b1) and (a2, b2),
        // the two intervals overlap if a2 is contained in (a1, b1)
        // _or_ b2 is contained in (a1, b1).
        let &Interval(a, b) = interval;
        self.contains(a) || self.contains(b)
    }
}

fn project_sphere_onto_axis<S>(sphere: &Sphere<S>, normalized_axis: Vector3<S>) -> Interval<S>
    where S: BaseFloat
{
    let axis = normalized_axis;

    // TODO: Implement Sub<Vector3> for Point3 in cgmath
    // Also, for some reason we can only do vector * radius,
    // and not radius * vector. Why?
    let a_pos = sphere.center + - (axis * sphere.radius);
    let b_pos = sphere.center + (axis * sphere.radius);

    Interval::new(
        project_onto_axis(a_pos, axis),
        project_onto_axis(b_pos, axis)
    )
}

fn project_onto_axis<S>(point: Point3<S>, axis: Vector3<S>) -> S
    where S: BaseNum
{
    point.dot(axis)
}

mod tests {
    use super::{Interval, Sphere, OverlapsWith};
    use cgmath::{Vector3, Point3};

    #[test]
    fn interval_is_sorted() {
        let Interval(a, b) = Interval::new(2, 4);
        assert_eq!((2, 4), (a, b));

        let Interval(a, b) = Interval::new(4, 2);
        assert_eq!((2, 4), (a, b));
    }

    #[test]
    fn interval_overlaps_with_interval() {
        {
            let a = Interval::new(2, 4);
            let b = Interval::new(0, 1);
            assert_eq!(false, a.overlaps_with(&b));
            assert_eq!(false, b.overlaps_with(&a));
        }

        {
            let a = Interval::new(2, 4);
            let b = Interval::new(5, 6);
            assert_eq!(false, a.overlaps_with(&b));
            assert_eq!(false, b.overlaps_with(&a));
        }

        {
            let a = Interval::new(1, 2);
            let b = Interval::new(1, 2);
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }

        {
            let a = Interval::new(1, 2);
            let b = Interval::new(2, 3);
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }

        {
            let a = Interval::new(1, 2);
            let b = Interval::new(0, 1);
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }

        {
            let a = Interval::new(1, 3);
            let b = Interval::new(2, 4);
            assert_eq!(true, a.overlaps_with(&b));
            assert_eq!(true, b.overlaps_with(&a));
        }
    }

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