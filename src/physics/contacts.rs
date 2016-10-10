use cgmath::{Vector3, Point3, InnerSpace, MetricSpace, Rotation, EuclideanSpace};
use entity::Entity;
use geometry::{Sphere, Cuboid};

#[derive(Copy, Clone, Debug)]
pub struct ContactData {
    pub point: Point3<f64>,
    pub normal: Vector3<f64>,
    pub penetration_depth: f64,
}

#[derive(Copy, Clone, Debug)]
pub struct Contact {
    pub objects: (Entity, Entity),
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

fn cuboid_axes(cuboid: Cuboid<f64>) -> [Vector3<f64>; 3] {
    let rot = cuboid.rotation;
    [
        rot.rotate_vector(Vector3::unit_x()),
        rot.rotate_vector(Vector3::unit_y()),
        rot.rotate_vector(Vector3::unit_z())
    ]
}

fn cuboid_projection_onto_axis(cuboid: Cuboid<f64>, axis: Vector3<f64>) -> f64 {
    let axes = cuboid_axes(cuboid);

    cuboid.half_size.x * axes[0].dot(axis).abs() +
    cuboid.half_size.y * axes[1].dot(axis).abs() +
    cuboid.half_size.z * axes[2].dot(axis).abs()
}

fn cuboid_cuboid_penetration_along_axis(cuboid1: Cuboid<f64>, cuboid2: Cuboid<f64>, axis: Vector3<f64>) -> f64 {
    let projection_1 = cuboid_projection_onto_axis(cuboid1, axis);
    let projection_2 = cuboid_projection_onto_axis(cuboid2, axis);

    // Center-to-center distance
    let distance_along_axis = axis.dot(cuboid2.center - cuboid1.center).abs();

    projection_1 + projection_2 - distance_along_axis
}

#[allow(dead_code)]
pub fn contact_cuboid_cuboid(a: Cuboid<f64>, b: Cuboid<f64>) -> Option<ContactData> {
    let a_axes = cuboid_axes(a);
    let b_axes = cuboid_axes(b);

    let axes = [
        // Face axes
        a_axes[0],
        a_axes[1],
        a_axes[2],
        b_axes[0],
        b_axes[1],
        b_axes[2],

        // Edge-edge axes
        a_axes[0].cross(b_axes[0]),
        a_axes[0].cross(b_axes[1]),
        a_axes[0].cross(b_axes[2]),
        a_axes[1].cross(b_axes[0]),
        a_axes[1].cross(b_axes[1]),
        a_axes[1].cross(b_axes[2]),
        a_axes[2].cross(b_axes[0]),
        b_axes[2].cross(b_axes[1]),
        a_axes[2].cross(b_axes[2])
    ];

    use ordered_float::OrderedFloat;

    let (min_index, min_depth) = axes.iter()
                                    .enumerate()
                                    // Skip axes which were created from (roughly) parallel edges
                                    .filter(|&(_, axis)| axis.magnitude2() > 0.0001)
                                    .map(|(index, axis)| (index, axis.normalize()))
                                    .map(|(index, axis)| (index, cuboid_cuboid_penetration_along_axis(a, b, axis.to_owned())))
                                    // Temporarily convert to OrderedFloat for the Ord implementation
                                    .map(|(index, depth)| (index, OrderedFloat::from(depth)))
                                    .min_by_key(|&(_, depth)| depth)
                                    .map(|(index, depth)| (index, depth.into_inner()))
                                    .unwrap();

    if min_depth >= 0.0 {
        let axis = axes[min_index];
        let relative_center = b.center - a.center;

        // Ensure normal is pointing from a towards b
        let normal = if axis.dot(relative_center) < 0.0 { -axis } else { axis };

        if min_index < 3 {
            // Face-Vertex contact, with face on 'a'

            // Recall that the face axes correspond to the world coordinates
            // of the axes of the Cuboid's local coordinate system.

            // Determine vertex in b's coordinate system
            let mut vertex = b.half_size;
            if normal.dot(b_axes[0]) > 0.0 { vertex.x = -vertex.x; };
            if normal.dot(b_axes[1]) > 0.0 { vertex.y = -vertex.y; };
            if normal.dot(b_axes[2]) > 0.0 { vertex.z = -vertex.z; };

            let vertex = b.rotation.rotate_vector(vertex);
            let vertex = vertex + b.center.to_vec();

            Some(ContactData {
                penetration_depth: min_depth,
                normal: normal,
                point: Point3::from_vec(vertex)
            })
        } else if min_index < 6 {
            // Face-Vertex contact, with face on 'b'

            // Recall that the face axes correspond to the world coordinates
            // of the axes of the Cuboid's local coordinate system.

            // Determine vertex in a's coordinate system
            let mut vertex = a.half_size;
            if normal.dot(a_axes[0]) < 0.0 { vertex.x = -vertex.x; };
            if normal.dot(a_axes[1]) < 0.0 { vertex.y = -vertex.y; };
            if normal.dot(a_axes[2]) < 0.0 { vertex.z = -vertex.z; };

            let vertex = a.rotation.rotate_vector(vertex);
            let vertex = vertex + a.center.to_vec();

            Some(ContactData {
                penetration_depth: min_depth,
                normal: normal,
                point: Point3::from_vec(vertex)
            })
        } else {
            // Edge-to-edge contact
            println!("Unhandled edge-to-edge!");
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::{Sphere, Cuboid};
    use cgmath::{Point3, Vector3, EuclideanSpace, InnerSpace, MetricSpace, Quaternion};

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
        assert_ulps_eq!(Vector3::new(1.0, 0.0, 0.0), contact_data.normal);
        assert_ulps_eq!(Point3::new(0.5, 0.0, 0.0), contact_data.point);
        assert_ulps_eq!(0.5, contact_data.penetration_depth);
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
        assert_ulps_eq!(1.0, contact_data.normal.magnitude());
        assert_ulps_eq!(1.0, contact_data.point.distance(Point3::origin()));
        assert_ulps_eq!(2.0, contact_data.penetration_depth);
    }

    #[test]
    pub fn contact_sphere_cuboid_no_collision() {
        let sphere = Sphere {
            radius: 1.0,
            center: Point3::origin()
        };
        let cuboid = Cuboid {
            center: Point3::new(5.0, 0.0, 0.0),
            half_size: Vector3::new(0.5, 0.5, 0.5),
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
            half_size: Vector3::new(0.5, 0.5, 0.5),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        };
        let contact_data = contact_sphere_cuboid(sphere, cuboid);
        assert!(contact_data.is_some());

        let contact_data = contact_data.unwrap();
        assert_ulps_eq!(Point3::new(0.75, 0.0, 0.0), contact_data.point);
        assert_ulps_eq!(Vector3::new(1.0, 0.0, 0.0), contact_data.normal);
        assert_ulps_eq!(0.25, contact_data.penetration_depth);
    }

    #[test]
    pub fn contact_cuboid_cuboid_vertex_face() {
        use cgmath::{Rotation3, Rad};

        // Test a vertex-face collision between two cuboids.
        // The test data has been constructed through experimentation
        // in Mathematica.

        let a = Cuboid {
            half_size: Vector3::new(1.0, 1.0, 1.0),
            center: Point3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        };
        let b = Cuboid {
            half_size: Vector3::new(1.0, 1.0, 1.0),
            center: Point3::new(5.0 / 2.0, 0.0, 0.0),
            rotation: Quaternion::from_axis_angle(Vector3::new(-0.4, 0.4, 0.6).normalize(), Rad(-1.2))
        };

        let expected_point = Point3::new(0.78232862985420703605, 0.088745132099030454853, -0.20427766816321585308);
        let expected_depth = 0.21767137014579296395;

        // Argument order a-b
        {
            let contact = contact_cuboid_cuboid(a, b);

            assert!(contact.is_some());
            let contact = contact.unwrap();

            let expected_point = Point3::new(0.78232862985420703605, 0.088745132099030454853, -0.20427766816321585308);
            assert_ulps_eq!(contact.normal, Vector3::new(1.0, 0.0, 0.0));
            assert_ulps_eq!(contact.point, expected_point);
            assert_ulps_eq!(contact.penetration_depth, expected_depth);
        }

        // Argument order b-a
        {
            let contact = contact_cuboid_cuboid(b, a);

            assert!(contact.is_some());
            let contact = contact.unwrap();

            assert_ulps_eq!(contact.normal, Vector3::new(-1.0, 0.0, 0.0));
            assert_ulps_eq!(contact.point, expected_point);
            assert_ulps_eq!(contact.penetration_depth, expected_depth);
        }
    }

}
