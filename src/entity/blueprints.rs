use ::entity::EntityBlueprint;
use render::{unit_sphere_renderable, box_renderable};
use geometry::{Sphere, Cuboid};
use physics::{Mass, RigidBody, DynamicRigidBody, DynamicBodyState, CollisionModel};
use cgmath::{Vector3};
use core::Transform;
use nalgebra;
use interop;

/// A blueprint of a sphere with zero velocity.
pub fn sphere(sphere: Sphere<f64>, mass: f64, num_subdivisions: u32) -> EntityBlueprint {
    let mut blueprint = EntityBlueprint::empty();
    let r = sphere.radius;
    let inertia_tensor = (2.0 / 5.0) * mass * r * r * nalgebra::Matrix3::identity();
    let inv_inertia_tensor = inertia_tensor.try_inverse()
                                .expect("Provided inertia tensor must be invertible.");
    let scale = Vector3::new(sphere.radius, sphere.radius, sphere.radius);

    let rb_state = DynamicBodyState {
        position: sphere.center,
        .. DynamicBodyState::default()
    };

    // Temporary, for interop between cgmath and nalgebra types
    let pos_cgmath = interop::nalgebra_point3_to_cgmath(&sphere.center);

    blueprint.renderable = Some(unit_sphere_renderable(num_subdivisions));
    blueprint.transform = Some(Transform { position: pos_cgmath, scale: scale, .. Transform::default() });
    blueprint.collision = Some(CollisionModel::Sphere(
        Sphere { center: nalgebra::Point3::origin(), .. sphere }));
    blueprint.rigid_body = Some(RigidBody::Dynamic(DynamicRigidBody {
        state: rb_state.clone(),
        prev_state: rb_state,
        inv_inertia_body: inv_inertia_tensor,
        mass: Mass::new(mass),
        .. DynamicRigidBody::default()
    }));

    blueprint
}

pub fn cuboid(cuboid: Cuboid<f64>, mass: f64) -> EntityBlueprint {
    let mut blueprint = EntityBlueprint::empty();

    let extents = 2.0 * cuboid.half_size;
    let inertia_tensor_diagonal = nalgebra::Vector3::new(extents.y * extents.y + extents.z * extents.z,
                                               extents.x * extents.x + extents.z * extents.z,
                                               extents.x * extents.x + extents.y * extents.y);
    let inertia_tensor = (mass / 12.0) * nalgebra::Matrix3::from_diagonal(&inertia_tensor_diagonal);
    let inv_inertia_tensor = inertia_tensor.try_inverse()
                                .expect("Provided inertia tensor must be invertible.");

    let rb_state = DynamicBodyState {
        position: cuboid.center,
        orientation: cuboid.rotation,
        .. DynamicBodyState::default()
    };

    blueprint.renderable = Some(box_renderable(cuboid.half_size.x as f32, cuboid.half_size.y as f32, cuboid.half_size.z as f32));
    // Note: Ignore orientation in Cuboid and instead model that through the transform component
    blueprint.collision = Some(CollisionModel::Cuboid(Cuboid {
        center: nalgebra::Point3::origin(),
        half_size: cuboid.half_size,
        rotation: nalgebra::UnitQuaternion::identity()
    }));
    blueprint.rigid_body = Some(RigidBody::Dynamic(DynamicRigidBody {
        state: rb_state.clone(),
        prev_state: rb_state,
        inv_inertia_body: inv_inertia_tensor,
        mass: Mass::new(mass),
        .. DynamicRigidBody::default()
    }));
    blueprint.transform = Some(Transform {
        position: interop::nalgebra_point3_to_cgmath(&cuboid.center),
        orientation: interop::nalgebra_unit_quat_to_cgmath(&cuboid.rotation),
        .. Transform::default()
    });

    blueprint
}
