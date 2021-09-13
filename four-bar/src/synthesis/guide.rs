use ndarray::{arr2, concatenate, Array2, Axis};

pub(crate) fn guide(c: &mut Array2<f64>, v: &[f64]) {
    for i in (0..v.len()).step_by(2) {
        let end = arr2(&[[
            c[[c.nrows() - 1, 0]] + v[i] * v[i + 1].cos(),
            c[[c.nrows() - 1, 1]] + v[i] * v[i + 1].sin(),
        ]]);
        *c = concatenate!(Axis(0), *c, end);
    }
}
