use crate::Point;
use std::marker::PhantomData;

/// Mechanism position formula.
#[allow(missing_docs)]
#[derive(Clone)]
pub enum Formula {
    Pla(usize, f64, f64, usize),
    Plap(usize, f64, f64, usize, usize),
    Pllp(usize, f64, f64, usize, bool, usize),
    Ppp(usize, usize, usize, usize),
}

impl Formula {
    /// Apply formulas.
    pub fn apply(&self, joints: &mut [impl Point]) {
        match *self {
            Self::Pla(c1, d0, a0, t) => {
                joints[t] = joints[c1].pla(d0, a0);
            }
            Self::Plap(c1, d0, a0, c2, t) => {
                joints[t] = joints[c1].plap(d0, a0, &joints[c2]);
            }
            Self::Pllp(c1, d0, d1, c2, inv, t) => {
                joints[t] = joints[c1].pllp(d0, d1, &joints[c2], inv);
            }
            Self::Ppp(c1, c2, c3, t) => {
                joints[t] = joints[c1].ppp(&joints[c2], &joints[c3]);
            }
        }
    }
}

/// A linkage type.
pub trait Linkage: Sized + Sync {
    /// Memory layout of the joints
    type Joint: Sized + Sync;

    /// Allocate memory for [`Mechanism`] type.
    fn allocate(&self) -> (Self::Joint, Vec<Formula>);
    /// Calculate the formula, and write the answer into provided array.
    fn apply<L, const N: usize>(
        m: &Mechanism<L>,
        angle: f64,
        joint: [usize; N],
        ans: &mut [[f64; 2]; N],
    ) where
        L: Linkage<Joint = Self::Joint>;
}

/// Geometry constraint solver of the linkage mechanisms.
pub struct Mechanism<L: Linkage> {
    /// The joint positions.
    pub joints: L::Joint,
    /// Formula set.
    pub fs: Vec<Formula>,
    _marker: PhantomData<L>,
}

impl<L: Linkage> Mechanism<L> {
    /// Create a mechanism for the linkage.
    pub fn new(linkage: &L) -> Self {
        let (joints, fs) = linkage.allocate();
        Self { joints, fs, _marker: PhantomData }
    }

    /// Calculate the formula, and write the answer into provided array.
    pub fn apply<const N: usize>(&self, angle: f64, joint: [usize; N], ans: &mut [[f64; 2]; N]) {
        L::apply(self, angle, joint, ans)
    }
}

/// Methods for four-bar linkages.
impl<L: Linkage<Joint = [[f64; 2]; 5]>> Mechanism<L> {
    /// A loop trajectory for only coupler point.
    pub fn curve(&self, start: f64, end: f64, n: usize) -> Vec<[f64; 2]> {
        #[cfg(feature = "rayon")]
        use crate::mh::rayon::prelude::*;
        let interval = (end - start) / n as f64;
        #[cfg(feature = "rayon")]
        let iter = (0..n).into_par_iter();
        #[cfg(not(feature = "rayon"))]
        let iter = 0..n;
        iter.map(|i| {
            let a = start + i as f64 * interval;
            let mut ans = [[0., 0.]];
            L::apply(self, a, [4], &mut ans);
            ans[0]
        })
        .collect()
    }

    /// A loop trajectory for all moving pivots. (3)
    ///
    /// + (1) Driver
    /// + (2) Follower
    /// + (3) Coupler
    pub fn curve_all(&self, start: f64, end: f64, n: usize) -> Vec<[[f64; 2]; 3]> {
        let interval = (end - start) / n as f64;
        let mut path = vec![[[0.; 2]; 3]; n];
        for (i, path) in path.iter_mut().enumerate() {
            let a = start + i as f64 * interval;
            let mut ans = [[0., 0.]; 3];
            L::apply(self, a, [2, 3, 4], &mut ans);
            *path = ans;
        }
        path
    }
}
