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

pub use crate::{defect::*, plane::*, sphere::*};
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub extern crate metaheuristics_nature as mh;
extern crate nalgebra as na;

#[cfg(feature = "codebook")]
pub mod cb;
pub mod curve;
mod defect;
pub mod planar_syn;
mod plane;
#[cfg(feature = "plot")]
pub mod plot2d;
#[cfg(feature = "plot")]
pub mod plot3d;
mod sphere;
pub mod spherical_syn;
#[cfg(test)]
mod tests;
