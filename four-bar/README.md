# Four🍀bar

Four-bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

```rust
use four_bar::{FourBar, Mechanism};

// A four-bar mechanism example
let m = Mechanism::new(FourBar::example());
// Get the trajectory of the coupler point
let path = m.curve(0., 360);
```

The synthesis function let you synthesize a four-bar mechanism by fitting target curve.

```rust
use four_bar::synthesis::synthesis;

let s = synthesis(&curve, gen, pop, |_| true);
let result = s.result();
```
