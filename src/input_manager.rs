use glium::glutin::{ElementState, VirtualKeyCode};
use message::{Message, MessageReceiver};

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
        use glium::glutin::VirtualKeyCode;
        use camera_controller::CameraAction;

        let mut messages = Vec::new();
        if state == ElementState::Pressed {
            let response = match vkcode {
                VirtualKeyCode::W => Some(Message::CameraCommand(CameraAction::TranslateForward)),
                VirtualKeyCode::S => Some(Message::CameraCommand(CameraAction::TranslateBackward)),
                VirtualKeyCode::D => Some(Message::CameraCommand(CameraAction::TranslateRight)),
                VirtualKeyCode::A => Some(Message::CameraCommand(CameraAction::TranslateLeft)),
                VirtualKeyCode::Q => Some(Message::CameraCommand(CameraAction::TwistLeft)),
                VirtualKeyCode::E => Some(Message::CameraCommand(CameraAction::TwistRight)),
                VirtualKeyCode::Left => Some(Message::CameraCommand(CameraAction::RotateLeft)),
                VirtualKeyCode::Right => Some(Message::CameraCommand(CameraAction::RotateRight)),
                VirtualKeyCode::Up => Some(Message::CameraCommand(CameraAction::RotateUp)),
                VirtualKeyCode::Down => Some(Message::CameraCommand(CameraAction::RotateDown)),
                _ => None,
            };

            if let Some(message) = response {
                messages.push(message);
            }
        }

        messages
    }
}

impl MessageReceiver for InputManager {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message> {
        use glium::glutin::Event;
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