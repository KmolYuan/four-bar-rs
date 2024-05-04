# Four🍀bar

[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Four-bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

```rust
use four_bar::FourBar;

// Get the trajectory of the coupler point
let path = FourBar::example().curve(360);
```

## Features
+ **Simulation**: Calculate the trajectory of the coupler point (or any point) on the planar/spherical four-bar linkage.
    + `FourBar` struct defines a planar four-bar linkage.
    + `MFourBar` struct defines a planar four-bar linkage for motion generation (rigid body guidance) synthesis.
    + `SFourBar` struct defines a spherical four-bar linkage.
    + `*NormFourBar` structs are the normalized versions of the above linkages without the translation, rotation, and scaling.
+ **Serialization**: Serialize and deserialize four-bar linkages and their trajectories to/from a file via `serde`. (`serde` and `csv` feature)
+ **Plotting**: Visualize the four-bar linkage and the trajectory of the coupler point. (`plot` feature)
+ **Synthesis**: Find the dimensions of the four-bar linkage that will guide the coupler point through a desired trajectory. You can use the `rayon` feature to speed up the synthesis process. Also, the `clap` feature provides a CLI interface for the synthesis tool.
+ **Atlas**: A collection of four-bar linkages with known trajectories. It can be used to find the best match for a given trajectory, which is similar to the synthesis process but without the optimization part. (`atlas` feature)
+ **GUI**: A graphical user interface `four-bar-ui` is available at the [repo](https://github.com/KmolYuan/four-bar-rs) with a web demo.

## Citations
The synthesis technique is based on the paper by us:
+ Chang, Y., Chang, JL., Lee, JJ. (2024). Atlas-Based Path Synthesis of Planar Four-Bar Linkages Using Elliptical Fourier Descriptors. In: Okada, M. (eds) Advances in Mechanism and Machine Science. IFToMM WC 2023. Mechanisms and Machine Science, vol 149. Springer, Cham. <https://doi.org/10.1007/978-3-031-45709-8_20>
+ Chang, Y., Chang, JL. & Lee, JJ. Path Synthesis of Planar Four-bar Linkages for Closed and Open Curves Using Elliptical Fourier Descriptors. J Mech Sci Technol (2024). <http://doi.org/10.1007/s12206-024-0436-y>
