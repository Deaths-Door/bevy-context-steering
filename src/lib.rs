mod agent;
mod behaviours;
mod clusters;
mod commands;
mod context;
mod prediction;
mod systems;
mod utils;

#[cfg(feature = "debug")]
pub mod debug;

pub use avian3d;
pub use bevy;
pub use bevy_many_relationships as many_relationships;

pub use agent::*;
pub use behaviours::*;
pub use clusters::*;
pub use commands::*;
pub use context::*;
pub use prediction::*;

pub(crate) use utils::*;

use bevy_many_relationships::ManyRelationshipsPlugin;

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use systems::*;

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ManyRelationshipsPlugin);

        app.init_resource::<ClusterMap>();
        app.add_observer(on_insert_cluster);
        app.add_observer(on_discard_cluster);

        // TODO: use system stes or smth
        app.add_systems(FixedPreUpdate, update_cluster_data);

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

        app.add_systems(FixedUpdate, behaviour_update);
        app.add_systems(
            FixedPostUpdate,
            (update_resultant_field, apply_forces).chain(),
        );
    }
}
