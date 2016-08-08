use glium::glutin::{ElementState, VirtualKeyCode};
use camera_controller::CameraAction;

#[derive(Clone, Debug)]
pub enum Message {
    WindowClosed,
    KeyboardInputReceived(ElementState, VirtualKeyCode),
    CameraCommand(CameraAction)
}

pub trait MessageReceiver {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message>;
}