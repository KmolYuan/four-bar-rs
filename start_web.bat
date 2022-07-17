@echo off
setlocal
set REPO=%~dp0

cargo install simple-http-server
simple-http-server --cors --ip 127.0.0.1 --index -- "%REPO%docs"
endlocal
