use super::*;

/// A relationship marker for a neighbourhood of a agent
///Stored on the agent, pointing at the neighbour entity. All fields are
/// expressed relative to the agent's own position
#[derive(Clone, Copy, Debug, Default)]
pub struct Neighbour {
    /// Distance between the agent's closest surface point and the
    /// neighbour's closest surface point.
    pub(crate) distance: f32,
    /// Offset from the agent's position to the closest point on the
    /// agent's own collider (relative to the agent, not world space).
    pub(crate) agent_point: Vec3,
    /// Offset from the agent's position to the closest point on the
    /// neighbour's collider (relative to the agent, not world space).
    pub(crate) hit_point: Vec3,
}

/// The filter used to determine which entities can be neighbors for the given agent
#[derive(
    Component, Reflect, Clone, Copy, Debug, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct NeighbourhoodFilter(pub LayerMask);

/// How far around the agent its neighbourhood extends. Authored/configured,
/// relative to the agent — does not itself move.
#[derive(Component, Reflect, Clone, Copy, Debug, Deref, DerefMut, PartialEq)]
pub struct NeighbourhoodExtents(pub Vec3);

impl Neighbour {
    /// Distance between the agent's and neighbour's closest surface points.
    pub const fn distance(&self) -> f32 {
        self.distance
    }

    /// Offset (relative to the agent) to the closest point on the agent's
    /// own collider.
    pub const fn agent_point(&self) -> Vec3 {
        self.agent_point
    }

    /// Offset (relative to the agent) to the closest point on the
    /// neighbour's collider.
    pub const fn hit_point(&self) -> Vec3 {
        self.hit_point
    }
}
impl From<LayerMask> for NeighbourhoodFilter {
    fn from(value: LayerMask) -> Self {
        Self(value)
    }
}

impl From<NeighbourhoodFilter> for LayerMask {
    fn from(value: NeighbourhoodFilter) -> Self {
        value.0
    }
}

impl From<Vec3> for NeighbourhoodExtents {
    fn from(value: Vec3) -> Self {
        Self(value)
    }
}

impl From<NeighbourhoodExtents> for Vec3 {
    fn from(value: NeighbourhoodExtents) -> Self {
        value.0
    }
}
