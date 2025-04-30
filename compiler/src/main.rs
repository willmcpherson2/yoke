mod lir;

use lir::*;

fn main() {
    let prog = Prog {
        globals: vec![
            Global {
                name: "F",
                symbol: 1,
                arity: 1,
            },
            Global {
                name: "A",
                symbol: 2,
                arity: 0,
            },
        ],
        funs: vec![],
        main: Block(vec![
            Op::LoadGlobal(LoadGlobal {
                name: "F",
                global: "F",
            }),
            Op::LoadGlobal(LoadGlobal {
                name: "A",
                global: "A",
            }),
            Op::NewApp(NewApp {
                name: "result",
                var: "F",
                args: vec!["A"],
            }),
            Op::FreeTerm(FreeTerm { var: "result" }),
            Op::ReturnSymbol(ReturnSymbol { var: "result" }),
        ]),
    };

    dbg!(prog.compile(Config {
        target: Target::Binary,
    }));
}
