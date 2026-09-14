#![doc = include_str!("../README.md")]

mod clusters;
mod commands;
mod components;
mod context;
mod neighbours;
mod obstacles;
mod plugin;
mod prediction;
mod utils;

pub mod behaviours;
pub mod motion;

#[cfg(feature = "debug")]
pub mod debug;

pub use avian3d;
pub use bevy;
pub use bevy_many_relationships as many_relationships;

pub use clusters::*;
pub use commands::*;
pub use components::*;
pub use context::*;
pub use neighbours::*;
pub use obstacles::*;
pub use plugin::*;
pub use prediction::*;

pub(crate) use utils::*;

use avian3d::prelude::*;
use bevy::{ecs::query::QueryData, prelude::*};

pub mod prelude {
    pub use super::*;
    pub use behaviours::*;
    pub use motion::*;

    #[cfg(feature = "debug")]
    pub use debug::*;
}
