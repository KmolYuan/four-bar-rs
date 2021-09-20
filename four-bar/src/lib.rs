//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage mechanism.
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
pub use crate::anti_sym_ext::*;
pub use crate::four_bar::*;
pub use crate::mechanism::*;
pub use crate::point::*;

#[cfg(feature = "synthesis")]
mod anti_sym_ext;
mod four_bar;
mod mechanism;
mod point;
#[cfg(feature = "synthesis")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "synthesis")))]
pub mod synthesis;
#[cfg(test)]
mod tests;
