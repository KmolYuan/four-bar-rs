//! Curve (trajectory) operation functions.
//!
//! The input curve can be both a owned type `Vec<[f64; 2]>` or a pointer type
//! `&[[f64; 2]]` since the generic are copy-on-write (COW) compatible.
pub use efd::{closed_lin, closed_rev, curve_diff};
use std::borrow::Cow;

/// Check if a curve's first and end points are very close.
pub fn is_closed<A: PartialEq>(curve: &[A]) -> bool {
    match (curve.first(), curve.last()) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// Input a curve, split out the longest finite parts to a continuous curve.
pub fn get_valid_part<'a, C>(curve: C) -> Vec<[f64; 2]>
where
    Cow<'a, [[f64; 2]]>: From<C>,
{
    let curve = Cow::from(curve);
    let mut iter = curve.iter();
    let mut last = Vec::new();
    while iter.len() > 0 {
        let v = iter
            .by_ref()
            .take_while(|[x, y]| x.is_finite() && y.is_finite())
            .copied()
            .collect::<Vec<_>>();
        if v.len() > last.len() {
            last = v;
        }
    }
    last
}

/// Remove the last point.
///
/// This function allows empty curve.
pub fn remove_last<'a, A, C>(curve: C) -> Vec<A>
where
    A: Clone + 'a,
    Cow<'a, [A]>: From<C>,
{
    let mut curve = Cow::from(curve).into_owned();
    curve.pop();
    curve
}

/// Close the open curve with a symmetry part.
///
/// Panic with empty curve.
pub fn closed_symmetric<'a, C>(curve: C) -> Vec<[f64; 2]>
where
    Cow<'a, [[f64; 2]]>: From<C>,
{
    let mut curve = Cow::from(curve).into_owned();
    let first = &curve[0];
    let end = &curve[curve.len() - 1];
    let curve2 = curve
        .iter()
        .rev()
        .take(curve.len() - 1)
        .skip(1)
        .map(|p| {
            let dx = end[0] - first[0];
            let dy = end[1] - first[1];
            let a = (dx * dx - dy * dy) / (dx * dx + dy * dy);
            let b = 2. * dx * dy / (dx * dx + dy * dy);
            let p_first_dx = p[0] - first[0];
            let p_first_dy = p[1] - first[1];
            let x = a * p_first_dx + b * p_first_dy + first[0];
            let y = b * p_first_dx - a * p_first_dy + first[1];
            [x, y]
        })
        .collect::<Vec<_>>();
    curve.extend(curve2);
    closed_lin(curve)
}

/// Close the open curve with an anti-symmetry part.
///
/// Panic with empty curve.
pub fn closed_anti_symmetric<'a, C>(curve: C) -> Vec<[f64; 2]>
where
    Cow<'a, [[f64; 2]]>: From<C>,
{
    let mut curve = Cow::from(curve).into_owned();
    let [ox, oy] = {
        let first = &curve[0];
        let end = &curve[curve.len() - 1];
        [(first[0] + end[0]) * 0.5, (first[1] + end[1]) * 0.5]
    };
    let curve2 = curve
        .iter()
        .take(curve.len() - 1)
        .map(|[x, y]| {
            use std::f64::consts::PI;
            let x = ox + PI.cos() * (x - ox) - PI.sin() * (y - oy);
            let y = oy + PI.sin() * (x - ox) + PI.cos() * (y - oy);
            [x, y]
        })
        .collect::<Vec<_>>();
    curve.extend(curve2);
    closed_lin(curve)
}

/// Close the open curve with anti-symmetric extension function.
///
/// Panic with empty curve.
pub fn closed_anti_sym_ext<'a, C>(curve: C) -> Vec<[f64; 2]>
where
    Cow<'a, [[f64; 2]]>: From<C>,
{
    let curve = Cow::from(curve).into_owned();
    let n = curve.len() - 1;
    let [x0, y0] = curve[0];
    let [xn, yn] = curve[n];
    let xd = xn - x0;
    let yd = yn - y0;
    let n = n as f64;
    let mut v1 = curve
        .iter()
        .enumerate()
        .map(|(i, &[x, y])| {
            let i = i as f64 / n;
            [x - x0 - xd * i, y - y0 - yd * i]
        })
        .collect::<Vec<_>>();
    let v2 = v1
        .iter()
        .take(curve.len() - 1)
        .skip(1)
        .map(|[x, y]| [-x, -y])
        .rev()
        .collect::<Vec<_>>();
    v1.extend(v2);
    v1
}

/// Geometry error between two closed curves.
///
/// The curves must have the same length.
pub fn geo_err(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    debug_assert!(!target.is_empty());
    debug_assert_eq!(target.len(), curve.len());
    // Find the starting point (correlation)
    let [tx, ty] = &target[0];
    let (i, _) = curve
        .iter()
        .map(|[x, y]| (tx - x).hypot(ty - y))
        .enumerate()
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();
    // Error
    target
        .iter()
        .zip(curve.iter().cycle().skip(i))
        .map(|([x1, y1], [x2, y2])| (x1 - x2).hypot(y1 - y2))
        .sum()
}

/// Count the crunodes of the curve.
pub fn crunode(curve: &[[f64; 2]]) -> usize {
    let mut order = (0..curve.len()).collect::<Vec<_>>();
    order.sort_unstable_by(|a, b| curve[*a][0].total_cmp(&curve[*b][0]));
    // Active list
    let mut act = vec![0; curve.len()];
    // Sweep line
    let mut count = 0;
    for i in 0..curve.len() {
        for prev_next in [false, true] {
            if order[i] == 0 && !prev_next {
                continue;
            }
            let prev_next = if prev_next {
                order[i] + 1
            } else {
                order[i] - 1
            };
            if prev_next >= curve.len() {
                continue;
            }
            // Overlap checking
            // Line 1:
            // order[i], prev_next
            // Line 2:
            // j - 1, j
            for j in 0..curve.len() {
                // Skip inactive point (no line)
                if j == 0 || act[j - 1] == 0 || act[j] == 0 {
                    continue;
                }
                // Check overlap
                // Ignore the connection
                let mut set = std::collections::HashSet::new();
                set.insert(order[i]);
                set.insert(prev_next);
                set.insert(j);
                set.insert(j - 1);
                if set.len() == 4
                    && intersect(
                        [curve[order[i]][0], curve[order[i]][1]],
                        [curve[prev_next][0], curve[prev_next][1]],
                        [curve[j][0], curve[j][1]],
                        [curve[j - 1][0], curve[j - 1][1]],
                    )
                {
                    count += 1;
                }
            }
            // Decrease counter if passed
            if curve[prev_next][0] >= curve[order[i]][0] {
                act[prev_next] += 1;
                act[order[i]] += 1;
            } else {
                act[prev_next] -= 1;
            }
        }
    }
    count / 3
}

fn orientation(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> u8 {
    let slp = (q[1] - p[1]) * (r[0] - q[0]) - (q[0] - p[0]) * (r[1] - q[1]);
    if slp == 0. {
        0
    } else if slp < 0. {
        1
    } else {
        2
    }
}

/// Return true if two lines have intersection.
///
/// ```
/// use four_bar::curve::intersect;
///
/// assert_eq!(false, intersect([1., 1.], [10., 1.], [1., 2.], [10., 2.]));
/// assert_eq!(true, intersect([10., 0.], [0., 10.], [0., 0.], [10., 10.]));
/// assert_eq!(false, intersect([-5., -5.], [0., 0.], [1., 1.], [10., 10.]));
/// ```
pub fn intersect(p1: [f64; 2], q1: [f64; 2], p2: [f64; 2], q2: [f64; 2]) -> bool {
    fn online(p: [f64; 2], q: [f64; 2], r: [f64; 2]) -> bool {
        q[0] <= p[0].max(r[0])
            && q[0] >= p[0].min(r[0])
            && q[1] <= p[1].max(r[1])
            && q[1] >= p[1].min(r[1])
    }
    let o1 = orientation(p1, q1, p2);
    let o2 = orientation(p1, q1, q2);
    let o3 = orientation(p2, q2, p1);
    let o4 = orientation(p2, q2, q1);
    o1 != o2 && o3 != o4
        || o1 == 0 && online(p1, p2, q1)
        || o2 == 0 && online(p1, q2, q1)
        || o3 == 0 && online(p2, p1, q2)
        || o4 == 0 && online(p2, q1, q2)
}
