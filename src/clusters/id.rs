use std::hash::{BuildHasher, Hash, Hasher};

use bevy::platform::hash::FixedHasher;

use super::*;

/// A unique identifier for a logical group of entities.
#[derive(Reflect, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct ClusterId(u64);

impl ClusterId {
    pub fn new<T: Hash>(value: T) -> Self {
        let mut state = FixedHasher::default().build_hasher();
        value.hash(&mut state);
        Self(state.finish())
    }
}

impl From<u64> for ClusterId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}
