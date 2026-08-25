mod enabled;
mod options;
mod systems;

pub use enabled::*;
pub use options::*;

use super::*;

/// A Bevy plugin that provides visual debugging and diagnostics for steering behaviors.
pub struct SteeringDebugPlugin;

impl Plugin for SteeringDebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPostUpdate, systems::debug_steering_context);
    }
}
