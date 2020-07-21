use cgmath::prelude::*;
use cgmath::BaseNum;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector2, Vector3, Vector4};
use std::ops::Neg;

pub const TAUf32: f32 = std::f32::consts::PI * 2.0;
pub const TAUf64: f64 = std::f64::consts::PI * 2.0;

pub type Vec2i = Vector2<i32>;
pub type Vec2 = Vector2<f64>;
pub type Vec3 = Vector3<f64>;
pub type Vec4 = Vector4<f64>;
pub type Mat4 = Matrix4<f64>;
pub type Pt3 = Point3<f64>;
pub type Degrees = Deg<f64>;
pub type Radians = Rad<f64>;

pub trait Vector2Ex<S: BaseNum + Neg> {
    /// Rotates the vector 90 degrees counterclockwise
    fn right_ccw(self) -> Self;

    /// Dot product; not restricted to floats
    fn dot_ex(self, other: Self) -> S;
}

impl<S> Vector2Ex<S> for Vector2<S>
where
    S: BaseNum + Neg<Output = S>,
{
    fn right_ccw(self) -> Self {
        Vector2::new(-self.y, self.x)
    }

    fn dot_ex(self, other: Self) -> S {
        self.x * other.x + self.y * other.y
    }
}

/// Useful for rendering
pub trait ToArray {
    type Array;

    fn to_array(&self) -> Self::Array;
}

impl ToArray for Vector2<f32> {
    type Array = [f32; 2];

    fn to_array(&self) -> Self::Array {
        [self.x, self.y]
    }
}

impl ToArray for Vector3<f32> {
    type Array = [f32; 3];

    fn to_array(&self) -> Self::Array {
        [self.x, self.y, self.z]
    }
}

impl ToArray for Vector4<f32> {
    type Array = [f32; 4];

    fn to_array(&self) -> Self::Array {
        [self.x, self.y, self.z, self.w]
    }
}

impl ToArray for Matrix4<f32> {
    type Array = [f32; 16];

    #[rustfmt::skip]
    fn to_array(&self) -> Self::Array {
        [
            self.x.x, self.x.y, self.x.z, self.x.w,
            self.y.x, self.y.y, self.y.z, self.y.w,
            self.z.x, self.z.y, self.z.z, self.z.w,
            self.w.x, self.w.y, self.w.z, self.w.w,
        ]
    }
}
