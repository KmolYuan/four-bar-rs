import * as wasm from "./pkg/four_bar_ui.js";

// Module level references
const a = document.createElement("a");
const input = document.createElement("input");
input.type = "file";
const reader = new FileReader();
input.addEventListener("change", function () {
    if (this.files.length !== 0)
        reader.readAsText(this.files[0]);
});

// Utility functions
window.save_file = function (s, file_name) {
    a.download = file_name;
    a.href = window.URL.createObjectURL(new Blob([s], {type: "application/octet-stream"}));
    a.click();
};
window.load_file = function (format, done) {
    reader.onload = e => done(e.target.result);
    input.accept = format;
    input.click();
};
window.get_host = function () {
    return location.href;
};
window.get_username = function () {
    const parts = ("; " + document.cookie).split("; username=");
    return parts.length === 2 ? parts.pop().split(";").shift() : "";
};
window.login = function (account, body, done) {
    fetch(location.href + "login/" + account, {
        method: "POST",
        body: body,
        headers: {"content-type": "application/json"},
        mode: "cors",
    }).then(res => done(res.ok));
};
window.logout = function (done) {
    fetch(location.href + "logout", {
        method: "POST",
        mode: "cors",
    }).then(res => done(res.ok));
};

// Startup WebAssembly
wasm.default().then(() => wasm.start("main_canvas"));
