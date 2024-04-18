# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)
[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Web demo: <https://kmolyuan.github.io/four-bar-rs/>

![](https://raw.githubusercontent.com/KmolYuan/four-bar-rs/master/img/screenshot.png)

Four🍀bar is a simulator and a synthesis tool for four-bar linkage mechanism.

+ CLI/GUI program is `four-bar-ui` crate, but the executable is called `four-bar`.
+ Kernel library `four-bar` is on <https://crates.io/crates/four-bar>.
+ Cite this work [here](#citation).

GUI powered by <https://github.com/emilk/egui>.

## Release

Download: <https://github.com/KmolYuan/four-bar-rs/releases/latest>

Native GUI is the default startup behavior, use `--help` option to see more commands.

```bash
# Equivalent to double-clicking the executable
four-bar
# Equivalent to opening by the app or file dragging
four-bar FILE_PATH1 FILE_PATH2
```

Some platforms require additional dependencies as listed below. If your platform have no pre-compiled binary, you need to build from source.

### Windows

The executable requires support for Visual C++. You can refer to the [Visual C++ support documentation](https://docs.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-160) for installation instructions.

### Linux

`libxcb` and `libgtk-3` are required to run the application on Linux, and most desktop environments come with these packages pre-installed.

### NetBSD

On NetBSD a pre-compiled binary is available from the official repositories, to install it simply run:

```bash
pkgin install four-bar-rs
```

## Build from Source

In Linux, you need to install some library for GUI.

This is an example for `apt`, please see <https://github.com/emilk/egui#demo>.

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev
```

Then run with cargo directly:

```bash
cargo build --release
cargo run --release -- --help
```

### WebAssembly

Powered by [`trunk`](<https://github.com/thedodd/trunk>), a Rust CLI tool for creating bin-type WASM projects.

Run those scripts from the repository:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
cd four-bar-ui
# Serve in localhost
trunk serve
# Compile WASM
trunk build --release
```

The application package will be deployed in the `dist` directory.

## File Format

Four🍀bar uses a custom file format to store the linkage data. The file is a plain text file of [RON](https://github.com/ron-rs/ron) syntax.

### Synthesis Target Format

The target coupler curve can be input from:

+ CSV `.csv` file
+ RON `.ron` file as a linkage to generate a coupler curve

The file name should be `name.mode.ext` pattern, where

+ `name` is the name of the case, can be any string
+ `mode` is the mode of the target curve, can be `closed`, `open`, and `partial`
+ `ext` is the file extension, can be `csv` or `ron`

For example, `example.closed.csv` is a target closed curve in CSV format.

### Figure Configuration

The figure configuration is a RON file with the extension `.fig.ron`. Four🍀bar GUI can import figure configurations to generate linkage plots, and the `syn` command will redraw the linkage result without cleaning if the figure configuration exists.

## Synthesis

Four🍀bar can synthesize a four-bar linkage from the target curve. The synthesis algorithm is based on the paper by us.

```bash
four-bar syn TARGET_FILE_PATH --[OPTION FLAGS]
# Print help message
four-bar syn --help
```

The synthesis result will be saved in a project directory with the same name as the target file.

Execute the `./paper-ex.sh` to reproduce the paper results under this repository.

## Citations

+ Chang, Y., Chang, JL., Lee, JJ. (2024). Atlas-Based Path Synthesis of Planar Four-Bar Linkages Using Elliptical Fourier Descriptors. In: Okada, M. (eds) Advances in Mechanism and Machine Science. IFToMM WC 2023. Mechanisms and Machine Science, vol 149. Springer, Cham. <https://doi.org/10.1007/978-3-031-45709-8_20>
