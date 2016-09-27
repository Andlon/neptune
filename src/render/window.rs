use glium;
use glium::backend::glutin_backend::GlutinFacade;
use glium::Surface;
use message::Message;

pub struct Frame {
    pub internal_frame: glium::Frame,
}

impl Frame {
    pub fn finish(self) {
        // For now ignore any kind of error handling.
        self.internal_frame.finish().unwrap();
    }
}

pub struct Window {
    // TODO: Make this private but still accessible for other
    // submodules in the render module
    pub display: GlutinFacade,
}

impl Window {
    pub fn new() -> Self {
        // Note: May have to implement the builder pattern in the future.
        use glium::{DisplayBuild};
        Window {
            display: glium::glutin::WindowBuilder::new()
                        .with_depth_buffer(24)
                        .with_vsync()
                        .build_glium().unwrap()
        }
    }

    pub fn begin_frame(&self) -> Frame {
        let mut frame = Frame { internal_frame: self.display.draw() };
        frame.internal_frame.clear_color_and_depth((0.0, 0.0, 0.0, 1.0), 1.0);
        frame
    }

    pub fn check_events(&self) -> Vec<Message> {
        let mut messages = Vec::new();
        for event in self.display.poll_events() {
            match event {
                glium::glutin::Event::Closed => messages.push(Message::WindowClosed),
                glium::glutin::Event::KeyboardInput(state, _, opt_vk) => {
                    if let Some(vk) = opt_vk {
                        messages.push(Message::KeyboardInputReceived(state, vk));
                    }
                }
                _ => ()
            }
        }
        messages
    }
}