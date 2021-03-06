use camera::Camera;
use message::{Message, MessageReceiver};
use cgmath::{Vector3, Zero, InnerSpace};

#[derive(Copy, Clone, Debug)]
pub enum CameraAction {
    TranslateForwardBegin,
    TranslateForwardEnd,
    TranslateBackwardBegin,
    TranslateBackwardEnd,
    TranslateLeftBegin,
    TranslateLeftEnd,
    TranslateRightBegin,
    TranslateRightEnd,
    RotateRightBegin,
    RotateRightEnd,
    RotateLeftBegin,
    RotateLeftEnd,
    RotateUpBegin,
    RotateUpEnd,
    RotateDownBegin,
    RotateDownEnd,
    TwistRightBegin,
    TwistRightEnd,
    TwistLeftBegin,
    TwistLeftEnd
}

pub struct CameraController {
    // Current controller state
    translate_forward: bool,
    translate_backward: bool,
    translate_left: bool,
    translate_right: bool,
    rotate_up: bool,
    rotate_down: bool,
    rotate_left: bool,
    rotate_right: bool,
    twist_right: bool,
    twist_left: bool
}

impl CameraController {
    pub fn new() -> Self {
        CameraController {
            translate_forward: false,
            translate_backward: false,
            translate_left: false,
            translate_right: false,
            rotate_up: false,
            rotate_down: false,
            rotate_left: false,
            rotate_right: false,
            twist_right: false,
            twist_left: false
        }
    }
}

impl CameraController {

    pub fn update(&mut self, camera: Camera, frame_time: f64) -> Camera {
        assert!(frame_time >= 0.0);
        const TRANSLATION_SPEED: f64 = 4.0;
        const ROTATION_SPEED: f64 = 1.5;

        let trans_amount = (TRANSLATION_SPEED * frame_time) as f32;
        let rot_angle = (ROTATION_SPEED * frame_time) as f32;

        let translation = trans_amount * self.determine_direction(&camera);
        let rotated_camera = self.rotate_camera(camera, rot_angle);

        rotated_camera.translate(translation)
    }

    fn determine_direction(&self, camera: &Camera) -> Vector3<f32> {
        let mut direction = Vector3::zero();

        if self.translate_forward { direction += camera.direction(); };
        if self.translate_backward { direction -= camera.direction(); };
        if self.translate_left { direction -= camera.right(); };
        if self.translate_right { direction += camera.right(); };

        if direction.is_zero() { direction} else { direction.normalize() }
    }

    fn rotate_camera(&self, mut camera: Camera, angle: f32) -> Camera {
        use cgmath::Rad;
        let angle = Rad(angle);

        if self.rotate_right { camera = camera.rotate_axis_angle(camera.up(), -angle); }
        if self.rotate_left  { camera = camera.rotate_axis_angle(camera.up(), angle); }
        if self.rotate_up    { camera = camera.rotate_axis_angle(camera.right(), angle); }
        if self.rotate_down  { camera = camera.rotate_axis_angle(camera.right(), -angle); }
        if self.twist_right  { camera = camera.rotate_axis_angle(camera.direction(), angle); }
        if self.twist_left   { camera = camera.rotate_axis_angle(camera.direction(), -angle); }

        camera
    }

    fn perform_action(&mut self, action: CameraAction) {
        match action {
            CameraAction::TranslateForwardBegin => self.translate_forward = true,
            CameraAction::TranslateForwardEnd => self.translate_forward = false,
            CameraAction::TranslateBackwardBegin => self.translate_backward = true,
            CameraAction::TranslateBackwardEnd => self.translate_backward = false,
            CameraAction::TranslateLeftBegin => self.translate_left = true,
            CameraAction::TranslateLeftEnd => self.translate_left = false,
            CameraAction::TranslateRightBegin => self.translate_right = true,
            CameraAction::TranslateRightEnd => self.translate_right = false,
            CameraAction::RotateUpBegin => self.rotate_up = true,
            CameraAction::RotateUpEnd => self.rotate_up = false,
            CameraAction::RotateDownBegin => self.rotate_down = true,
            CameraAction::RotateDownEnd => self.rotate_down = false,
            CameraAction::RotateLeftBegin => self.rotate_left = true,
            CameraAction::RotateLeftEnd => self.rotate_left = false,
            CameraAction::RotateRightBegin => self.rotate_right = true,
            CameraAction::RotateRightEnd => self.rotate_right = false,
            CameraAction::TwistLeftBegin => self.twist_left = true,
            CameraAction::TwistLeftEnd => self.twist_left = false,
            CameraAction::TwistRightBegin => self.twist_right = true,
            CameraAction::TwistRightEnd => self.twist_right = false,
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