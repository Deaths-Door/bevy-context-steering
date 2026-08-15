use super::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Debug, Reflect, Deref, DerefMut, Clone, Copy, PartialEq, PartialOrd)]
pub struct ClusterWeight(pub f32);

impl Default for ClusterWeight {
    fn default() -> Self {
        Self(1.0)
    }
}

impl Add for ClusterWeight {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Add<f32> for ClusterWeight {
    type Output = Self;
    #[inline]
    fn add(self, rhs: f32) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Add<ClusterWeight> for f32 {
    type Output = ClusterWeight;
    #[inline]
    fn add(self, rhs: ClusterWeight) -> Self::Output {
        ClusterWeight(self + rhs.0)
    }
}

impl AddAssign for ClusterWeight {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl AddAssign<f32> for ClusterWeight {
    #[inline]
    fn add_assign(&mut self, rhs: f32) {
        self.0 += rhs;
    }
}

impl Sub for ClusterWeight {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Sub<f32> for ClusterWeight {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: f32) -> Self::Output {
        Self(self.0 - rhs)
    }
}

impl SubAssign for ClusterWeight {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl SubAssign<f32> for ClusterWeight {
    #[inline]
    fn sub_assign(&mut self, rhs: f32) {
        self.0 -= rhs;
    }
}

impl Mul for ClusterWeight {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl Mul<f32> for ClusterWeight {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Mul<ClusterWeight> for f32 {
    type Output = ClusterWeight;
    #[inline]
    fn mul(self, rhs: ClusterWeight) -> Self::Output {
        ClusterWeight(self * rhs.0)
    }
}

impl MulAssign for ClusterWeight {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

impl MulAssign<f32> for ClusterWeight {
    #[inline]
    fn mul_assign(&mut self, rhs: f32) {
        self.0 *= rhs;
    }
}
impl Div for ClusterWeight {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0 / rhs.0)
    }
}

impl Div<f32> for ClusterWeight {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl DivAssign for ClusterWeight {
    #[inline]
    fn div_assign(&mut self, rhs: Self) {
        self.0 /= rhs.0;
    }
}

impl DivAssign<f32> for ClusterWeight {
    #[inline]
    fn div_assign(&mut self, rhs: f32) {
        self.0 /= rhs;
    }
}

impl Neg for ClusterWeight {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}