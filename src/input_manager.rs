use render::SceneRenderer;
use glium::glutin::{ElementState, VirtualKeyCode};

use cgmath::*;

pub struct InputManager {

}

impl InputManager {
    pub fn new() -> Self {
        InputManager {

        }
    }

    pub fn handle_keyboard_input(&mut self,
        scene_renderer: &mut SceneRenderer,
        state: ElementState,
        vkcode: VirtualKeyCode)
    {
        // Note: Here we take the scene_renderer as a parameter to handle input, which
        // is of course very untidy. In the future we need to make some kind of messaging systems
        // to avoid such unnecessary tight coupling.
        // Instead, there'll be some sort of CameraController which receives events that the
        // Camera should move so and so much. For now, we just hack things together here
        // so that we can actually see things coming together on screen.

        use glium::glutin::VirtualKeyCode;
        let delta = 0.25;
        let delta_rot = 0.1;

        if state == ElementState::Pressed {
            let camera = scene_renderer.camera;
            match vkcode {
                VirtualKeyCode::W => {
                    scene_renderer.camera = walk_camera(scene_renderer.camera, delta)
                },
                VirtualKeyCode::S => {
                    scene_renderer.camera = walk_camera(scene_renderer.camera, -delta)
                },
                VirtualKeyCode::D => {
                    scene_renderer.camera = strafe_camera(scene_renderer.camera, delta)
                },
                VirtualKeyCode::A => {
                    scene_renderer.camera = strafe_camera(scene_renderer.camera, -delta)
                },
                VirtualKeyCode::Q => {
                    scene_renderer.camera = camera.rotate_axis_angle(camera.direction(), Rad::new(-delta))
                },
                VirtualKeyCode::E => {
                    scene_renderer.camera = camera.rotate_axis_angle(camera.direction(), Rad::new(delta))
                },
                VirtualKeyCode::Left => {
                    let rotation = Matrix3::from_axis_angle(camera.up(), Rad::new(delta_rot));
                    scene_renderer.camera = camera.rotate(rotation)
                },
                VirtualKeyCode::Right => {
                    let rotation = Matrix3::from_axis_angle(camera.up(), Rad::new(-delta_rot));
                    scene_renderer.camera = camera.rotate(rotation)
                },
                VirtualKeyCode::Up => {
                    let rotation = Matrix3::from_axis_angle(camera.right(), Rad::new(delta_rot));
                    scene_renderer.camera = camera.rotate(rotation)
                },
                VirtualKeyCode::Down => {
                    let rotation = Matrix3::from_axis_angle(camera.right(), Rad::new(-delta_rot));
                    scene_renderer.camera = camera.rotate(rotation)
                },
                _ => ()
            }
        }
    }
}

use render::Camera;

/// Moves the camera delta units in its current direction.
fn walk_camera(camera: Camera, delta: f32) -> Camera {
    camera.translate(delta * camera.direction())
}

/// Moves the camera delta units right or left, where delta > 0 moves the camera to the right.
fn strafe_camera(camera: Camera, delta: f32) -> Camera {
    camera.translate(delta * camera.right())
}
