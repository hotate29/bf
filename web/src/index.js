import { init, WASI } from '@wasmer/wasi';
import { bf_to_wasm } from '@hotate29/bf';

const button = document.querySelector('button');
button.addEventListener('click', run);

async function run() {
    const startWasiTask = async (wasm, stdin) => {
        await init();
        let wasi = new WASI({
            env: {},
            args: []
        });

        let module = await WebAssembly.compile(wasm);
        await wasi.instantiate(module, {});

        wasi.setStdinString(stdin)
        wasi.start()

        const stdout = wasi.getStdoutString();
        return stdout
    }

    const bf_element = document.querySelector('textarea[id="bf"]');
    const bf = bf_element.value;

    const stdin_element = document.querySelector('textarea[id="stdin"]');
    const stdin = stdin_element.value;

    const span = document.querySelector('textarea[id="stdout"]');

    const start_transpile = performance.now();
    const wasm = bf_to_wasm(bf);
    const end_transpile = performance.now();
    const transpile_time = end_transpile - start_transpile;

    span.innerHTML = ""

    const start_exec = performance.now();
    let stdout = await startWasiTask(wasm, stdin)
    const end_exec = performance.now();

    const exec_time = end_exec - start_exec;

    const p = document.querySelector('p');
    p.textContent = `Transpile: ${transpile_time}ms Execution: ${exec_time}ms`;

    span.textContent = stdout
}

