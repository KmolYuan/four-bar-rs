use ndarray::{concatenate, s, Array2, AsArray, Axis, Ix2};

/// Anti-Symmetric Extension
pub fn anti_sym_ext<'a, V>(polygon: V) -> Array2<f64>
where
    V: AsArray<'a, f64, Ix2>,
{
    let mut polygon = polygon.into().to_owned();
    let n = polygon.nrows() - 1;
    let [x0, y0] = [polygon[[0, 0]], polygon[[0, 1]]];
    let [xn, yn] = [polygon[[n, 0]], polygon[[n, 1]]];
    for i in 0..polygon.nrows() {
        polygon[[i, 0]] -= x0 + (xn - x0) * i as f64 / n as f64;
        polygon[[i, 1]] -= y0 + (yn - y0) * i as f64 / n as f64;
    }
    #[allow(clippy::reversed_empty_ranges)]
    let inverse = &polygon.slice(s![1..-1;-1, ..]) * -1.;
    concatenate!(Axis(0), polygon, inverse)
}
