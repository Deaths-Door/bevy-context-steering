use super::*;

/// Steering behavior that pushes the agent away from the center of mass of its assigned clusters.
#[derive(Component, Debug, Reflect, Default, Deref, DerefMut)]
#[component(on_add = on_add_into_steering_context::<Self>, on_remove = on_remove_from_steering_context::<Self>)]
#[require(SteeringContext)]
pub struct Scatter {
    /// Distance-based attenuation profile controlling force magnitude drop-off.
    pub falloff: Falloff,
}

impl Scatter {
    /// Creates a new `Self`  no distance falloff ([`Falloff::None`]).
    pub const fn new() -> Self {
        Self {
            falloff: Falloff::None,
        }
    }

    /// Sets the falloff behavior
    pub const fn with_falloff(mut self, falloff: Falloff) -> Self {
        self.falloff = falloff;
        self
    }
}

impl Scatter {
    pub(crate) fn steering_behaviour_update(
        agent_query: ActiveAgentsQuery<cohere::BehaviourQueryData<Self>>,
    ) {
        cohere::update_internal::<Self>(agent_query)
    }
}

impl cohere::Behaviour for Scatter {
    fn update_context(&self, context: &mut SteeringContext, target_direction: Vec3) {
        let distance = target_direction.length();
        let factor = self.falloff.outwards_factor(distance);
        let target_direction = target_direction * factor;
        context.set_danger::<Self>(target_direction);
    }
}
