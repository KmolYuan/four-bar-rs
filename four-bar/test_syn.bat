@echo off
setlocal
set REPO=%~dp0
cd "%REPO%" || exit
cargo test --release --lib tests::test_syn --all-features -- --nocapture
endlocal
