#![forbid(unsafe_code)]
//! Data-oriented math primitives built on glam.
//!
//! The engine uses a right-handed world space with Y up and negative Z as forward.
//! This crate currently provides a validated axis-aligned bounding box and will grow
//! to include frusta and explicit coordinate-space transforms.
//!
//! # Examples
//!
//! ~~~rust
//! use engine_math::Aabb;
//! use glam::Vec3;
//! let bounds = Aabb::new(Vec3::ZERO, Vec3::splat(1.0)).unwrap();
//! assert!(bounds.contains_point(Vec3::splat(0.5)));
//! ~~~

use glam::Vec3;

/// Axis-aligned bounding box represented by minimum and maximum corners.
///
/// The box is closed on all faces. Construction rejects inverted and non-finite bounds.
///
/// # Examples
///
/// ~~~rust
/// use engine_math::Aabb;
/// use glam::Vec3;
/// let bounds = Aabb::new(Vec3::ZERO, Vec3::splat(2.0)).unwrap();
/// assert_eq!(bounds.center(), Vec3::ONE);
/// ~~~
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    /// Creates an AABB if every minimum component is at most its maximum.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// assert!(Aabb::new(Vec3::ZERO, Vec3::ONE).is_some());
    /// assert!(Aabb::new(Vec3::ONE, Vec3::ZERO).is_none());
    /// ~~~
    pub fn new(min: Vec3, max: Vec3) -> Option<Self> {
        if min.is_finite() && max.is_finite() && min.cmple(max).all() {
            Some(Self { min, max })
        } else {
            None
        }
    }

    /// Returns the minimum corner.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let bounds = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
    /// assert_eq!(bounds.min(), Vec3::ZERO);
    /// ~~~
    pub fn min(&self) -> Vec3 { self.min }

    /// Returns the maximum corner.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let bounds = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
    /// assert_eq!(bounds.max(), Vec3::ONE);
    /// ~~~
    pub fn max(&self) -> Vec3 { self.max }

    /// Returns the center point of the box.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let bounds = Aabb::new(Vec3::ZERO, Vec3::splat(2.0)).unwrap();
    /// assert_eq!(bounds.center(), Vec3::ONE);
    /// ~~~
    pub fn center(&self) -> Vec3 { (self.min + self.max) * 0.5 }

    /// Returns half the box dimensions on each axis.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let bounds = Aabb::new(Vec3::ZERO, Vec3::splat(2.0)).unwrap();
    /// assert_eq!(bounds.half_extents(), Vec3::ONE);
    /// ~~~
    pub fn half_extents(&self) -> Vec3 { (self.max - self.min) * 0.5 }

    /// Returns true when a point lies inside or on the box.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let bounds = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
    /// assert!(bounds.contains_point(Vec3::splat(0.5)));
    /// ~~~
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }

    /// Returns true when this box overlaps or touches another box.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let a = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
    /// let b = Aabb::new(Vec3::ONE, Vec3::splat(2.0)).unwrap();
    /// assert!(a.intersects(&b));
    /// ~~~
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }

    /// Returns the smallest box containing both input boxes.
    ///
    /// # Examples
    ///
    /// ~~~rust
    /// use engine_math::Aabb;
    /// use glam::Vec3;
    /// let a = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
    /// let b = Aabb::new(Vec3::splat(2.0), Vec3::splat(3.0)).unwrap();
    /// assert_eq!(a.union(&b).max(), Vec3::splat(3.0));
    /// ~~~
    pub fn union(&self, other: &Self) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }
}

#[cfg(test)]
mod tests {
    use super::Aabb;
    use glam::Vec3;

    #[test]
    fn rejects_inverted_and_non_finite_bounds() {
        assert!(Aabb::new(Vec3::ONE, Vec3::ZERO).is_none());
        assert!(Aabb::new(Vec3::ZERO, Vec3::splat(f32::NAN)).is_none());
    }

    #[test]
    fn containment_and_intersection_include_boundaries() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
        let b = Aabb::new(Vec3::ONE, Vec3::splat(2.0)).unwrap();
        assert!(a.contains_point(Vec3::ONE));
        assert!(a.intersects(&b));
    }

    #[test]
    fn union_contains_both_input_boxes() {
        let a = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
        let b = Aabb::new(Vec3::splat(2.0), Vec3::splat(3.0)).unwrap();
        let result = a.union(&b);
        assert!(result.contains_point(a.min()));
        assert!(result.contains_point(b.max()));
    }
}
