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

pub use crate::fb::*;
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub extern crate metaheuristics_nature as mh;
#[cfg(feature = "plot")]
pub use self::plot::{plot2d, plot3d};

#[cfg(feature = "codebook")]
pub mod cb;
#[cfg(feature = "csv")]
pub mod csv;
mod fb;
#[cfg(feature = "plot")]
pub mod plot;
pub mod syn;
