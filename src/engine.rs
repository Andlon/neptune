use entity::{Entity, EntityManager};
use render::*;
use glium;
use glium::backend::Facade;
use input_manager::InputManager;

use cgmath::{Vector3, Point3};

enum Event {
    Quit,
}

pub struct Engine {

}

impl Engine {

    pub fn new() -> Engine {
        Engine { }
    }

    pub fn run(&mut self) {
        // Set up stores
        let mut entity_manager = EntityManager::new();
        let mut scene_renderable_store = SceneRenderableStore::new();
        let mut scene_transform_store = SceneTransformStore::new();

        // Move this into a WindowManager or similar
        use glium::{DisplayBuild, Surface};
        let display = glium::glutin::WindowBuilder::new().build_glium().unwrap();

        // Set up systems
        let mut scene_renderer = SceneRenderer::new(&display);
        let mut input_manager = InputManager::new();

        // Temporarily create a triangle entity here for testing
        let (a, b, c, d) = (Point3::new(-0.5, 0.0, 0.0), Point3::new(0.5, 0.0, 0.0),
                            Point3::new(0.0, 0.5, 0.0), Point3::new(0.0, 0.25, 0.5));
        let triangle_entity = entity_manager.create();
        let triangle_renderable = build_tetrahedron_renderable(&display, a, b, c, d);
        let triangle_transform = SceneTransform {
            position: Point3 { x: 0.0, y: 5.0, z: 0.0 }
        };
        scene_renderable_store.set_renderable(triangle_entity, triangle_renderable);
        scene_transform_store.set_transform(triangle_entity, triangle_transform);

        // Also create an icosahedron
        let ico_entity = entity_manager.create();
        let ico_renderable = build_icosahedron_renderable(&display);
        let ico_transform = SceneTransform {
            position: Point3 { x: 0.0, y: 15.0, z: 0.0 }
        };
        scene_renderable_store.set_renderable(ico_entity, ico_renderable);
        scene_transform_store.set_transform(ico_entity, ico_transform);

        loop {
            // Move this into a window manager or something too
            let mut target = display.draw();
            target.clear_color(0.0, 0.0, 0.0, 1.0);

            scene_renderer.render(&scene_renderable_store, &scene_transform_store, &mut target);

            target.finish().unwrap();

            for ev in display.poll_events() {
                match ev {
                    glium::glutin::Event::Closed => return,
                    glium::glutin::Event::KeyboardInput(state, _, vkcode) => {
                        if let Some(vkcode) = vkcode {
                            input_manager.handle_keyboard_input(&mut scene_renderer, state, vkcode);
                        }
                    },
                    _ => ()
                }
            }
        }
    }
}
