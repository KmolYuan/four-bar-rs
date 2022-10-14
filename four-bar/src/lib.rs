//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage
//! mechanism.
//!
//! <https://en.wikipedia.org/wiki/Four-bar_linkage>
//!
//! ```
//! use four_bar::FourBar;
//! use std::f64::consts::TAU;
//!
//! // Get the trajectory of the coupler point
//! let path = FourBar::example().curve(0., TAU, 360);
//! ```
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub use crate::four_bar::*;
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub extern crate metaheuristics_nature as mh;

#[cfg(feature = "codebook")]
pub mod cb;
pub mod curve;
mod four_bar;
#[cfg(feature = "plot")]
pub mod plot;
pub mod syn;
#[cfg(test)]
mod tests;
