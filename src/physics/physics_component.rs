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
pub struct DynamicBodyState {
    pub position: Point3<f64>,
    pub orientation: UnitQuaternion<f64>,
    pub velocity: Vector3<f64>,
    pub angular_momentum: Vector3<f64>,
    pub acceleration: Vector3<f64>
}

// #[derive(Clone, Debug)]
// pub struct RigidBody {
//     pub state: RigidBodyState,
//     pub prev_state: RigidBodyState,
//     pub mass: Mass,
//     pub inv_inertia_body: Matrix3<f64>
// }

#[derive(Clone, Debug)]
pub struct StaticRigidBody {
    position: Point3<f64>,
    orientation: UnitQuaternion<f64>
}

#[derive(Clone, Debug)]
pub struct DynamicRigidBody {
    pub state: DynamicBodyState,
    pub prev_state: DynamicBodyState,
    pub mass: Mass,
    pub inv_inertia_body: Matrix3<f64>
}

#[derive(Clone, Debug)]
pub enum RigidBody {
    Static(StaticRigidBody),
    Dynamic(DynamicRigidBody)
}

impl Default for DynamicBodyState {
    fn default() -> Self {
        DynamicBodyState {
            position: Point3::origin(),
            orientation: UnitQuaternion::identity(),
            velocity: nalgebra::zero::<Vector3<_>>(),
            angular_momentum: nalgebra::zero::<Vector3<_>>(),
            acceleration: nalgebra::zero::<Vector3<_>>()
        }
    }
}

impl RigidBody {
    pub fn position(&self) -> Point3<f64> {
        match self {
            &RigidBody::Static(ref rb) => { rb.position },
            &RigidBody::Dynamic(ref rb) => { rb.state.position }
        }
    }

    pub fn orientation(&self) -> UnitQuaternion<f64> {
        match self {
            &RigidBody::Static(ref rb) => { rb.orientation },
            &RigidBody::Dynamic(ref rb) => { rb.state.orientation }
        }
    }
}

impl Default for DynamicRigidBody {
    fn default() -> Self {
        DynamicRigidBody {
            state: DynamicBodyState::default(),
            prev_state: DynamicBodyState::default(),
            mass: Mass::zero(),
            inv_inertia_body: Matrix3::identity()
        }
    }
}
