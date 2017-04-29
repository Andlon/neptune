use nalgebra::Vector3;

pub enum ForceGenerator {
    UniformAccelerationField {
        acceleration: Vector3<f64>
    },
}
