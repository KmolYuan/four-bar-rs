#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub use crate::mech::{FourBar, MFourBar, MNormFourBar, NormFourBar, SFourBar, SNormFourBar};
pub use efd;
pub use mh;
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
