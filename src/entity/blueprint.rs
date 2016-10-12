use ::physics::{PhysicsComponent, CollisionModel};
use ::render::{SceneRenderable};
use ::core::Transform;

pub struct EntityBlueprint {
    pub physics: Option<PhysicsComponent>,
    pub collision: Option<CollisionModel>,
    pub renderable: Option<SceneRenderable>,
    pub transform: Option<Transform>
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
}