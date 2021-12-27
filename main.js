import * as wasm from "./pkg/four_bar_ui.js";

// Module level references
const a = document.createElement("a");
const input = document.createElement("input");
input.type = "file";
const reader = new FileReader();
input.addEventListener("change", function () {
    if (this.files.length === 0)
        return;
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
window.identity = function () {
    const name = "username=";
    let i = 0;
    while (i < document.cookie.length) {
        const j = i + name.length;
        let end = document.cookie.indexOf(";", j);
        if (end === -1)
            end = document.cookie.length;
        if (document.cookie.substring(i, j) === name)
            return document.cookie.substring(j, end);
        i = end;
    }
    return "";
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
(async () => {
    await wasm.default();
    await wasm.start("main_canvas");
})();
