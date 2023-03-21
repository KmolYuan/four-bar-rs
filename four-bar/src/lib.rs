//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage
//! mechanism.
//!
//! <https://en.wikipedia.org/wiki/Four-bar_linkage>
//!
//! ```
//! use four_bar::FourBar;
//!
//! // Get the trajectory of the coupler point
//! let path = FourBar::example().curve(360);
//! ```
#![cfg_attr(doc_cfg, feature(doc_auto_cfg))]
#![warn(missing_docs)]
#![warn(clippy::semicolon_if_nothing_returned)]

pub use crate::{fb2d::*, fb3d::*};
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub extern crate metaheuristics_nature as mh;

#[cfg(feature = "codebook")]
pub mod cb;
#[cfg(feature = "csv")]
pub mod csv;
pub mod curve;
mod fb2d;
mod fb3d;
#[cfg(feature = "plot")]
pub mod plot2d;
#[cfg(feature = "plot")]
pub mod plot3d;
pub mod syn2d;
pub mod syn3d;
#[cfg(test)]
mod tests;
