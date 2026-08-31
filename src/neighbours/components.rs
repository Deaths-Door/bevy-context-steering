use super::*;

/// A relationship marker for a neighbourhood of a agent
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Neighbour;

/// The filter used to determine which entities can be neighbors for the given agent
#[derive(
    Component, Reflect, Clone, Copy, Debug, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct NeighbourhoodFilter(LayerMask);

#[derive(Component, Reflect, Clone, Copy, Debug, Deref, DerefMut, PartialEq)]
pub struct NeighbourhoodBounds(ColliderAabb);

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
