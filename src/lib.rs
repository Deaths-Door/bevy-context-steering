mod agent;
mod behaviours;
mod clusters;
mod commands;
mod context;
mod debug;
mod prediction;
mod systems;
mod utils;

pub use avian3d;
pub use bevy;
pub use many_relationships as bevy_many_relationships;

pub use agent::*;
pub use behaviours::*;
pub use clusters::*;
pub use commands::*;
pub use context::*;
pub use debug::*;
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

        app.add_observer(on_add_cluster_member);
        app.add_observer(on_remove_cluster_member);

        
        let behaviour_update = (
            Seek::steering_behaviour_update,
            Flee::steering_behaviour_update,
            Pursuit::steering_behaviour_update,
            Evade::steering_behaviour_update,
            /*behaviours::pursuit::update,
            behaviours::evade::update,
            behaviours::cohere::update,
            behaviours::scatter::update,
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
