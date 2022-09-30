# Four🍀bar

[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Four-bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

```rust
use four_bar::FourBar;
use std::f64::consts::TAU;

// Get the trajectory of the coupler point
let path = FourBar::example().curve(0., TAU, 360);
```

The synthesis function let you synthesize a four-bar mechanism by fitting target curve.

```rust
use four_bar::{mh, syn};

let s = mh::Solver::build(mh::De::default())
    .task(|ctx| ctx.gen == gen)
    .pop_num(pop_num)
    .record(|ctx| ctx.best_f)
    .solve(syn::PathSyn::from_curve(target, None, n, syn::Mode::Close))
    .unwrap();
```
