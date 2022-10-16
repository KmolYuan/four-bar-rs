// Utility functions
window.loading_finished = () => document.getElementById("loading-text").remove();
window.open_file = (ext, done, multiple) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ext;
    input.multiple = multiple;
    input.onchange = () => [...input.files].forEach(f => f.text().then(t => done(f.name, t)));
    input.click();
};
window.open_bfile = (ext, done) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ext;
    input.multiple = true;
    input.onchange = () => [...input.files].forEach(f => f.arrayBuffer().then(a => done(f.name, new Uint8Array(a))));
    input.click();
};
window.save_file = (s, path) => {
    const a = document.createElement("a");
    a.download = path;
    a.href = URL.createObjectURL(new Blob([s], {type: "application/octet-stream"}));
    a.click();
};
