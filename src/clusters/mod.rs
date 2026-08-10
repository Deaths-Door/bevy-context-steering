mod id;
mod relations;

pub use id::*;
pub use relations::*;

use super::*;
use bevy::{ecs::entity::EntityHashSet, platform::collections::HashMap};
use std::hash::{DefaultHasher, Hash, Hasher};


/// A global record of all active groups and their aggregate data.
#[derive(Resource, Default, Deref)]
pub struct ClusterRegistery {
    clusters: HashMap<ClusterId, ClusterData>,
}

#[derive(Default, Reflect)]
pub struct ClusterData {
    members: EntityHashSet,
    average_centre: Vec3,
    average_velocity: Vec3,
}

impl<T> From<T> for ClusterData
where
    T: IntoIterator<Item = Entity>,
{
    fn from(value: T) -> Self {
        Self {
            members: EntityHashSet::from_iter(value),
            average_centre: Vec3::ZERO,
            average_velocity: Vec3::ZERO,
        }
    }
}

impl ClusterData {
    pub const fn members(&self) -> &EntityHashSet {
        &self.members
    }

    pub const fn average_centre(&self) -> Vec3 {
        self.average_centre
    }

    pub const fn average_velocity(&self) -> Vec3 {
        self.average_velocity
    }
}
