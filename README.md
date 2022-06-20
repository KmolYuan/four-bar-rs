# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)
[![documentation](https://docs.rs/four-bar/badge.svg)](https://docs.rs/four-bar)

Four🍀bar is a simulator and a synthesizing tool for four-bar linkage mechanism.

+ GUI program is called `four-bar-ui`, but the runtime is called `four-bar`.
+ Kernel library `four-bar` is on crates-io.

Online demo: <https://kmolyuan.github.io/four-bar-rs/>

Native icon: <https://icons8.com>

Powered by <https://github.com/emilk/egui>.

## Native Executable

In Linux, `libxcb` and `libgtk-3` are required. (most desktops are already provided)

In Windows, [the support of Visual C++](https://docs.microsoft.com/zh-TW/cpp/windows/latest-supported-vc-redist?view=msvc-160) is required.

Download: <https://github.com/KmolYuan/four-bar-rs/releases/latest>

Native GUI is the default startup behaviour, use `--help` option to see more commands.

```bash
# Equivalent to double-clicking the executable
four-bar
four-bar ui
```

## Native Build

In Linux, you need to install some library for GUI:

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev
```

Then run with cargo directly:

```bash
cargo run -- --help
```

## Serving WebAssembly Client in Localhost

### From Releases

The default port is 8080.

```bash
four-bar serve --port PORT
```

A security connection (https) can be built with encryption program like [Stunnel](https://www.stunnel.org/).

### From Repository

Run those scripts from the repository:

```bash
./setup_web.sh
./build_web.sh
cargo run -- serve --port PORT
```

The client-side application is deployed in the `docs` directory (entire files).

### Cloud Computing Service

*This function is still developing.*

Cloud Computing Services are optional for native clients and are designed to improve performance by sending the synthesis task to another machine. But WebAssembly clients are not support to run the task in a web browser, and for security reasons, they should use the cloud computing services hosted by their domain.
