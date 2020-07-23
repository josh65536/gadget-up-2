use cgmath::BaseNum;
use cgmath::{Deg, Matrix4, Point3, Rad, Vector2, Vector3, Vector4};
use std::ops::Neg;

pub const TAUf32: f32 = std::f32::consts::PI * 2.0;
pub const TAUf64: f64 = std::f64::consts::PI * 2.0;

pub type Vec2i = Vector2<isize>;
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

    /// Rotates the vector 90 degrees clockwise
    fn right_cw(self) -> Self;

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

    fn right_cw(self) -> Self {
        -self.right_ccw()
    }

    fn dot_ex(self, other: Self) -> S {
        self.x * other.x + self.y * other.y
    }
}
