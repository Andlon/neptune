use ecs::{Entity, EntityManager};
use scene_renderer::{SceneRenderer, SceneRenderable, RenderVertex};
use glium;
use glium::backend::Facade;

enum Event {
    Quit,
}

pub struct Engine {

}

// Temp code to build a triangle entity
fn build_triangle_renderable<F>(display: &F) -> SceneRenderable where F: Facade {
    let a = RenderVertex { pos: [-0.5, -0.5, 0.0] };
    let b = RenderVertex { pos: [ 0.0,  0.5, 0.0] };
    let c = RenderVertex { pos: [ 0.5, -0.25, 0.0] };
    let shape = vec![a, b, c];

    let vertex_buffer = glium::VertexBuffer::new(display, &shape).unwrap();
    let indices = glium::IndexBuffer::new(display,
        glium::index::PrimitiveType::TrianglesList,
        &[0, 1, 2]).unwrap();

    use std::rc::Rc;
    SceneRenderable {
        vertices: Rc::new(vertex_buffer),
        indices: Rc::new(indices)
    }
}

impl Engine {

    pub fn new() -> Engine {
        Engine { }
    }

    pub fn run(&mut self) {
        // Move this into a WindowManager or similar
        use glium::{DisplayBuild, Surface};
        let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

        let mut entity_manager = EntityManager::new();
        let mut scene_renderer = SceneRenderer::new(&display);

        // Temporarily create a triangle entity here for testing
        let triangle_entity = entity_manager.create();
        let triangle_renderable = build_triangle_renderable(&display);
        let triangle_identifier = scene_renderer.add_renderable(
            triangle_entity,
            triangle_renderable
        );

        loop {
            // Move this into a window manager or something too
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            scene_renderer.render(&mut target);

            target.finish().unwrap();

            for ev in display.poll_events() {
                match ev {
                    glium::glutin::Event::Closed => return,
                    _ => ()
                }
            }
        }
    }
}
