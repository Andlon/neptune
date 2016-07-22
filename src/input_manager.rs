use render::SceneRenderer;
use glium::glutin::{ElementState, VirtualKeyCode};

use cgmath::{InnerSpace, Point3, Vector3};

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

        if state == ElementState::Pressed {
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
                _ => ()
            }
        }
    }
}

use render::Camera;

/// Moves the camera delta units in its current direction.
fn walk_camera(camera: Camera, delta: f32) -> Camera {
    let mut camera = camera;
    let unit_direction = camera.direction.normalize();
    camera.pos = camera.pos + delta * unit_direction;
    camera
}

/// Moves the camera delta units right or left, where delta > 0 moves the camera to the right.
fn strafe_camera(camera: Camera, delta: f32) -> Camera {
    let mut camera = camera;
    let unit_direction = camera.direction.cross(camera.up).normalize();

    // Note the sign: Remember that we are not really moving the camera as much as we are
    // moving the world.
    // Note to self: Is this correct, or a bug? Should go over the math.
    camera.pos = camera.pos + (- delta) * unit_direction;
    camera
}
