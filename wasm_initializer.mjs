export default function () {
    const status = document.createElement('div');
    status.style = "position: absolute; top: 10px; left: 10px; color: white; font-family: monospace; z-index: 1000;";
    
    return {
        onStart: () => {
            document.body.appendChild(status);
            status.innerText = "Initializing...";
        },
        onProgress: ({ current, total }) => {
            // current is bytes downloaded
            const mb = (current / 1024 / 1024).toFixed(2);
            status.innerText = `Loaded: ${mb} MB`;
        },
        onComplete: () => {
            status.remove();
        }
    };
}