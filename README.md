# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)

Four-bar is a simulator and a synthesizing tool for four-bar linkage mechanism. The GUI program is called `four-bar-ui`.

Online demo: <https://kmolyuan.github.io/four-bar-rs/>

Native icon: <https://icons8.com>

Powered by <https://github.com/emilk/egui>.

## Native Execution

In Linux, you need to install some library:

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev libgtk-3-dev
```

Then run with cargo directly:

```bash
cargo run
```

## Serving WASM in Localhost

Clone this repository first, then run those scripts:

```bash
./setup_web.sh
./build_web.sh
./start_server.sh
```

## Library

The library `four-bar` is also on crates-io, exclude the ui parts.
