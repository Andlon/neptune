use ::physics::{RigidBody, StaticRigidBody, CollisionModel};
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

    pub fn make_static(mut self) -> Self {
        if let Some(RigidBody::Dynamic(rb)) = self.rigid_body {
            let static_rb = StaticRigidBody {
                position: rb.state.position,
                orientation: rb.state.orientation
            };
            self.rigid_body = Some(RigidBody::Static(static_rb));
        }
        self
    }
}
