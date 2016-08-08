use entity::{Entity, EntityManager};
use render::*;
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

        let window = Window::new();

        // Set up systems
        let mut scene_renderer = SceneRenderer::new(&window);
        let mut input_manager = InputManager::new();

        // Set up scene (temporarily here for simplicity, will of course be dynamically
        // loaded later on).
        {
            // Temporarily create a triangle entity here for testing
            let (a, b, c, d) = (Point3::new(-0.5, 0.0, 0.0), Point3::new(0.5, 0.0, 0.0),
                                Point3::new(0.0, 0.5, 0.0), Point3::new(0.0, 0.25, 0.5));
            let triangle_entity = entity_manager.create();
            let triangle_renderable = tetrahedron_renderable(&window, a, b, c, d);
            let triangle_transform = SceneTransform {
                position: Point3 { x: 0.0, y: 5.0, z: 0.0 }
            };
            scene_renderable_store.set_renderable(triangle_entity, triangle_renderable);
            scene_transform_store.set_transform(triangle_entity, triangle_transform);

            // Also create an icosahedron
            let ico_entity = entity_manager.create();
            let ico_renderable = icosahedron_renderable(&window);
            let ico_transform = SceneTransform {
                position: Point3 { x: 0.0, y: 15.0, z: 0.0 }
            };
            scene_renderable_store.set_renderable(ico_entity, ico_renderable);
            scene_transform_store.set_transform(ico_entity, ico_transform);

            // And a unit sphere
            let sphere_entity = entity_manager.create();
            let sphere_renderable = unit_sphere_renderable(&window, 4);
            let sphere_transform = SceneTransform {
                position: Point3 { x: 0.0, y: 15.0, z: 5.0 }
            };
            scene_renderable_store.set_renderable(sphere_entity, sphere_renderable);
            scene_transform_store.set_transform(sphere_entity, sphere_transform);
        }

        loop {
            let mut frame = window.begin_frame();
            scene_renderer.render(&mut frame, &scene_renderable_store, &scene_transform_store);
            frame.finish()

            // for ev in display.poll_events() {
            //     match ev {
            //         glium::glutin::Event::Closed => return,
            //         glium::glutin::Event::KeyboardInput(state, _, vkcode) => {
            //             if let Some(vkcode) = vkcode {
            //                 input_manager.handle_keyboard_input(&mut scene_renderer, state, vkcode);
            //             }
            //         },
            //         _ => ()
            //     }
            // }
        }
    }
}
