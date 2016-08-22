use cgmath::*;

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// The position of the camera relative to the world coordinate system.
    pub position: Point3<f32>,

    /// Orientation of camera coordinate system relative to the world coordinate system.
    orientation: Quaternion<f32>
}

impl Camera {
    pub fn look_at(camera_position: Point3<f32>, point: Point3<f32>, up: Vector3<f32>) -> Option<Camera> {
        let direction = point - camera_position;
        Camera::look_in(camera_position, direction, up)
    }

    pub fn look_in(camera_position: Point3<f32>, direction: Vector3<f32>, up: Vector3<f32>) -> Option<Camera> {
        let right = direction.cross(up);

        if direction.is_zero() || up.is_zero() || right.is_zero() {
            None
        } else {
            // Construct linearly independent unit vectors d, u, p, which
            // help form a basis for the rotation transformation from the
            // world space to the camera space
            let d = direction.normalize();
            let u = right.cross(d).normalize();
            let p = right.normalize();

            // The p, u and -d unit vectors happen to be the image of the
            // x, y and z axis vectors in world space under the rotation transform,
            // so we may form the rotation matrix from this
            let rotation_matrix = Matrix3::from_cols(p, u, -d);
            let camera = Camera {
                position: camera_position,
                orientation: Quaternion::from(rotation_matrix)
            };

            Some(camera)
        }
    }

    pub fn translate(self, translation: Vector3<f32>) -> Self {
        Camera {
            position: self.position + translation,
            orientation: self.orientation
        }
    }

    // TODO: remove this method and use only axis angle
    pub fn rotate(self, rotation: Matrix3<f32>) -> Self {
        // Note: For now we just take a general 3x3 matrix, but this
        // makes for a pretty bad API as there are no guarantees that
        // the user supplies a rotation matrix. The current API of
        // cgmath is insufficient for this particular case.
        let new_orientation = (Quaternion::from(rotation) * self.orientation).normalize();

        Camera {
            position: self.position,
            orientation: new_orientation
        }
    }

    pub fn rotate_axis_angle(&self, axis: Vector3<f32>, angle: Rad<f32>) -> Self {
        let quat = Quaternion::from_axis_angle(axis, angle);
        let new_orientation = (quat * self.orientation).normalize();
        Camera {
            position: self.position,
            orientation: new_orientation
        }
    }

    /// Returns the world coordinates of the direction
    /// that the camera is facing in.
    pub fn direction(&self) -> Vector3<f32> {
        let neg_zaxis = Vector3::new(0.0, 0.0, -1.0f32);
        (self.orientation * neg_zaxis).normalize()
    }

    /// Returns the world coordinates of the 'right' axis of the camera space,
    /// meaning that the right, direction and up vectors form a right-handed
    /// coordinate system.
    pub fn right(&self) -> Vector3<f32> {
        let xaxis = Vector3::new(1.0f32, 0.0, 0.0);
        (self.orientation * xaxis).normalize()
    }

    /// Returns the world coordinates of the conventional 'up' vector.
    pub fn up(&self) -> Vector3<f32> {
        let yaxis = Vector3::new(0.0, 1.0f32, 0.0);
        (self.orientation * yaxis).normalize()
    }

    /// Returns the view_matrix associated with this camera,
    /// which maps from world space to camera space.
    /// This is the inverse of the camera's own transform in world space.
    pub fn view_matrix(&self) -> Matrix4<f32> {
        let mut camera_transform = Matrix4::from(self.orientation);
        camera_transform.w = self.position.to_vec().extend(1.0);
        camera_transform.inverse_transform().unwrap()
    }
}

impl ApproxEq for Camera {
    type Epsilon = f32;

    fn approx_eq_eps(&self, other: &Self, epsilon: &Self::Epsilon) -> bool {
        self.position.approx_eq_eps(&other.position, epsilon)
            && self.orientation.approx_eq_eps(&other.orientation, epsilon)
    }
}

use std::f32::consts::PI;

#[test]
fn camera_look_in_has_correct_initial_position() {
    let position: Point3<f32> = Point3::new(3.0, -2.0, 1.0);
    let camera = Camera::look_in(position, Vector3::unit_x(), Vector3::unit_y()).unwrap();

    assert_approx_eq!(position, camera.position);
}

#[test]
fn camera_look_in_has_correct_initial_orientation() {
    let x: Vector3<f32> = Vector3::unit_x();
    let y: Vector3<f32> = Vector3::unit_y();
    let z: Vector3<f32> = Vector3::unit_z();

    let camera = Camera::look_in(Point3::origin(), y, z).unwrap();

    // Check that the camera's rotation matrix rotates
    // the basis vectors in world space in the following way:
    // x -> x
    // y -> z
    // z -> -y

    assert_approx_eq!(x, camera.orientation * x);
    assert_approx_eq!(z, camera.orientation * y);
    assert_approx_eq!(-y, camera.orientation * z);
}

#[test]
fn camera_translate() {
    let translation = Vector3::new(1.0, -2.0, 3.0);
    let initial_position = Point3::new(1.0, 1.0, 1.0);

    let camera = Camera::look_in(initial_position, Vector3::unit_y(), Vector3::unit_z()).unwrap();
    let translated = camera.translate(translation);

    let expected = Camera {
        position: Point3::new(2.0, -1.0, 4.0),
        orientation: camera.orientation
    };

    assert_approx_eq!(expected, translated);
}

#[test]
fn camera_rotate() {
    let camera = Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z()).unwrap();

    let rotation1 = Matrix3::from_angle_x(Rad::new(PI / 2.0));
    let rotation2 = Matrix3::from_angle_y(Rad::new(PI));
    let rotation = rotation1 * rotation2;

    let rotated = camera.rotate(rotation);

    let expected_camera = Camera::look_in(Point3::origin(), Vector3::unit_z(), Vector3::unit_y()).unwrap();

    assert_approx_eq!(expected_camera.position, rotated.position);
    assert_approx_eq!(expected_camera.direction(), rotated.direction());
    assert_approx_eq!(expected_camera.up(), rotated.up());
    assert_approx_eq!(expected_camera.right(), rotated.right());
}

#[test]
fn camera_direction() {
    let direction = vec3(1.0, 1.0, 1.0);
    let z = Vector3::unit_z();
    let camera = Camera::look_in(Point3::origin(), direction, z).unwrap();

    assert_approx_eq!(direction.normalize(), camera.direction());
}

#[test]
fn camera_up() {
    let direction = vec3(1.0, 1.0, 1.0);
    let z = Vector3::unit_z();
    let camera = Camera::look_in(Point3::origin(), direction, z).unwrap();

    let expected_up = vec3(-1.0, -1.0, 2.0).normalize();
    assert_approx_eq!(expected_up, camera.up());
}

#[test]
fn camera_right() {
    let direction = vec3(1.0, 1.0, 1.0).normalize();
    let z = Vector3::unit_z();
    let camera = Camera::look_in(Point3::origin(), direction, z).unwrap();

    let expected_right = vec3(1.0, -1.0, 0.0).normalize();
    assert_approx_eq!(expected_right, camera.right());
}

#[test]
fn test_camera_view_matrix_undoes_translation() {
    let translation = Vector3::new(2.0, -3.0, 5.0);
    let camera = Camera::look_in(Point3::origin(), Vector3::unit_y(), Vector3::unit_z())
        .unwrap()
        .translate(translation);
    let view = camera.view_matrix();

    let trans4 = translation.extend(1.0);
    let expected = Vector4::new(0.0, 0.0, 0.0, 1.0f32);

    assert_approx_eq!(expected, view * trans4);
}

// TODO: Write more tests for view matrix