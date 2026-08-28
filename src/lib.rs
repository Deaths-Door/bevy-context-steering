mod agent;
mod behaviours;
mod clusters;
mod commands;
mod context;
mod plugin;
mod prediction;
mod utils;

pub mod motion;

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
pub use plugin::*;
pub use prediction::*;

pub(crate) use utils::*;

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
