use bevy::math::FloatPow;

use super::*;

/// Steering behavior that accelerates an agent toward a target position in global space.
///
/// When active, `Seek` calculates an attractive vector pointed directly toward [`Self::target`].
/// The intensity of this force decays over distance according to the configured [`Falloff`].
///
/// # Examples
///
/// ```rust
/// use bevy::prelude::*;
/// use bevy_context_steering::{Seek,Falloff};
/// // Create a seek behavior with smoothstep falloff within 20 units
/// let seek = Seek::new(Vec3::new(10.0, 0.0, 0.0))
///     .with_falloff(Falloff::SmoothStep { threshold: 20.0 });
/// ```
#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component, Debug)]
#[component(
    on_add = on_add_into_steering_context::<Self>, 
    on_remove = on_remove_from_steering_context::<Self>
)]
#[require(SteeringContext)]
pub struct Seek {
    /// Global spatial position the agent is attempting to seek.
    pub target: Vec3,
    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Seek {
  /// Creates a new `Seek` behavior pointing toward `target` with no distance falloff ([`Falloff::None`]).
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

impl Seek {
    pub(crate) fn steering_behaviour_update(mut query: ActiveAgentsQuery<BehaviourQueryData<Self>>) {
        query
            .par_iter_mut()
            .for_each(|mut agent| {
                let desired_direction = Self::desired_direction(&agent);

                let distance = agent.transform.translation().distance(agent.behaviour.target);

                // TODO: remvoe this since this doesnt help
                if distance.squared() <= f32::EPSILON {
                    agent.context.clear_interest::<Self>();
                } else {
                    let factor = agent.behaviour.falloff.inwards_factor(distance);

                    let desired_direction = desired_direction * factor;
                    agent.context.set_interest::<Self>(desired_direction);
                }
            });
    }

    pub(super) fn desired_direction<T:BehaviourData>(agent: &BehaviourQueryDataItem<'_, '_, T>) -> Vec3 {
        let position = agent.transform.translation();
        let target = agent.behaviour.target();
        let desired_direction = (target - position).normalize_or_zero();
        desired_direction
    }
}

#[derive(QueryData)]
#[query_data(mutable)]
pub(crate) struct BehaviourQueryData<T: Component> {
    pub(crate) behaviour: &'static T,
    pub(crate) transform: &'static GlobalTransform,
    pub(crate) context: &'static mut SteeringContext,
}

pub(crate) trait BehaviourData : Component {
    fn target(&self) -> Vec3;
}

impl BehaviourData for Seek {
    fn target(&self) -> Vec3 {
        self.target
    }
}

