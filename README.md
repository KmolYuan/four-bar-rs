# Four🍀bar

[![dependency status](https://deps.rs/repo/github/KmolYuan/four-bar-rs/status.svg)](https://deps.rs/repo/github/KmolYuan/four-bar-rs)

Four-bar is a simulator, a synthesizing tool for four-bar linkage mechanism.

Online demo: <https://kmolyuan.github.io/four-bar-rs/>

Native icon: <https://icons8.com>

## Native Execution

In Linux, you need to install some library:

```bash
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev libxkbcommon-dev
```

Then run with cargo directly:

```bash
cargo run
```

## Serving WASM in Localhost

```bash
./setup_web.sh
./build_web.sh
./start_server.sh
```

## Library

The library `four-bar` can also install by Cargo.
