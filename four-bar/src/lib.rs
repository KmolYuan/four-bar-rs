//! Four🍀bar is a simulator, a synthesizing tool for four-bar linkage mechanism.
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![warn(missing_docs)]
pub use crate::anti_sym_ext::*;
pub use crate::four_bar::*;
pub use crate::mechanism::*;
pub use crate::point::*;

mod anti_sym_ext;
mod four_bar;
mod mechanism;
pub mod plot;
mod point;
pub mod synthesis;
#[cfg(test)]
mod tests;
