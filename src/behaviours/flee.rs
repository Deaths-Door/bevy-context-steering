use super::*;

/// Steering behavior that accelerates an agent away from a target position, line, or hazard zone.
///
/// `Flee` calculates a repelling vector pointed directly away from a spatial target point or line.
/// By configuring [`Self::axis_direction`], `Flee` unifies traditional **Point Fleeing**
/// and **Line / Hazard Evasion**.
///
/// - **Point Flee** (`axis_direction == Vec3::ZERO`): Repels the agent spherically away from `target + offset`.
/// - **Line / Rail Flee** (`axis_direction == unit vector`): Repels the agent perpendicularly away 
///   from an infinite 3D line passing through `target + offset` (useful for evading lasers or linear hazards).
///
/// Use [`Self::offset`] to position the repulsion center relative to an entity or danger zone.
///
/// The intensity of this force decays over distance according to the configured [`Falloff`].
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_context_steering::{Flee, Falloff};
///
/// // 1. Traditional 3D Point Flee with distance attenuation
/// let point_flee = Flee::new(Vec3::new(10.0, 0.0, 0.0))
///     .with_falloff(Falloff::Linear { threshold: 15.0 });
///
/// // 2. Line / Laser Evasion (repels perpendicularly away from a Y-axis hazard line)
/// let line_flee = Flee::rail(Vec3::Y, Vec3::ZERO)
///     .with_falloff(Falloff::Linear { threshold: 5.0 });
/// ```
#[derive(Component, Debug, Reflect)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Flee {
    /// Global spatial position the agent is attempting to flee from.
    pub target: Vec3,

    /// Spatial offset applied relative to the target's position.
    ///
    /// Defines a localized threat position shifted from `target` (e.g. fleeing from 
    /// a point in front of an enemy or a projected hazard location).
    ///
    /// - `Vec3::ZERO`: Flees directly from `target`.
    /// - Non-zero `Vec3`: Flees from a relative point shifted from `target`.
    pub offset: Vec3,

    /// Unconstrained free-motion axis for the repulsive threat zone.
    ///
    /// - `Vec3::ZERO`: Standard 3D Point Flee (repels spherically away from the target point).
    /// - Normalized unit vector (e.g. `Vec3::Y`): Infinite Line / Rail Flee (repels 
    ///   perpendicularly away from an infinite 3D line passing through the target).
    pub axis_direction: Vec3, 

    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Flee {
    /// Creates a new `Flee` behavior pointing away from `target` with no distance falloff ([`Falloff::None`]).
    pub const fn new(target: Vec3) -> Self {
        Self {
            target,
            offset : Vec3::ZERO,
            axis_direction : Vec3::ZERO,
            falloff: Falloff::None,
        }
    }

    /// **Rail / Line Seek.** Repels the agent perpendicularly away 
    ///   from an infinite 3D line passing through `target + offset` (useful for evading lasers or linear hazards).
    pub fn rail(axis_direction: Vec3, position: Vec3) -> Self {
        Self::new(position).with_axis_direction(axis_direction)
    }


    /// Sets the falloff behavior 
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }

    /// Sets the relative spatial offset from the target.
    pub const fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Sets the unconstrained direction axis 
    pub fn with_axis_direction(mut self, axis_direction: Vec3) -> Self {
        self.axis_direction = axis_direction.normalize_or_zero();
        self
    }
}

impl Flee {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<seek::BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {
                let v = Seek::v_direction(&agent);
             
                let distance = agent.transform.translation().distance(v);

                if distance <= f32::EPSILON {
                    agent.context.clear_interest::<Self>();                    
                    agent.context.clear_danger::<Self>();
                } else {
                    let agent_position = agent.transform.translation();
                    let steering_dir = (v - agent_position).normalize_or_zero();

                    let factor = agent.behaviour.falloff.outwards_factor(distance);
                    let steering_dir = steering_dir * factor;
                    
                    agent.context.set_danger::<Self>(steering_dir);
            
                    // This prevents snapping to default direction (perpendicular to target) when using danger-only.
                    // By telling it to go directly opposite
                    agent.context.set_interest::<Self>(-steering_dir);
                }
            });
    }
}

impl seek::BehaviourData for Flee {
    fn target(&self) -> Vec3 {
        self.target
    }
    
    fn offset(&self) -> Vec3 {
        self.offset     
    }
    
    fn axis_direction(&self) -> Vec3 {
        self.axis_direction
    }
    
}
