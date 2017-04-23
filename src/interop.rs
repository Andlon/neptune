use cgmath;
use nalgebra;

pub fn cgmath_vector3_to_nalgebra<T>(v: &cgmath::Vector3<T>)
    -> nalgebra::Vector3<T>
    where T: nalgebra::Scalar
{
    nalgebra::Vector3::new(v[0], v[1], v[2])
}

pub fn cgmath_quat_to_nalgebra(v: &cgmath::Quaternion<f64>)
    -> nalgebra::Quaternion<f64>
{
    nalgebra::Quaternion::new(v.s, v.v.x, v.v.y, v.v.z)
}

pub fn cgmath_point3_to_nalgebra<T>(v: &cgmath::Point3<T>)
    -> nalgebra::Point3<T>
    where T: nalgebra::Scalar
{
    nalgebra::Point3::new(v.x, v.y, v.z)
}

pub fn nalgebra_unit_quat_to_cgmath(v: &nalgebra::UnitQuaternion<f64>)
    -> cgmath::Quaternion<f64>
{
    let v = v.unwrap();
    let w = v.scalar();
    let xyz = v.vector();
    let x = xyz[0];
    let y = xyz[1];
    let z = xyz[2];
    cgmath::Quaternion::new(w, x, y, z)
}

pub fn nalgebra_point3_to_cgmath<T>(v: &nalgebra::Point3<T>)
    -> cgmath::Point3<T>
    where T: nalgebra::Scalar + cgmath::BaseNum
{
    cgmath::Point3::new(v[0], v[1], v[2])
}

/// Stop-gap solution for inverting 3x3 matrices
/// with nalgebra, since nalgebra uses an inappropriate
/// approximate check against the determinant to determine
/// if the matrix is invertible, which makes the inversion
/// fail in cases when the matrix is perfectly invertible.
///
/// The following code is a copy of the nalgebra code
/// for 3x3 inversion, but with a modified determinant check.
pub fn try_3x3_inverse<T>(mut mat: nalgebra::Matrix3<T>)
    -> Result<nalgebra::Matrix3<T>, ()>
    where T: nalgebra::Scalar + ::num::Float
{
    unsafe {
        let m11 = *mat.get_unchecked(0, 0);
        let m12 = *mat.get_unchecked(0, 1);
        let m13 = *mat.get_unchecked(0, 2);

        let m21 = *mat.get_unchecked(1, 0);
        let m22 = *mat.get_unchecked(1, 1);
        let m23 = *mat.get_unchecked(1, 2);

        let m31 = *mat.get_unchecked(2, 0);
        let m32 = *mat.get_unchecked(2, 1);
        let m33 = *mat.get_unchecked(2, 2);


        let minor_m12_m23 = m22 * m33 - m32 * m23;
        let minor_m11_m23 = m21 * m33 - m31 * m23;
        let minor_m11_m22 = m21 * m32 - m31 * m22;

        let determinant = m11 * minor_m12_m23 -
                            m12 * minor_m11_m23 +
                            m13 * minor_m11_m22;

        if determinant == T::zero() {
            Err(())
        } else {
            *mat.get_unchecked_mut(0, 0) = minor_m12_m23 / determinant;
            *mat.get_unchecked_mut(0, 1) = (m13 * m32 - m33 * m12) / determinant;
            *mat.get_unchecked_mut(0, 2) = (m12 * m23 - m22 * m13) / determinant;

            *mat.get_unchecked_mut(1, 0) = -minor_m11_m23 / determinant;
            *mat.get_unchecked_mut(1, 1) = (m11 * m33 - m31 * m13) / determinant;
            *mat.get_unchecked_mut(1, 2) = (m13 * m21 - m23 * m11) / determinant;

            *mat.get_unchecked_mut(2, 0) = minor_m11_m22  / determinant;
            *mat.get_unchecked_mut(2, 1) = (m12 * m31 - m32 * m11) / determinant;
            *mat.get_unchecked_mut(2, 2) = (m11 * m22 - m21 * m12) / determinant;

            Ok(mat)
        }
    }
}
