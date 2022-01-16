//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage mechanism.
//!
//! <https://en.wikipedia.org/wiki/Four-bar_linkage>
//!
//! ```
//! use std::f64::consts::TAU;
//! use four_bar::{FourBar, Mechanism};
//!
//! // A four-bar mechanism example
//! let m = Mechanism::four_bar(&FourBar::example());
//! // Get the trajectory of the coupler point
//! let path = m.four_bar_loop(0., TAU, 360);
//! ```
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
pub use crate::{four_bar::*, mechanism::*, point::*};

mod four_bar;
mod mechanism;
#[cfg(feature = "plotters")]
pub mod plot;
mod point;
pub mod synthesis;
pub mod tests;
