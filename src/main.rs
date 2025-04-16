mod lir;
mod rts;

use lir::*;

fn main() {
    let prog = Prog {
        globals: vec![Global {
            name: "True",
            symbol: 1,
            arity: 0,
        }],
        funs: vec![],
        main: vec![
            Op::LoadGlobal(LoadGlobal {
                name: "True",
                global: "True",
            }),
            Op::ReturnSymbol(ReturnSymbol { var: "True" }),
        ],
    };

    compile(prog);
}
