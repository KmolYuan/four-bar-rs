@echo off
setlocal
set REPO=%~dp0
cd "%REPO%" || exit

@REM This is required to enable the web_sys clipboard API which egui_web uses
@REM https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Clipboard.html
@REM https://rustwasm.github.io/docs/wasm-bindgen/web-sys/unstable-apis.html
set RUSTFLAGS=--cfg=web_sys_unstable_apis

echo Generating JS bindings for wasm...
wasm-pack build --release --out-dir ../docs/pkg -t web --no-typescript "%REPO%four-bar-ui"
del "%REPO%docs\pkg\.gitignore"
del "%REPO%docs\pkg\package.json"
copy "%REPO%four-bar-ui\assets\favicon.png" "%REPO%docs"
copy "%REPO%LICENSE" "%REPO%docs"

echo Make the archive...
for %%f in ("%REPO%target\debug", "%REPO%target\release") do if exist %%f (
    powershell -Command "Compress-Archive -Path %REPO%docs\* -DestinationPath %%f\four-bar-wasm-unknown.zip -Force"
)

echo Finished
endlocal
