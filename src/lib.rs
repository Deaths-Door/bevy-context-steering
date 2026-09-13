mod commands;
mod components;
mod context;
mod obstacles;
mod plugin;
mod prediction;
mod utils;
mod clusters;
mod neighbours;

pub mod behaviours;
pub mod motion;

#[cfg(feature = "debug")]
pub mod debug;

pub use avian3d;
pub use bevy;
pub use bevy_many_relationships as many_relationships;

pub use commands::*;
pub use components::*;
pub use context::*;
pub use plugin::*;
pub use prediction::*;
pub use clusters::*;
pub use neighbours::*;

pub(crate) use utils::*;

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};
