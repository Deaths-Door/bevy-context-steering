use super::*;


pub trait SteeringCommands {
    /// Sets the weight of a specific behavior.
    fn apply_weight<T: 'static>(&mut self, weight: f32) -> &mut Self;

    /// Modifies the weight of a behavior using a closure.
    fn update_weight<T: 'static>(
        &mut self,
        modify: impl FnOnce(f32) -> f32 + Send + Sync + 'static,
    ) -> &mut Self;
}


impl SteeringCommands for EntityCommands<'_> {
    fn apply_weight<T: 'static>(&mut self, weight: f32) -> &mut Self {
        self.update_weight::<T>(move |_| weight)
    }

    fn update_weight<T: 'static>(
        &mut self,
        modify: impl FnOnce(f32) -> f32 + Send + 'static,
    ) -> &mut Self {
        self.queue(move |mut entity: EntityWorldMut| {
            let Some(mut context) = entity.get_mut::<SteeringContext>() else {
                warn!("SteeringContext missing on entity {:?}", entity.id());
                return;
            };

            let Some(entry) = context.get_mut::<T>() else {
                let type_name = std::any::type_name::<T>();
                warn!("Steering behavior {} not found in SteeringContext on Entity {:?}. Ensure it was inserted before modifying weights.", type_name, entity.id());
                return;
            };

            let weight = entry.weight();
            entry.set_weight(modify(weight));
        });
        self
    }
}
