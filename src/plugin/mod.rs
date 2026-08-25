mod sets;
mod systems;

pub use sets::*;

use super::*;
use bevy_many_relationships::ManyRelationshipsPlugin;
use systems::*;

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ManyRelationshipsPlugin );

        app.init_resource::<ClusterMap>();
        app.add_observer(on_insert_cluster);
        app.add_observer(on_discard_cluster);

        app.configure_sets(FixedPreUpdate, SteeringSpatialSet);
        app.configure_sets(FixedUpdate, SteeringBehaviorSet);
        app.configure_sets(FixedPostUpdate, SteeringPhysicsSet);

        app.add_systems(
            FixedPreUpdate,
            update_cluster_data.in_set(SteeringSpatialSet),
        );

        let behaviour_update = (
            Seek::steering_behaviour_update,
            Flee::steering_behaviour_update,
            Pursuit::steering_behaviour_update,
            Evade::steering_behaviour_update,
            Brake::steering_behaviour_update,
            Throttle::steering_behaviour_update,
            Cohere::steering_behaviour_update,
            Scatter::steering_behaviour_update,
            /*
            behaviours::align::update,
            behaviours::seperate::update,
            behaviours::standoff::position::update, */
        );

        app.add_systems(FixedUpdate, behaviour_update.in_set(SteeringBehaviorSet));
        app.add_systems(
            FixedPostUpdate,
            update_resultant_field.in_set(SteeringPhysicsSet),
        );
    }
}
