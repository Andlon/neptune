use render::Camera;
use message::{Message, MessageReceiver};

#[derive(Copy, Clone, Debug)]
pub enum CameraAction {
    TranslateForward,
    TranslateBackward,
    TranslateLeft,
    TranslateRight,
    RotateRight,
    RotateLeft,
    RotateUp,
    RotateDown,
    TwistRight,
    TwistLeft,
}

pub struct CameraController {
    camera: Camera
}

impl From<Camera> for CameraController {
    fn from(camera: Camera) -> Self {
        CameraController {
            camera: camera
        }
    }
}

impl CameraController {
    pub fn camera(&self) -> Camera {
        self.camera
    }

    fn perform_action(&mut self, action: CameraAction) {
        use cgmath::Rad;
        let camera = self.camera;
        let delta_trans = 0.25;
        let delta_rot = Rad::new(0.1);

        self.camera = match action {
            CameraAction::TranslateForward => camera.translate(delta_trans * camera.direction()),
            CameraAction::TranslateBackward => camera.translate(-delta_trans * camera.direction()),
            CameraAction::TranslateLeft => camera.translate(-delta_trans * camera.right()),
            CameraAction::TranslateRight => camera.translate(delta_trans * camera.right()),
            CameraAction::RotateUp => camera.rotate_axis_angle(camera.right(), delta_rot),
            CameraAction::RotateDown => camera.rotate_axis_angle(camera.right(), -delta_rot),
            CameraAction::RotateLeft => camera.rotate_axis_angle(camera.up(), delta_rot),
            CameraAction::RotateRight => camera.rotate_axis_angle(camera.up(), -delta_rot),
            CameraAction::TwistLeft => camera.rotate_axis_angle(camera.direction(), -delta_rot),
            CameraAction::TwistRight => camera.rotate_axis_angle(camera.direction(), delta_rot)
        }
    }
}

impl MessageReceiver for CameraController {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message> {
        for message in messages {
            match message {
                &Message::CameraCommand(action) => self.perform_action(action),
                _ => ()
            }
        }
        Vec::new()
    }
}