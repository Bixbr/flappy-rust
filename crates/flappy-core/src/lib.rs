#![cfg_attr(not(test), no_std)]

pub mod atlas;
pub mod game;
pub mod rng;
mod sin_table;

pub use atlas::{AtlasMap, SpriteRect};
pub use game::*;
pub use rng::Rng;
