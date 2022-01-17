import * as wasm from "./pkg/four_bar_ui.js";

// Utility functions
window.open_file = (ext, done, multiple) => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = multiple;
    input.onchange = () => Array.from(input.files).forEach(file => {
        const reader = new FileReader();
        reader.onload = () => done(file.name, reader.result);
        reader.readAsText(file);
    });
    input.accept = ext;
    input.click();
};
window.save_file = (s, file_name) => {
    const a = document.createElement("a");
    a.download = file_name;
    a.href = URL.createObjectURL(new Blob([s], {type: "application/octet-stream"}));
    a.click();
};
window.get_host = () => location.href;
window.get_username = () => ("; " + document.cookie).split("; username=").pop().split(";").shift();
window.login = (account, body, done) =>
    fetch(location.href + "login/" + account, {
        method: "POST",
        body: body,
        headers: {"content-type": "application/json"},
        mode: "cors",
    }).then(res => done(res.ok));
window.logout = done =>
    fetch(location.href + "logout", {
        method: "POST",
        mode: "cors",
    }).then(res => done(res.ok));

// Startup WebAssembly
wasm.default().then(() => wasm.start("main_canvas"));
