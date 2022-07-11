const WASM_BINARY_MAGIC: u32 = 0x6d736100; // \0asm
const WASM_VERSION: u32 = 1;

mod var;

use var::{Var, VarImpl};

enum Section {
    Type,
    Import,
    Function,
    Table,
    Memory,
    Data,
    Global,
    Start,
    Element,
    Code,
}
