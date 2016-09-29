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

impl<S> Cuboid<S> where S: BaseFloat {
    /// Computes the point in the Cuboid, in world coordinates, which is closest to the given point (in world coordinates).
    pub fn closest_interior_point(&self, point: Point3<S>) -> Point3<S> {
        // Transform the point into the local coordinate system of the cuboid
        let local_point = Point3::from_vec(self.rotation.invert().rotate_vector(point - self.center));

        // In the local coordinate system, the cuboid is axis-aligned, so we can decompose the problem
        // into considering the distance along each axis of the coordinate system
        let x = {
            let dist_along_axis = local_point.x;
            if dist_along_axis > self.half_size.x {
                self.half_size.x
            } else if dist_along_axis < -self.half_size.x {
                -self.half_size.x
            } else {
                dist_along_axis
            }
        };

        let y = {
            let dist_along_axis = local_point.y;
            if dist_along_axis > self.half_size.y {
                self.half_size.y
            } else if dist_along_axis < -self.half_size.y {
                -self.half_size.y
            } else {
                dist_along_axis
            }
        };

        let z = {
            let dist_along_axis = local_point.z;
            if dist_along_axis > self.half_size.z {
                self.half_size.z
            } else if dist_along_axis < -self.half_size.z {
                -self.half_size.z
            } else {
                dist_along_axis
            }
        };

        let local_closest = Point3::new(x, y, z);
        let global_closest = self.rotation.rotate_point(local_closest) + self.center.to_vec();
        global_closest
    }
}

#[cfg(test)]
mod tests {
    use super::{Cuboid};
    use cgmath::{Vector3, Point3, Rad, Quaternion, EuclideanSpace, Rotation3, Zero};

    #[test]
    fn cuboid_closest_interior_point_for_axis_aligned_cuboid() {
        let half_size = Vector3::new(0.5, 0.5, 0.5);
        let center = Point3::origin();
        let rotation = Quaternion::from_axis_angle(Vector3::unit_x(), Rad::zero());
        let cuboid = Cuboid { center: center, half_size: half_size, rotation: rotation };

        {
            // Test with a point located exactly at the origin
            let point = Point3::origin();
            assert_ulps_eq!(Point3::origin(), cuboid.closest_interior_point(point));
        }

        {
            // Test with an arbitrary interior point
            let point = Point3::new(0.2, 0.1, -0.3);
            let expected = point;
            assert_ulps_eq!(expected, cuboid.closest_interior_point(point));
        }

        {
            // Test with a point located exactly at a corner vertex
            let point = Point3::new(0.5, 0.5, 0.5);
            let expected = point;
            assert_ulps_eq!(expected, cuboid.closest_interior_point(point));
        }

        {
            // Test with a point located outside the cuboid
            let point = Point3::new(1.0, 0.0, 0.0);
            let expected = Point3::new(0.5, 0.0, 0.0);
            assert_ulps_eq!(expected, cuboid.closest_interior_point(point));
        }
    }

    // TODO: Implement tests for arbitrary cuboid. Perhaps use quickcheck or similar property-based testing?
    // TODO: Also write tests where the sphere is on the 'negative' side of the axes
}
