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
window.save_file = (s, path) => {
    const a = document.createElement("a");
    a.download = path;
    a.href = URL.createObjectURL(new Blob([s], {type: "application/octet-stream"}));
    a.click();
};

// Startup WebAssembly
wasm.default().then(() => {
    wasm.start("main_canvas");
    document.getElementById("loading-text").remove();
}).catch(err => {
    document.getElementById("loading-text").innerHTML = `
    <p>An error occurred during loading:</p>
    <p style="font-family: Courier New, Ubuntu Mono, monospace">${err}</p>
    <p style="font-size: 14px">Make sure you use a modern browser with WebGL and WASM enabled.</p>`;
});
