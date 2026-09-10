use super::*;

/// Relation component recording that an agent currently detects an
/// obstacle at a given entity.
#[derive(Clone, Debug)]
pub struct Obstacle {
    pub(crate) shape_hit: ShapeHitData,
}

/// The filter used to determine which entities can be obstacles for the given agent
#[derive(
    Component, Reflect, Clone, Copy, Debug, Deref, DerefMut, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ObstacleFilter(pub LayerMask);

/// Configuration for how an agent detects nearby obstacles.

#[derive(Component, Clone, Debug)]
pub struct ObstacleDetection {
    /// The collider shape used to detect obstacles. Size this to the
    /// agent's desired detection radius (e.g. `Collider::sphere(radius)`),
    /// not to the agent's own physical footprint.
    pub shape: Collider,
    /// The local rotation applied to `shape` before casting.
    pub rotation: Quat,
    /// Extra configuration
    pub config: ShapeCastConfig,
    /// Direction the shape is cast in.
    pub direction: Dir3,
}

impl ObstacleDetection {
    /// Creates a new detection config from a shape, using an arbitrary
    /// default rotation and direction and the given cast config.
    pub fn new(shape: Collider, config: ShapeCastConfig) -> Self {
        Self {
            shape,
            rotation: Quat::IDENTITY,
            config,
            direction: Dir3::X,
        }
    }

    /// Sets the detection shape. Should be sized to the agent's intended
    /// detection radius, not its physical collider.
    pub fn with_shape(mut self, shape: Collider) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the local rotation applied to the detection shape.
    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        self.rotation = rotation;
        self
    }

    /// Sets the cast configuration (`max_hits`, `max_distance`, etc.).
    pub fn with_config(mut self, config: ShapeCastConfig) -> Self {
        self.config = config;
        self
    }

    /// Sets the cast direction. Only meaningfully affects results when
    /// `config.max_distance` is large enough for the shape to travel
    /// before overlapping something.
    pub fn with_direction(mut self, direction: Dir3) -> Self {
        self.direction = direction;
        self
    }
}

impl Obstacle {
    /// Returns the underlying shapecast hit data for this obstacle.
    pub const fn shape_hit(&self) -> &ShapeHitData {
        &self.shape_hit
    }
}
