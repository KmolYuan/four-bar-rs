//! Curve (trajectory) operation.

#[inline(always)]
fn boxed_iter<'a, I>(iter: I) -> Box<dyn Iterator<Item = &'a [f64; 2]> + 'a>
where
    I: Iterator<Item = &'a [f64; 2]> + 'a,
{
    Box::new(iter) as _
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

/// Anti-symmetric extension function.
pub fn anti_sym_ext(curve: &[[f64; 2]]) -> Vec<[f64; 2]> {
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

/// Close the open curve directly.
pub fn close_loop(mut curve: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    curve.push(curve[0]);
    curve
}

/// Return false if curve contains any NaN coordinate.
pub fn is_valid_curve(curve: &[[f64; 2]]) -> bool {
    !curve.iter().any(|[x, y]| !x.is_finite() || !y.is_finite())
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
        .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
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
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    (basic_err + err) / target.len() as f64
}
