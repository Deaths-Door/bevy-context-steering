use bevy::reflect::Reflect;

/// Combines the distance threshold (radius) with the attenuation curve (kind).
#[derive(Clone, Default, Debug, Reflect)]
pub enum Falloff {
    #[default]
    None,
    /// Stop behaviour once within/outside threshold
    Stop {
        threshold: f32,
    },
    Linear {
        threshold: f32,
    },
    Quadratic {
        threshold: f32,
    },
    Cubic {
        threshold: f32,
    },
    /// Smooth S-curve transition (3x² - 2x³), zero derivative at boundaries (0.0 and 1.0).
    /// Eliminates harsh light/vector cutoff edges.
    SmoothStep {
        threshold: f32,
    },

    /// Higher-order smooth curve (6x⁵ - 15x⁴ + 10x³), zero 1st and 2nd derivatives at boundaries.
    SmootherStep {
        threshold: f32,
    },

    /// Inverse square law (1 / (1 + distance²)), common for realistic physical attenuation.
    InverseSquare {
        threshold: f32,
    },

    /// Exponential falloff (e.g., 1 - exp(-k * x)), useful for rapid initial drop-off.
    Exponential {
        threshold: f32,
        exponent: f32,
    },
}

impl Falloff {
    /// Inwards falloff (e.g. Seek / Arrival).
    /// - Outside threshold (distance >= threshold): returns 1.0 (full force).
    /// - Inside threshold: attenuates smoothly down to 0.0 at distance = 0.0.
    pub fn inwards_factor(&self, distance: f32) -> f32 {
        let Some(threshold) = self.threshold() else {
            return 1.0;
        };

        if threshold <= f32::EPSILON {
            // Degenerate threshold: treat as an infinitely thin boundary at distance=0.
            return if distance <= f32::EPSILON { 0.0 } else { 1.0 };
        }
        if distance >= threshold {
            return 1.0;
        }

        // x ranges from 0.0 at center (distance = 0) to 1.0 at threshold boundary
        let x = (distance / threshold).clamp(0.0, 1.0);
        self.curve_factor(x)
    }

    /// Outwards falloff (e.g. Flee / Danger zone).
    /// - Outside threshold (distance >= threshold): returns 0.0 (out of range / safe).
    /// - Inside threshold: ramps up smoothly from 0.0 to 1.0 at distance = 0.0.
    pub fn outwards_factor(&self, distance: f32) -> f32 {
        let Some(threshold) = self.threshold() else {
            return 1.0;
        };

        if distance >= threshold {
            return 0.0;
        }

        // x ranges from 1.0 at center (distance = 0) down to 0.0 at threshold boundary
        let x = (1.0 - (distance / threshold)).clamp(0.0, 1.0);
        self.curve_factor(x)
    }

    /// Helper to extract the threshold value across variants.
    pub const fn threshold(&self) -> Option<f32> {
        match self {
            Self::None => None,
            Self::Stop { threshold }
            | Self::Linear { threshold }
            | Self::Quadratic { threshold }
            | Self::Cubic { threshold }
            | Self::SmoothStep { threshold }
            | Self::SmootherStep { threshold }
            | Self::InverseSquare { threshold }
            | Self::Exponential { threshold, .. } => Some(*threshold),
        }
    }

    /// Maps normalized progress x in [0.0, 1.0] onto the curve function.
    fn curve_factor(&self, x: f32) -> f32 {
        match self {
            Self::None => 1.0,
            Self::Stop { .. } => 0.0,
            Self::Linear { .. } => x,
            Self::Quadratic { .. } => x * x,
            Self::Cubic { .. } => x * x * x,
            Self::SmoothStep { .. } => x * x * (3.0 - 2.0 * x),
            Self::SmootherStep { .. } => x * x * x * (x * (x * 6.0 - 15.0) + 10.0),
            Self::InverseSquare { .. } => (2.0 * x * x) / (1.0 + x * x),
            Self::Exponential { exponent, .. } => {
                let k = *exponent;
                (1.0 - (-k * x).exp()) / (1.0 - (-k).exp())
            }
        }
    }
}
