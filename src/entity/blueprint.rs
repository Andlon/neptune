use ::physics::{PhysicsComponent, CollisionModel};
use ::render::{SceneRenderable, SceneTransform};

pub struct EntityBlueprint {
    pub physics: Option<PhysicsComponent>,
    pub collision: Option<CollisionModel>,
    pub renderable: Option<SceneRenderable>,
    pub transform: Option<SceneTransform>
}

impl EntityBlueprint {
    pub fn empty() -> Self {
        EntityBlueprint {
            physics: None,
            collision: None,
            renderable: None,
            transform: None
        }
    }

    pub fn with_physics_component(mut self, physics_component: PhysicsComponent) -> Self {
        self.physics = Some(physics_component);
        self
    }

    pub fn with_collision_model(mut self, model: CollisionModel) -> Self {
        self.collision = Some(model);
        self
    }

    pub fn with_renderable(mut self, renderable: SceneRenderable) -> Self {
        self.renderable = Some(renderable);
        self
    }

    pub fn with_transform(mut self, transform: SceneTransform) -> Self {
        self.transform = Some(transform);
        self
    }
}