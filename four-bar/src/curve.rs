//! Curve (trajectory) operation.
//!
//! Curves are typed with `&[[f64; 2]]`, allow containing `NaN`s.

use std::f64::consts::PI;

#[inline(always)]
fn boxed_iter<'a, I>(iter: I) -> Box<dyn Iterator<Item = &'a [f64; 2]> + 'a>
where
    I: Iterator<Item = &'a [f64; 2]> + 'a,
{
    Box::new(iter)
}

/// Check if a curve is closed. (first point and end point are close)
pub fn is_closed(curve: &[[f64; 2]]) -> bool {
    let first = curve[0];
    let end = curve[curve.len() - 1];
    (first[0] - end[0]).abs() < f64::EPSILON && (first[1] - end[1]).abs() < f64::EPSILON
}

/// Input a curve, split out finite parts to a continuous curve. (greedy method)
///
/// The result is close to the first-found finite item,
/// and the part of infinity and NaN will be dropped.
pub fn get_valid_part(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
    let is_invalid = |[x, y]: &[f64; 2]| !x.is_finite() || !y.is_finite();
    let is_valid = |[x, y]: &[f64; 2]| x.is_finite() && y.is_finite();
    let mut iter = curve.iter();
    match iter.position(is_valid) {
        None => Vec::new(),
        Some(t1) => match iter.position(is_invalid) {
            None => curve[t1..].to_vec(),
            Some(t2) => {
                let s1 = curve[t1..t1 + t2].to_vec();
                let mut iter = curve.iter().rev();
                match iter.position(is_valid) {
                    Some(t1) if t1 == 0 => {
                        let t1 = curve.len() - 1 - t1;
                        let t2 = t1 - iter.position(is_invalid).unwrap();
                        [&curve[t2..t1], &s1].concat()
                    }
                    _ => s1,
                }
            }
        },
    }
}

/// Close the open curve with a line.
///
/// Panic with empty curve.
pub fn close_line(mut curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    curve.push(curve[0]);
    curve
}

/// Close the open curve with a symmetry part.
///
/// Panic with empty curve.
pub fn close_symmetric(mut curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
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
    close_line(curve)
}

/// Close the open curve with an anti-symmetry part.
///
/// Panic with empty curve.
pub fn close_anti_symmetric(mut curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    let [ox, oy] = {
        let first = &curve[0];
        let end = &curve[curve.len() - 1];
        [(first[0] + end[0]) * 0.5, (first[1] + end[1]) * 0.5]
    };
    let curve2 = curve
        .iter()
        .take(curve.len() - 1)
        .skip(1)
        .map(|[x, y]| {
            let x = ox + PI.cos() * (x - ox) - PI.sin() * (y - oy);
            let y = oy + PI.sin() * (x - ox) + PI.cos() * (y - oy);
            [x, y]
        })
        .collect::<Vec<_>>();
    curve.extend(curve2);
    close_line(curve)
}

/// Close the open curve with anti-symmetric extension function.
///
/// Panic with empty curve.
pub fn close_anti_sym_ext(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
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
            let i_n = i as f64 / n;
            [x - x0 - xd * i_n, y - y0 - yd * i_n]
        })
        .collect::<Vec<_>>();
    let mut v2 = v1
        .iter()
        .take(curve.len() - 1)
        .skip(1)
        .map(|[x, y]| [-x, -y])
        .rev()
        .collect();
    v1.append(&mut v2);
    v1
}

/// Geometry error between two curves.
///
/// The given curve must longer than target curve.
pub fn geo_err(target: &[[f64; 2]], curve: &[[f64; 2]]) -> f64 {
    let end = curve.len();
    debug_assert!(!target.is_empty());
    debug_assert!(target.len() < end);
    // Find the starting point (correlation)
    let (index, basic_err) = curve
        .iter()
        .enumerate()
        .map(|(i, [x, y])| (i, (target[0][0] - x).hypot(target[0][1] - y)))
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
        .unwrap();
    let iter = boxed_iter(curve.iter().cycle().skip(index).take(end));
    let rev = boxed_iter(curve.iter().rev().cycle().skip(end - index).take(end));
    let err = [iter, rev]
        .into_iter()
        .map(|mut iter| {
            let target = &target[1..];
            let mut geo_err = 0.;
            let mut left = &curve[index];
            for [tx, ty] in target {
                let [x, y] = left;
                let mut last_d = (tx - x).hypot(ty - y);
                for c @ [x, y] in &mut iter {
                    let d = (tx - x).hypot(ty - y);
                    if d < last_d {
                        last_d = d;
                    } else {
                        left = c;
                        break;
                    }
                }
                geo_err += last_d;
            }
            geo_err
        })
        .min_by(|a, b| a.total_cmp(b))
        .unwrap();
    (basic_err + err) / target.len() as f64
}

/// Count the cusp of the curve.
pub fn cusp(curve: &[[f64; 2]], open: bool) -> usize {
    use std::f64::consts::{FRAC_PI_2, TAU};
    let mut iter = curve
        .iter()
        .cycle()
        .take(if open { curve.len() } else { curve.len() + 1 });
    let mut pre = match iter.next() {
        Some(v) => v,
        None => return 0,
    };
    let mut num = 0;
    let mut pre_angle = 0.;
    for c @ [x, y] in &mut iter {
        let [pre_x, pre_y] = pre;
        let angle = (y - pre_y).atan2(x - pre_x);
        let angle_diff = (angle - pre_angle).rem_euclid(TAU);
        if pre_angle != 0. && angle_diff > FRAC_PI_2 && angle_diff < TAU - FRAC_PI_2 {
            num += 1;
        }
        pre_angle = angle;
        pre = c;
    }
    num
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
