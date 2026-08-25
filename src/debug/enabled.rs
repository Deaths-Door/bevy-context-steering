use super::*;

/// Marker component attached to an agent to enable visual steering debug.
///
/// Remove this component from an entity to completely disable gizmo rendering for it.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnableSteeringDebug;
