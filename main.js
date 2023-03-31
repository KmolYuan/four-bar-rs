// Utility functions
window.loading_finished = () => document.getElementById("loading-text").remove();
window.open_file = (ext, done, is_multiple, is_bin) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ext;
    input.multiple = is_multiple;
    input.onchange = () => [...input.files].forEach(f => {
        if (is_bin)
            f.arrayBuffer().then(a => done(new Uint8Array(a)));
        else
            f.text().then(t => done(f.name, t));
    });
    input.click();
};
window.save_file = (s, path) => {
    const a = document.createElement("a");
    a.download = path;
    a.href = URL.createObjectURL(new Blob([s], { type: "application/octet-stream" }));
    a.click();
};
