use super::*;

/// Steering behavior that accelerates an agent away from a target position in global space.
///
/// When active, `Flee` calculates a repelling vector pointed directly away from [`Self::target`].
/// The intensity of this force decays over distance according to the configured [`Falloff`].
///
/// # Examples
///
/// ```rust
/// // Create a flee behavior with linear falloff within 15 units
/// let flee = Flee::new(Vec3::new(10.0, 0.0, 0.0))
///     .with_falloff(Falloff::Linear { threshold: 15.0 });
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
    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Flee {
    /// Creates a new `Flee` behavior pointing away from `target` with no distance falloff ([`Falloff::None`]).
    pub const fn new(target: Vec3) -> Self {
        Self {
            target,
            falloff: Falloff::None,
        }
    }

    /// Sets the falloff behavior using builder syntax.
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }
}

impl Flee {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<seek::BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {
                let desired_direction = Seek::desired_direction(&agent);

                let distance = agent.transform.translation().distance(agent.behaviour.target);
                let factor = agent.behaviour.falloff.outwards_factor(distance);
                let desired_direction = desired_direction * factor;
           
                agent.context.set_danger::<Self>(desired_direction);
            
                // This prevents snapping to default direction (perpendicular to target) when using danger-only.
                // By telling it to go directly opposite
                agent.context.set_interest::<Self>(-desired_direction);
            });
    }
}

impl seek::BehaviourData for Flee {
    fn target(&self) -> Vec3 {
        self.target
    }
}
