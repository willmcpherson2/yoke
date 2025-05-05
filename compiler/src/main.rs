mod lir;

use lir::*;

fn main() {
    let prog = Prog {
        globals: vec![Global {
            name: "True",
            symbol: 1,
            arity: 0,
        }],
        funs: vec![Fun {
            name: "id",
            arg_name: "self",
            symbol: 2,
            arity: 1,
            block: Block(vec![
                Op::LoadArg(LoadArg {
                    name: "x",
                    var: "self",
                    index: 0,
                }),
                Op::FreeArgs(FreeArgs { var: "self" }),
                Op::Eval(Eval {
                    name: "x",
                    var: "x",
                }),
                Op::Return(Return { var: "x" }),
            ]),
        }],
        main: Block(vec![
            Op::LoadGlobal(LoadGlobal {
                name: "id",
                global: "id",
            }),
            Op::LoadGlobal(LoadGlobal {
                name: "True",
                global: "True",
            }),
            Op::NewApp(NewApp {
                name: "result",
                var: "id",
                args: vec!["True"],
            }),
            Op::Eval(Eval {
                name: "result",
                var: "result",
            }),
            Op::ReturnSymbol(ReturnSymbol { var: "result" }),
        ]),
    };

    dbg!(prog.compile(Config {
        target: Target::Binary,
        opt_level: OptimizationLevel::Aggressive
    }));
}
