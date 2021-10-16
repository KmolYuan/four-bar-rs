//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage mechanism.
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
pub use crate::{four_bar::*, mechanism::*, point::*};

mod four_bar;
mod mechanism;
pub mod plot;
mod point;
pub mod synthesis;
#[cfg(test)]
mod tests;
