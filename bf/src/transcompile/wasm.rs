// use wasmtime::{Engine, Linker, Module, Store};
// use wasmtime_wasi::WasiCtxBuilder;

pub fn bf_wat(bf: &str) -> String {
    fn code_to_wat(code: &str, memory_base_address: i32) -> String {
        let mut loop_count = 0;
        let mut loop_stack = Vec::new();

        let mut wat =
            format!("(local $pointer i32) i32.const {memory_base_address} local.set $pointer ");
        for code in code.chars() {
            match code {
                '+' => {
                    wat += "local.get $pointer\n";
                    wat += "i32.const 1\n";
                    wat += "call $add";
                    wat += "\n";
                }
                '-' => {
                    wat += "local.get $pointer\n";
                    wat += "i32.const 1\n";
                    wat += "call $sub\n";
                    wat += "\n";
                }
                '>' => {
                    wat += "local.get $pointer\n";
                    wat += "i32.const 1\n";
                    wat += "i32.add\n";
                    wat += "local.set $pointer\n";
                    wat += "\n";
                }
                '<' => {
                    wat += "local.get $pointer\n";
                    wat += "i32.const 1\n";
                    wat += "i32.sub\n";
                    wat += "local.set $pointer\n";
                    wat += "\n";
                }
                '.' => {
                    wat += "local.get $pointer\n";
                    wat += "i32.load8_u\n";
                    wat += "call $print_char\n";
                    wat += "\n";
                }
                '[' => {
                    let loop_name = format!("loop_{loop_count}");
                    loop_stack.push(loop_name.clone());
                    let exit_name = format!("exit_{loop_count}");
                    loop_count += 1;

                    wat += &format!(
                        "
(block ${exit_name}
    (loop ${loop_name}
        i32.const 0
        local.get $pointer
        i32.load8_u

        (br_if ${exit_name} (i32.eq))
"
                    );
                }
                ']' => {
                    let loop_name = loop_stack.pop().unwrap();
                    wat += &format!("(br ${loop_name})");
                    wat += "))";
                }
                _ => continue,
            }
        }
        wat
    }

    let memory_base_address = 40;

    let mut wat = String::from(
        r#"
(module
    (import "wasi_unstable" "fd_write" (func $fd_write (param i32 i32 i32 i32) (result i32)))
    (memory (export "memory") 1 1000)
    (func $print_char (param $char i32)
        i32.const 0
        local.get $char
        i32.store8

        ;; Creating a new io vector within linear memory
        (i32.store (i32.const 4) (i32.const 0))  ;; iov.iov_base - This is a pointer to the start of the 'hello world\n' string
        (i32.store (i32.const 8) (i32.const 1))  ;; iov.iov_len - The length of the 'hello world\n' string

        (call $fd_write
            (i32.const 1) ;; file_descriptor - 1 for stdout
            (i32.const 4) ;; *iovs - The pointer to the iov array, which is stored at memory location 0
            (i32.const 1) ;; iovs_len - We're printing 1 string stored in an iov - so one.
            (i32.const 12) ;; nwritten - A place in memory to store the number of bytes written
        )
        drop ;; Discard the number of bytes written from the top of the stack
    )
    (func $add (param $pointer i32) (param $value i32)
        local.get $pointer
        local.get $pointer
        i32.load8_u
        local.get $value
        i32.add

        i32.store8
    )
    (func $sub (param $pointer i32) (param $value i32)
        local.get $pointer
        local.get $pointer
        i32.load8_u
        local.get $value
        i32.sub

        i32.store8
    )
"#,
    );

    let main_func = format!(
        "(func $main (export \"_start\") {})",
        code_to_wat(bf, memory_base_address)
    );

    wat += &main_func;
    wat += ")";

    wat
}

// pub fn run_bf(bf: &str) -> anyhow::Result<()> {
//     let wat = bf_wat(bf);

//     let engine = Engine::default();

//     let mut linker = Linker::new(&engine);
//     wasmtime_wasi::add_to_linker(&mut linker, |a| a)?;

//     let wasi = WasiCtxBuilder::new().inherit_stdout().build();
//     let mut store = Store::new(&engine, wasi);

//     let module = Module::new(&engine, wat)?;

//     linker
//         .module(&mut store, "", &module)?
//         .get_default(&mut store, "")?
//         .typed::<(), (), _>(&store)?
//         .call(&mut store, ())?;
//     Ok(())
// }
