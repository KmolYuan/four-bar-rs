# Four🍀bar

Four-bar is a simulator, a synthesizing tool for four-bar linkage mechanism.

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
