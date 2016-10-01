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
}