use bevy_many_relationships::{ManyRelatedEntityCommands, OnManyRelationshipAdded};

use super::*;

// 2. Relationship edge payload managed by bevy_many_relationships
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ClusterMember;

// 3. Cluster dummy root component
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Reflect, Deref)]
#[component(immutable)]
pub struct Cluster(pub ClusterId);
