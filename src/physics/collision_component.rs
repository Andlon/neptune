use cgmath::Point3;

pub enum CollisionModel {
    Sphere {
        radius: f64,
        position: Point3<f64>
    }
}

pub type CollisionComponentId = usize;
pub struct CollisionComponentStore {

}