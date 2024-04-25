//! The functions used to plot the 2D motion and synthesis result.
#[doc(no_inline)]
pub use super::*;

/// Drawing option of motion four-bar linkage.
///
/// This is a adaptor type of [`fb::Figure`].
///
/// See also [`Figure::add_pose()`]/[`Figure::push_pose()`] for more details.
pub type Figure<'a, 'b> = fb::Figure<'a, 'b>;

impl<'a, 'b> Figure<'a, 'b> {
    /// Create from an optional motion four-bar linkage.
    pub fn from_mfb(mfb: MFourBar) -> Self {
        Self::new_fb(mfb.into_fb())
    }

    /// Create from an optional motion four-bar linkage reference.
    pub fn from_mfb_ref(mfb: &'b MFourBar) -> Self {
        Self::new_ref(mfb.as_fb())
    }
}
