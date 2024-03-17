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

pub use crate::mech::{FourBar, MFourBar, MNormFourBar, NormFourBar, SFourBar, SNormFourBar};
#[doc(no_inline)]
pub use efd;
#[doc(no_inline)]
pub extern crate metaheuristics_nature as mh;
#[cfg(feature = "atlas")]
pub use ndarray;

#[cfg(feature = "atlas")]
pub mod atlas;
#[cfg(feature = "csv")]
pub mod csv;
pub mod mech;
#[cfg(feature = "plot")]
pub mod plot;
pub mod syn;
