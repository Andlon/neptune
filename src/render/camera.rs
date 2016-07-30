use cgmath::{Quaternion, Point3, Vector3, Matrix4, Transform};
use cgmath::{ApproxEq, EuclideanSpace, Rad, Deg, Rotation3, Rotation};

#[derive(Copy, Clone, Debug)]
pub struct Camera {
    /// The position of the camera relative to the world coordinate system.
    pub position: Point3<f32>,

    /// Orientation of camera coordinate system relative to the world coordinate system.
    ///
    /// Here, we assume the camera coordinate system to be defined such that
    /// the camera is pointing in the direction of the y-axis. The x-axis plays
    /// the role of the conventional "right" vector, while the z-axis corresponds to the
    /// "up" vector. 
    pub orientation: Quaternion<f32>,
}

impl Camera {
    // TODO: impl Default trait instead...?
    pub fn default() -> Self {
        Camera {
            position: Point3::new(0.0, 0.0, 0.0),
            orientation: Quaternion::new(1.0, 0.0, 0.0, 0.0)
        }
    }

    pub fn translate(self, translation: Vector3<f32>) -> Self {
        Camera {
            position: self.position + translation,
            orientation: self.orientation
        }
    }

    pub fn rotate(self, rotation: Quaternion<f32>) -> Self {
        Camera {
            position: self.position,
            orientation: rotation * self.orientation
        }
    }

    pub fn direction(&self) -> Vector3<f32> {
        let yaxis = Vector3::new(0.0, 1.0f32, 0.0);
        self.orientation.rotate_vector(yaxis)
    }

    pub fn right(&self) -> Vector3<f32> {
        let xaxis = Vector3::new(1.0f32, 0.0, 0.0);
        self.orientation.rotate_vector(xaxis)
    }

    pub fn up(&self) -> Vector3<f32> {
        let zaxis = Vector3::new(0.0, 0.0, 1.0f32);
        self.orientation.rotate_vector(zaxis)
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        let mut view_mat = Matrix4::from(self.orientation);
        view_mat.w = self.position.to_vec().extend(1.0);
        view_mat.inverse_transform().unwrap()
    }
}

impl ApproxEq for Camera {
    type Epsilon = f32;

    fn approx_eq_eps(&self, other: &Self, epsilon: &Self::Epsilon) -> bool {
        self.position.approx_eq_eps(&other.position, epsilon)
            && self.orientation.approx_eq_eps(&other.orientation, epsilon)
    }
}

#[test]
fn test_camera_translate() {
    let translation = Vector3::new(1.0f32, -2.0, 3.0);

    let camera = Camera::default();
    let translated = camera.translate(translation);

    let expected = Camera {
        position: Point3::from_vec(translation),
        orientation: camera.orientation 
    };

    assert_approx_eq!(expected, translated);
}

#[test]
fn test_camera_rotate() {
    let rotation = Quaternion::new(2.0, 2.0, -3.0, 5.0);

    let camera = Camera::default();
    let rotated = camera.rotate(rotation);

    let expected = Camera {
        position: Point3::new(0.0, 0.0, 0.0),
        orientation: rotation * camera.orientation
    };

    assert_approx_eq!(expected, rotated);
}

#[test]
fn test_camera_direction_is_initially_y_axis() {
    let camera = Camera::default();
    let yaxis = Vector3::new(0.0, 1.0f32, 0.0);

    assert_approx_eq!(yaxis, camera.direction());    
}

#[test]
fn test_camera_direction_after_rotation() {
    let rotation = Quaternion::from_angle_z(Rad::from(Deg::new(-90.0)));
    let expected = Vector3::new(1.0, 0.0, 0.0);

    let camera = Camera::default();
    let rotated = camera.rotate(rotation);

    assert_approx_eq!(expected, rotated.direction());
}

#[test]
fn test_camera_right_is_initially_x_axis() {
    let camera = Camera::default();
    let xaxis = Vector3::new(1.0f32, 0.0, 0.0);

    assert_approx_eq!(xaxis, camera.right());
}

#[test]
fn test_camera_right_after_rotation() {
    let rotation = Quaternion::from_angle_z(Rad::from(Deg::new(-90.0)));
    let expected = Vector3::new(0.0, -1.0, 0.0);

    let camera = Camera::default();
    let rotated = camera.rotate(rotation);

    assert_approx_eq!(expected, rotated.right());
}

#[test]
fn test_camera_up_is_initially_z_axis() {
    let camera = Camera::default();
    let zaxis = Vector3::new(0.0, 0.0, 1.0f32);
    
    assert_approx_eq!(zaxis, camera.up())
}

#[test]
fn test_camera_up_after_rotation() {
    let rotation = Quaternion::from_angle_x(Rad::from(Deg::new(90.0)));
    let neg_yaxis = Vector3::new(0.0, -1.0f32, 0.0);
    let camera = Camera::default().rotate(rotation);

    assert_approx_eq!(neg_yaxis, camera.up());
}
