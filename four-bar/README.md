# Four🍀bar

[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Four-bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

```rust
use four_bar::FourBar;

// Get the trajectory of the coupler point
let path = FourBar::example().curve(360);
```

The synthesis function let you synthesize a four-bar mechanism by fitting target curve.

```rust
use four_bar::{mh, syn};

let func = syn::FbSyn::from_curve(curve, syn::Mode::Closed).res(res);
let s = mh::Solver::build(mh::Rga::default(), func)
    .task(|ctx| ctx.gen == gen)
    .pop_num(pop)
    .solve()
    .unwrap();
```
