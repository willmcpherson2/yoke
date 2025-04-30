mod lir;
mod rts;

use lir::*;

macro_rules! test {
    ($prog:expr, $expected:expr) => {
        let Output::Jit(result) = $prog.compile(Config {
            target: Target::Jit,
        }) else {
            panic!()
        };
        assert_eq!(result, $expected);
    };
}

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
            Op::ReturnSymbol(ReturnSymbol { var: "result" }),
        ]),
    };

    prog.compile(Config {
        target: Target::Binary,
    });
}
