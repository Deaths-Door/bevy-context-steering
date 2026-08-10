use super::*;

pub trait SteeringCommands {
    /// Sets the weight of a specific steering behavior for this entity.
    fn set_steering_weight<B: 'static>(&mut self, weight: f32) -> &mut Self;

    /// Modifies the weight of a steering behavior using a closure.
    fn update_steering_weight<B: 'static>(
        &mut self,
        modify: impl FnOnce(f32) -> f32 + Send + 'static,
    ) -> &mut Self;
}

impl SteeringCommands for EntityCommands<'_> {
    fn set_steering_weight<B: 'static>(&mut self, weight: f32) -> &mut Self {
        self.update_steering_weight::<B>(move |_| weight)
    }

    fn update_steering_weight<B: 'static>(
        &mut self,
        modify: impl FnOnce(f32) -> f32 + Send + 'static,
    ) -> &mut Self {
        self.queue(move |mut entity: EntityWorldMut| {
            let entity_id = entity.id();

            let Some(mut context) = entity.get_mut::<SteeringContext>() else {
                warn!("`SteeringContext` missing on Entity {entity_id:?}");
                return;
            };

            let Some(entry) = context.get_mut::<B>() else {
                warn!(
                    "Steering behavior `{}` not found in `SteeringContext` on Entity {entity_id:?}. Ensure it was inserted prior to weight updates.",
                    std::any::type_name::<B>()
                );
                return;
            };

            let current_weight = entry.weight();
            entry.set_weight(modify(current_weight));
        });
        self
    }
}
