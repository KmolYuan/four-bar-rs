// Utility functions
window.loading_finished = () => document.getElementById("loading-text").remove();
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
