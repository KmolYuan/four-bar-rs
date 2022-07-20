# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)
[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Four🍀bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

+ GUI program is called `four-bar-ui`, but the runtime is called `four-bar`.
+ Kernel library `four-bar` is on <https://crates.io>.

Online demo: <https://kmolyuan.github.io/four-bar-rs/>

Native icon: <https://icons8.com>

Powered by <https://github.com/emilk/egui>.

## Native Executable

In Linux, `libxcb` and `libgtk-3` are required. (most desktops are already provided)

In Windows, since the builds are set in Visual Studio, the [support of Visual C++](https://docs.microsoft.com/zh-TW/cpp/windows/latest-supported-vc-redist?view=msvc-160) is required.

Download: <https://github.com/KmolYuan/four-bar-rs/releases/latest>

Native GUI is the default startup behaviour, use `--help` option to see more commands.

```bash
# Equivalent to double-clicking the executable
four-bar
# Equivalent to opening by the app or file dragging
four-bar FILE_PATH1 FILE_PATH2
```

## Native Build

In Linux, you need to install some library for GUI:

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

Run those scripts from the repository:

```bash
./setup_web.sh
./build_web.sh
# Start local server at 127.0.0.1:8080
./start_web.sh
```

The application is deployed in the `docs` directory (entire files).

Windows platform also has the `*.bat` files corresponding to the same names.
