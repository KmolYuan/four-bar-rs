@echo off
rustup target add wasm32-unknown-unknown
powershell -Command "Invoke-WebRequest https://github.com/rustwasm/wasm-pack/releases/latest/download/wasm-pack-init.exe -OutFile wasm-pack-init.exe"
echo y|.\wasm-pack-init.exe
del wasm-pack-init.exe
