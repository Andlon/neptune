use cgmath::*;
use entity::Entity;
use geometry::{Sphere, Cuboid};
use physics::*;

#[derive(Copy, Clone, Debug)]
pub struct ContactData {
    pub point: Point3<f64>,
    pub normal: Vector3<f64>,
    pub penetration_depth: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct Contact {
    pub objects: (Entity, Entity),
    pub physics_components: (PhysicsComponentId, PhysicsComponentId),
    pub data: ContactData
}

pub fn contact_sphere_sphere(sphere1: Sphere<f64>, sphere2: Sphere<f64>) -> Option<ContactData> {
    let r = sphere2.center - sphere1.center;
    let r2 = r.magnitude2();
    if r2 <= (sphere1.radius + sphere2.radius).powi(2) {
        let normal = if r2 == 0.0 { Vector3::new(1.0, 0.0, 0.0) } else { r.normalize() };
        // TODO: Implement Sub<Vector3<S>> for Point3<S> in cgmath?
        let point = sphere2.center + (- sphere2.radius * normal);
        let point2 = sphere1.center + sphere1.radius * normal;
        let depth = point.distance(point2);
        Some(ContactData {
            point: point,
            normal: normal,
            penetration_depth: depth
        })
    } else {
        None
    }
}

pub fn contact_sphere_cuboid(sphere: Sphere<f64>, cuboid: Cuboid<f64>) -> Option<ContactData> {
    let contact_point = cuboid.closest_interior_point(sphere.center);
    let r = contact_point - sphere.center;
    if r.magnitude2() <= sphere.radius.powi(2) {
        let normal = r.normalize();
        let depth = (sphere.radius * normal - r).magnitude();
        Some(ContactData {
            point: contact_point,
            normal: normal,
            penetration_depth: depth
        })
    } else {
        None
    }
}

mod tests {
    use super::*;
    use geometry::{Sphere, Cuboid};
    use cgmath::{Point3, Vector3, EuclideanSpace, InnerSpace, MetricSpace, ApproxEq, Quaternion};

    #[test]
    pub fn contact_sphere_sphere_no_collision() {
        let sphere1 = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
        let sphere2 = Sphere { radius: 1.0, center: Point3::new(3.0, 0.0, 0.0) };

        let contact_data = contact_sphere_sphere(sphere1, sphere2);
        assert!(contact_data.is_none());
    }

    #[test]
    pub fn contact_sphere_sphere_collision() {
        let sphere1 = Sphere { radius: 1.0, center: Point3::new(0.0, 0.0, 0.0) };
        let sphere2 = Sphere { radius: 1.0, center: Point3::new(1.5, 0.0, 0.0) };

        let contact_data = contact_sphere_sphere(sphere1, sphere2);
        assert!(contact_data.is_some());

        let contact_data = contact_data.unwrap();
        assert_approx_eq!(Vector3::new(1.0, 0.0, 0.0), contact_data.normal);
        assert_approx_eq!(Point3::new(0.5, 0.0, 0.0), contact_data.point);
        assert_approx_eq!(0.5, contact_data.penetration_depth);
    }

    #[test]
    pub fn contact_sphere_sphere_complete_overlap() {
        let sphere1 = Sphere { radius: 1.0, center: Point3::origin() };
        let sphere2 = Sphere { radius: 1.0, center: Point3::origin() };

        let contact_data = contact_sphere_sphere(sphere1, sphere2);
        assert!(contact_data.is_some());

        let contact_data = contact_data.unwrap();
        // Neither the normal nor the point of contact is well defined in this situation,
        // but we can check that the point of contact lies somewhere on the unit sphere.
        assert_approx_eq!(1.0, contact_data.normal.magnitude());
        assert_approx_eq!(1.0, contact_data.point.distance(Point3::origin()));
        assert_approx_eq!(2.0, contact_data.penetration_depth);
    }

    #[test]
    pub fn contact_sphere_cuboid_no_collision() {
        let sphere = Sphere {
            radius: 1.0,
            center: Point3::origin()
        };
        let cuboid = Cuboid {
            center: Point3::new(5.0, 0.0, 0.0),
            halfSize: Vector3::new(0.5, 0.5, 0.5),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        };

        let contact_data = contact_sphere_cuboid(sphere, cuboid);
        assert!(contact_data.is_none());
    }

    #[test]
    pub fn contact_sphere_cuboid_overlap() {
        let sphere = Sphere {
            radius: 1.0,
            center: Point3::origin()
        };
        let cuboid = Cuboid {
            center: Point3::new(1.25, 0.0, 0.0),
            halfSize: Vector3::new(0.5, 0.5, 0.5),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        };
        let contact_data = contact_sphere_cuboid(sphere, cuboid);
        assert!(contact_data.is_some());

        let contact_data = contact_data.unwrap();
        assert_approx_eq!(Point3::new(0.75, 0.0, 0.0), contact_data.point);
        assert_approx_eq!(Vector3::new(1.0, 0.0, 0.0), contact_data.normal);
        assert_approx_eq!(0.25, contact_data.penetration_depth);
    }


}