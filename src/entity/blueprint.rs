use ::physics::{RigidBody, CollisionModel};
use ::render::{SceneRenderable};
use ::core::Transform;

pub struct EntityBlueprint {
    pub rigid_body: Option<RigidBody>,
    pub collision: Option<CollisionModel>,
    pub renderable: Option<SceneRenderable>,
    pub transform: Option<Transform>
}

impl EntityBlueprint {
    pub fn empty() -> Self {
        EntityBlueprint {
            rigid_body: None,
            collision: None,
            renderable: None,
            transform: None
        }
    }

    /// Turns the blueprint into a blueprint for a static object, effectively
    /// removing the rigid body component.
    pub fn make_static(mut self) -> Self {
        self.rigid_body = None;
        self
    }
}
