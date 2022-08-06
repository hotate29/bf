import { bf_to_wasm } from '@hotate29/bf';

const start_button = document.querySelector('button[id="start"]');
start_button.addEventListener('click', run);

// まだ
const abort_button = document.querySelector('button[id="abort"]');
abort_button.disabled = true;
abort_button.addEventListener('click', abort);

const p = document.querySelector('p');

async function run() {
    start_button.disabled = true;

    const worker = new Worker(new URL('./worker.js', import.meta.url));
    window.worker = worker

    const bf_element = document.querySelector('textarea[id="bf"]');
    const bf = bf_element.value;

    const stdin_element = document.querySelector('textarea[id="stdin"]');
    const stdin = stdin_element.value;

    const stdout_pre = document.querySelector('pre');

    const start_transpile = performance.now();

    let wasm;
    try { wasm = bf_to_wasm(bf); } catch (e) {
        alert(e);
        return
    }

    const module = await WebAssembly.compile(wasm);

    const end_transpile = performance.now();
    const transpile_time = end_transpile - start_transpile;

    worker.postMessage({ module: module, stdin: stdin })
    abort_button.disabled = false;

    p.textContent = "Running..."

    stdout_pre.textContent = ""

    worker.onmessage = function (e) {
        const msg = e.data

        if (typeof msg.out === 'number') {
            stdout_pre.textContent += String.fromCharCode(msg.out)
        }
        else if (typeof msg.exec_time === 'number') {
            const exec_time = msg.exec_time
            p.textContent = `Transpile: ${transpile_time}ms Execution: ${exec_time}ms`;
            start_button.disabled = false;
            abort_button.disabled = true;
        }
    }
}

async function abort() {
    window.worker.terminate();
    abort_button.disabled = true;
    start_button.disabled = false;
    p.textContent = "Aborted"
}
