use nalgebra;
use nalgebra::{Point3, Vector3, Matrix3, UnitQuaternion};

#[derive(Copy, Clone, Debug)]
pub struct Mass {
    value: f64
}

impl Mass {
    pub fn new(mass: f64) -> Mass {
        assert!(mass.is_finite() && mass >= 0.0,
            "Mass must be a non-negative number.");
        Mass {
            value: mass
        }
    }

    pub fn zero() -> Mass {
        Mass {
            value: 0.0
        }
    }

    pub fn value(&self) -> f64 {
        self.value
    }
}

#[derive(Clone, Debug)]
pub struct RigidBodyState {
    pub position: Point3<f64>,
    pub orientation: UnitQuaternion<f64>,
    pub velocity: Vector3<f64>,
    pub angular_momentum: Vector3<f64>,
    pub acceleration: Vector3<f64>
}

#[derive(Clone, Debug)]
pub struct RigidBody {
    pub state: RigidBodyState,
    pub prev_state: RigidBodyState,
    pub mass: Mass,
    pub inv_inertia_body: Matrix3<f64>
}

impl Default for RigidBodyState {
    fn default() -> Self {
        RigidBodyState {
            position: Point3::origin(),
            orientation: UnitQuaternion::identity(),
            velocity: nalgebra::zero::<Vector3<_>>(),
            angular_momentum: nalgebra::zero::<Vector3<_>>(),
            acceleration: nalgebra::zero::<Vector3<_>>()
        }
    }
}

impl Default for RigidBody {
    fn default() -> Self {
        RigidBody {
            state: RigidBodyState::default(),
            prev_state: RigidBodyState::default(),
            mass: Mass::zero(),
            inv_inertia_body: Matrix3::identity()
        }
    }
}
