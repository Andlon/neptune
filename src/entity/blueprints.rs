use ::entity::EntityBlueprint;
use render::{SceneTransform, unit_sphere_renderable, box_renderable};
use geometry::{Sphere, Cuboid};
use physics::{PhysicsComponent, CollisionModel};
use cgmath::{Matrix3, SquareMatrix, Point3, Vector3, EuclideanSpace};

/// A blueprint of a sphere with zero velocity.
pub fn sphere(sphere: Sphere<f64>, mass: f64, num_subdivisions: u32) -> EntityBlueprint {
    let mut blueprint = EntityBlueprint::empty();
    let r = sphere.radius;
    let inertia_tensor = (2.0 / 5.0) * mass * r * r * Matrix3::identity();
    let scale = sphere.radius as f32;
    let scale = Vector3::new(scale, scale, scale);

    blueprint.renderable = Some(unit_sphere_renderable(num_subdivisions));
    blueprint.transform = Some(SceneTransform { scale: scale, .. SceneTransform::default() });
    blueprint.collision = Some(CollisionModel::Sphere(Sphere { center: Point3::origin(), .. sphere }));
    blueprint.physics = Some(PhysicsComponent {
        position: sphere.center,
        inertia_body: inertia_tensor,
        mass: mass,
        .. PhysicsComponent::default()
    });

    blueprint
}

pub fn cuboid(cuboid: Cuboid<f64>, mass: f64) -> EntityBlueprint {
    let mut blueprint = EntityBlueprint::empty();

    let extents = 2.0 * cuboid.half_size;
    let inertia_tensor_diagonal = Vector3::new(extents.y * extents.y + extents.z * extents.z,
                                               extents.x * extents.x + extents.z * extents.z,
                                               extents.x * extents.x + extents.y * extents.y);
    let inertia_tensor = (mass / 12.0) * Matrix3::from_diagonal(inertia_tensor_diagonal);

    blueprint.renderable = Some(box_renderable(cuboid.half_size.x as f32, cuboid.half_size.y as f32, cuboid.half_size.z as f32));
    blueprint.collision = Some(CollisionModel::Cuboid(Cuboid { center: Point3::origin(), .. cuboid }));
    blueprint.physics = Some(PhysicsComponent {
        position: cuboid.center,
        inertia_body: inertia_tensor,
        mass: mass,
        .. PhysicsComponent::default()
    });

    blueprint
}