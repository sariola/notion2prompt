use super::ValidationError;
use std::ops::Deref;

/// Compile-time validated collections with const generics
#[derive(Debug, Clone, PartialEq)]
pub struct BoundedVec<T, const MIN: usize, const MAX: usize> {
    items: Vec<T>,
}

impl<T, const MIN: usize, const MAX: usize> BoundedVec<T, MIN, MAX> {
    /// Create a new bounded vector with validation
    pub fn new(items: Vec<T>) -> Result<Self, ValidationError> {
        if items.len() < MIN || items.len() > MAX {
            return Err(ValidationError::BoundsViolation {
                actual: items.len(),
                min: MIN,
                max: MAX,
            });
        }
        Ok(Self { items })
    }

    /// Create from a slice
    pub fn from_slice(slice: &[T]) -> Result<Self, ValidationError>
    where
        T: Clone,
    {
        Self::new(slice.to_vec())
    }

    /// Get as slice
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }

    /// Get mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.items
    }

    /// Into inner vector
    pub fn into_inner(self) -> Vec<T> {
        self.items
    }

    /// Length of the vector
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Iterate over items
    pub fn iter(&self) -> std::slice::Iter<T> {
        self.items.iter()
    }

    /// Iterate mutably over items
    pub fn iter_mut(&mut self) -> std::slice::IterMut<T> {
        self.items.iter_mut()
    }
}

impl<T, const MIN: usize, const MAX: usize> Deref for BoundedVec<T, MIN, MAX> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T, const MIN: usize, const MAX: usize> IntoIterator for BoundedVec<T, MIN, MAX> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<'a, T, const MIN: usize, const MAX: usize> IntoIterator for &'a BoundedVec<T, MIN, MAX> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

/// Non-empty vector type
#[allow(dead_code)]
pub type NonEmptyVec<T> = BoundedVec<T, 1, { usize::MAX }>;

/// Limited size vector for API constraints
#[allow(dead_code)]
pub type LimitedVec<T, const MAX: usize> = BoundedVec<T, 0, MAX>;

/// Bounded integer type with compile-time constraints
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct BoundedU32<const MIN: u32, const MAX: u32>(u32);

impl<const MIN: u32, const MAX: u32> BoundedU32<MIN, MAX> {
    /// Create a new bounded integer
    #[allow(dead_code)]
    pub fn new(value: u32) -> Result<Self, ValidationError> {
        if value < MIN || value > MAX {
            return Err(ValidationError::OutOfBounds {
                value,
                min: MIN,
                max: MAX,
            });
        }
        Ok(Self(value))
    }

    /// Get the inner value
    #[allow(dead_code)]
    pub fn get(&self) -> u32 {
        self.0
    }

    /// Saturating add
    #[allow(dead_code)]
    pub fn saturating_add(&self, other: u32) -> Self {
        Self((self.0 + other).min(MAX))
    }

    /// Saturating sub
    #[allow(dead_code)]
    pub fn saturating_sub(&self, other: u32) -> Self {
        Self((self.0.saturating_sub(other)).max(MIN))
    }
}

impl<const MIN: u32, const MAX: u32> Default for BoundedU32<MIN, MAX> {
    fn default() -> Self {
        Self(MIN)
    }
}

/// Common bounded types
#[allow(dead_code)]
pub type Percentage = BoundedU32<0, 100>;
#[allow(dead_code)]
pub type Priority = BoundedU32<1, 10>;
#[allow(dead_code)]
pub type ConcurrencyLimit = BoundedU32<1, 100>;
#[allow(dead_code)]
pub type DepthLimit = BoundedU32<1, 50>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_vec() {
        // Valid case
        let vec: BoundedVec<i32, 1, 5> = BoundedVec::new(vec![1, 2, 3]).unwrap();
        assert_eq!(vec.len(), 3);

        // Too few items
        let result: Result<BoundedVec<i32, 2, 5>, _> = BoundedVec::new(vec![1]);
        assert!(result.is_err());

        // Too many items
        let result: Result<BoundedVec<i32, 1, 3>, _> = BoundedVec::new(vec![1, 2, 3, 4]);
        assert!(result.is_err());
    }

    #[test]
    fn test_non_empty_vec() {
        let vec: NonEmptyVec<&str> = NonEmptyVec::new(vec!["hello"]).unwrap();
        assert_eq!(vec.len(), 1);

        let result: Result<NonEmptyVec<&str>, _> = NonEmptyVec::new(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_bounded_u32() {
        let percentage = Percentage::new(50).unwrap();
        assert_eq!(percentage.get(), 50);

        assert!(Percentage::new(101).is_err());

        let p2 = percentage.saturating_add(60);
        assert_eq!(p2.get(), 100);

        let p3 = percentage.saturating_sub(60);
        assert_eq!(p3.get(), 0);
    }
}
