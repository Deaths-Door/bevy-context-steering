use bevy::{platform::collections::HashMap, prelude::*};
use std::any::TypeId;

/// Component attached to an agent to configure its steering visual debug gizmos.
#[derive(Component, Debug, Clone, Deref, DerefMut)]
pub struct SteeringDebugOptions {
    /// Per-behavior configuration map indexed by the behavior's Rust [`TypeId`].
    #[deref]
    pub behaviors: HashMap<TypeId, SteeringDebugStyle>,

    /// Visual channel for the chosen steering vector (normalized direction).
    pub resultant_direction: Channel,
    /// Visual channel for the final calculated velocity vector.
    pub resultant_velocity: Channel,
}

/// Visual style and toggle settings for a single steering behavior's debug channels.
#[derive(Debug, Clone)]
pub struct SteeringDebugStyle {
    /// Master toggle for this specific behavior's gizmos.
    pub enabled: bool,
    /// Visual channel for slot interest weights ($0.0 \dots 1.0$).
    pub interest: Channel,
    /// Visual channel for slot danger weights ($0.0 \dots 1.0$).
    pub danger: Channel,
    /// Visual channel for the current entity velocity vector.
    pub velocity: Channel,
}

/// Configuration for a single visual channel (e.g., Interest, Danger).
#[derive(Debug, Clone)]
pub struct Channel {
    /// Whether this specific channel is rendered.
    pub enabled: bool,

    /// Base color corresponding to maximum weight .
    pub base_color: Color,

    /// Ray length in world units
    pub length: f32,

    /// Line thickness for rendered gizmos.
    pub thickness: f32,

    /// Minimum weight required to draw the ray.
    ///
    /// Rays below this value are skipped entirely to save rendering CPU/GPU overhead.
    pub threshold: f32,
}

impl SteeringDebugOptions {
    /// Registers a custom configuration for a specific behavior type `T`.
    pub fn register_behavior<T: 'static>(&mut self, style: SteeringDebugStyle) -> &mut Self {
        self.behaviors.insert(TypeId::of::<T>(), style);
        self
    }

    /// Gets the configuration for behavior `T`, falling back to `None` if unconfigured.
    pub fn get_style<T: 'static>(&self) -> Option<&SteeringDebugStyle> {
        self.behaviors.get(&TypeId::of::<T>())
    }
}

impl Channel {
    /// Creates a enabled channel).
    pub fn new(base_color: Color, length: f32) -> Self {
        Self {
            enabled: true,
            base_color,
            length,
            thickness: 2.0,
            threshold: 0.05,
        }
    }

    /// Evaluates a weight ($0.0 \dots 1.0$) against channel settings.
    ///
    /// Returns `Some((Color, RayLength))` scaled by weight if enabled and above threshold,
    /// or `None` if the ray should be pruned.
    pub fn using_weight(&self, weight: f32) -> Option<(Color, f32)> {
        if !self.enabled || weight < self.threshold {
            return None;
        }

        let clamped = weight.clamp(0.0, 1.0);

        // Scales RGB values towards 0.0 (black) based on weight intensity
        let linear_color = self.base_color.to_linear();
        let color = Color::LinearRgba(LinearRgba {
            red: linear_color.red * clamped,
            green: linear_color.green * clamped,
            blue: linear_color.blue * clamped,
            alpha: linear_color.alpha,
        });

        let length = self.length * clamped;
        Some((color, length))
    }
}

impl Default for SteeringDebugStyle {
    fn default() -> Self {
        Self {
            enabled: true,
            interest: Channel::new(Color::srgb(0.0, 1.0, 0.0), 1.5), // Green
            danger: Channel::new(Color::srgb(1.0, 0.0, 0.0), 1.5),   // Red
            velocity: Channel::new(Color::srgb(0.0, 0.5, 1.0), 2.0), // Cyan
        }
    }
}

impl Default for SteeringDebugOptions {
    fn default() -> Self {
        Self {
            behaviors: Default::default(),
            resultant_direction: Channel::new(Color::srgb(1.0, 1.0, 0.0), 2.5), // Yellow
            resultant_velocity: Channel::new(Color::srgb(1.0, 0.5, 0.0), 2.5),  // Orange
        }
    }
}
