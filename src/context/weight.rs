use super::*;

/// A directional weight pair representing the desirability and risk of a specific vector.
#[derive(Default, Clone)]
pub struct Weight {
    /// How much the agent wants to move in this direction.
    pub(super) interest: f32,
    /// How much the agent wants to avoid moving in this direction.
    pub(super) danger: f32,

    /// Desired speed in this direction,
    /// `None` = no velocity preference written for this slot (excluded from
    /// resolve). `Some(vec3)` = a behaviour explicitly wants this velocity speed here.
    pub(super) velocity: Option<Vec3>,
}

impl Weight {
    /// Creates a new `Weight` instance with specified interest and danger values.
    pub const fn new(interest: f32, danger: f32) -> Self {
        Self {
            interest,
            danger,
            velocity: None,
        }
    }

    /// Returns the interest value.
    pub const fn interest(&self) -> f32 {
        self.interest
    }

    /// Returns the danger value.
    pub const fn danger(&self) -> f32 {
        self.danger
    }

    /// Sets the interest value.
    pub const fn set_interest(&mut self, interest: f32) {
        self.interest = interest;
    }

    /// Sets the danger value.
    pub const fn set_danger(&mut self, danger: f32) {
        self.danger = danger;
    }

    /// Gets the desired speed in this direction.
    pub const fn velocity(&self) -> Option<&Vec3> {
        self.velocity.as_ref()
    }

    /// Sets the desired speed in this direction.
    pub const fn set_velocity(&mut self, velocity: Option<Vec3>) {
        self.velocity = velocity;
    }
}
