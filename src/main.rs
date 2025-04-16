mod lir;
mod rts;

use lir::{compile, Prog};

fn main() {
    let prog = Prog {
        globals: vec![],
        funs: vec![],
        main: vec![],
    };

    compile(prog);
}
