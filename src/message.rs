use glium::glutin::{ElementState, VirtualKeyCode};
use camera::CameraAction;
use entity::Entity;

#[derive(Clone, Debug)]
pub enum Message {
    WindowClosed,
    KeyboardInputReceived(ElementState, VirtualKeyCode),
    CameraCommand(CameraAction),
    CollisionDetected(Entity, Entity)
}

pub trait MessageReceiver {
    fn process_messages(&mut self, messages: &[Message]) -> Vec<Message>;
}