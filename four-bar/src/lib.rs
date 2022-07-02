//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage mechanism.
//!
//! <https://en.wikipedia.org/wiki/Four-bar_linkage>
//!
//! ```
//! use four_bar::{FourBar, Mechanism};
//! use std::f64::consts::TAU;
//!
//! // A four-bar mechanism example
//! let m = Mechanism::new(&FourBar::example());
//! // Get the trajectory of the coupler point
//! let path = m.curve(0., TAU, 360);
//! ```
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![warn(missing_docs)]
pub use crate::{four_bar::*, mechanism::*, point::*};
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub use metaheuristics_nature as mh;

pub mod curve;
mod four_bar;
mod mechanism;
#[cfg(feature = "plot")]
pub mod plot;
mod point;
pub mod repr;
pub mod syn;
pub mod tests;
