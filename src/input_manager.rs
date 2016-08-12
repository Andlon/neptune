use glium::glutin::{ElementState, VirtualKeyCode};
use message::{Message, MessageReceiver};
use camera::CameraAction;

pub struct InputManager;

impl InputManager {
    pub fn new() -> Self {
        InputManager {

        }
    }

    fn handle_keyboard_input(&self,
        state: ElementState,
        vkcode: VirtualKeyCode)
        -> Vec<Message>
    {
        let camera = |action| Some(Message::CameraCommand(action));
        let pressed = state == ElementState::Pressed;
        let released = state == ElementState::Released;

        let response = match vkcode {
            VirtualKeyCode::W     if pressed  => camera(CameraAction::TranslateForwardBegin),
            VirtualKeyCode::W     if released => camera(CameraAction::TranslateForwardEnd),
            VirtualKeyCode::S     if pressed  => camera(CameraAction::TranslateBackwardBegin),
            VirtualKeyCode::S     if released => camera(CameraAction::TranslateBackwardEnd),
            VirtualKeyCode::D     if pressed  => camera(CameraAction::TranslateRightBegin),
            VirtualKeyCode::D     if released => camera(CameraAction::TranslateRightEnd),
            VirtualKeyCode::A     if pressed  => camera(CameraAction::TranslateLeftBegin),
            VirtualKeyCode::A     if released => camera(CameraAction::TranslateLeftEnd),
            VirtualKeyCode::Q     if pressed  => camera(CameraAction::TwistLeftBegin),
            VirtualKeyCode::Q     if released => camera(CameraAction::TwistLeftEnd),
            VirtualKeyCode::E     if pressed  => camera(CameraAction::TwistRightBegin),
            VirtualKeyCode::E     if released => camera(CameraAction::TwistRightEnd),
            VirtualKeyCode::Left  if pressed  => camera(CameraAction::RotateLeftBegin),
            VirtualKeyCode::Left  if released => camera(CameraAction::RotateLeftEnd),
            VirtualKeyCode::Right if pressed  => camera(CameraAction::RotateRightBegin),
            VirtualKeyCode::Right if released => camera(CameraAction::RotateRightEnd),
            VirtualKeyCode::Up    if pressed  => camera(CameraAction::RotateUpBegin),
            VirtualKeyCode::Up    if released => camera(CameraAction::RotateUpEnd),
            VirtualKeyCode::Down  if pressed  => camera(CameraAction::RotateDownBegin),
            VirtualKeyCode::Down  if released => camera(CameraAction::RotateDownEnd),
            _ => None,
        };

        response.map(|x| vec![x])
                .unwrap_or_else(|| Vec::new())
    }
}

impl MessageReceiver for InputManager {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message> {
        let mut response = Vec::new();
        for message in messages {
            match message {
                &Message::KeyboardInputReceived(state, vkcode)
                    => response.extend(self.handle_keyboard_input(state, vkcode)),
                _ => ()
            }
        }
        response
    }
}