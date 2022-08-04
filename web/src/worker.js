onmessage = async function ({ data: data }) {
    const imports = {
        wasi_unstable: {
            fd_read: function (fd, iov, len, _) {
                // めっちゃ怪しい実装
                if (stdin.length == stdin_count) {
                    memory[0] = 255
                } else {
                    memory[0] = stdin.charCodeAt(stdin_count)
                    stdin_count += 1
                }
            },
            fd_write: function (fd, iov, len, _) {
                const c = memory[0];
                postMessage({ out: c })
            }
        }
    };


    const module = data.module
    const stdin = data.stdin

    let stdin_count = 0

    const instance = await WebAssembly.instantiate(module, imports);
    console.log(stdin)

    const memory = new Uint32Array(instance.exports.memory.buffer)

    const start = performance.now();
    instance.exports._start()
    const end = performance.now();
    postMessage({ exec_time: end - start })
}