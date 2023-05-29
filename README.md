# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)
[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Web demo: <https://kmolyuan.github.io/four-bar-rs/>

![](https://raw.githubusercontent.com/KmolYuan/four-bar-rs/master/img/screenshot.png)

Four🍀bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

+ CLI/GUI program is `four-bar-ui` crate, but the executable is called `four-bar`.
+ Kernel library `four-bar` is on <https://crates.io/crates/four-bar>.

GUI powered by <https://github.com/emilk/egui>.

## Releases

In Linux, `libxcb` and `libgtk-3` are required, and most desktops are already provided.

In Windows, since the builds are set in Visual Studio, the [support of Visual C++](https://docs.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist?view=msvc-160) is required.

Download: <https://github.com/KmolYuan/four-bar-rs/releases/latest>

Native GUI is the default startup behaviour, use `--help` option to see more commands.

```bash
# Equivalent to double-clicking the executable
four-bar
# Equivalent to opening by the app or file dragging
four-bar FILE_PATH1 FILE_PATH2
```

## Native Build

In Linux, you need to install some library for GUI.

This is an example for `apt`, please see <https://github.com/emilk/egui#demo>.

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev
```

Then run with cargo directly:

```bash
cargo run
# CLI Mode
cargo run -- --help
```

## WebAssembly Build

Powered by `trunk` (<https://github.com/thedodd/trunk>), a Rust CLI tool for creating bin-type WASM projects.

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

The application is deployed in the `dist` directory (entire files).

## NetBSD

On NetBSD a pre-compiled binary is available from the official repositories, to install it simply run:

```bash
pkgin install four-bar-rs
```
