mod agent;
mod behaviours;
mod context;
mod debug;
mod systems;

pub use agent::*;
pub use behaviours::*;
pub use context::*;

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
use systems::*;

pub struct SteeringPlugin;

impl Plugin for SteeringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPostUpdate,
            (update_resultant_field, apply_forces).chain(),
        );
    }
}
