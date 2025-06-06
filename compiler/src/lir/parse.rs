use super::{Arity, Block, Case, Global, Index, Name, Op, Program, Symbol};
use chumsky::{extra::Err, prelude::*};
use text::{ascii::ident, whitespace};

pub fn parse(input: &str) -> Result<Program, Vec<Rich<char>>> {
    program().parse(input).into_result()
}

fn program<'a>() -> impl Parser<'a, &'a str, Program, Err<Rich<'a, char>>> {
    global()
        .separated_by(whitespace())
        .collect::<Vec<Global>>()
        .padded()
        .map(globals_to_program)
}

fn globals_to_program(globals: Vec<Global>) -> Program {
    let mut program = Program {
        globals: vec![],
        main: Block(vec![]),
    };

    for global in globals {
        match global {
            Global::Fun { name, block, .. } if name == "main" => program.main = block,
            _ => program.globals.push(global),
        }
    }

    program
}

fn global<'a>() -> impl Parser<'a, &'a str, Global, Err<Rich<'a, char>>> {
    choice((
        just("const")
            .then_ignore(whitespace())
            .then(name())
            .then_ignore(whitespace())
            .then(arity())
            .then_ignore(whitespace())
            .then(symbol())
            .map(|(((_, name), arity), symbol)| Global::Const {
                name,
                arity,
                symbol,
            }),
        just("fun")
            .then_ignore(whitespace())
            .then(name())
            .then_ignore(whitespace())
            .then(arity())
            .then_ignore(whitespace())
            .then(block())
            .map(|(((_, name), arity), block)| Global::Fun { name, arity, block }),
    ))
    .labelled("global")
}

fn block<'a>() -> impl Parser<'a, &'a str, Block, Err<Rich<'a, char>>> {
    recursive(|block| {
        let case = symbol()
            .then_ignore(whitespace())
            .then(block)
            .map(|(symbol, block)| Case { symbol, block });

        let cases = case
            .separated_by(whitespace())
            .collect::<Vec<Case>>()
            .padded()
            .delimited_by(just('{'), just('}'))
            .labelled("cases");

        let op = choice((
            just("load_global")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .map(|((_, name), global)| Op::LoadGlobal { name, global }),
            just("load_arg")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(index())
                .map(|(((_, name), var), index)| Op::LoadArg { name, var, index }),
            just("new_app")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(args())
                .map(|(((_, name), var), args)| Op::NewApp { name, var, args }),
            just("new_partial")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(args())
                .map(|(((_, name), var), args)| Op::NewPartial { name, var, args }),
            just("apply_partial")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(args())
                .map(|(((_, name), var), args)| Op::ApplyPartial { name, var, args }),
            just("copy")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .map(|((_, name), var)| Op::Copy { name, var }),
            just("eval")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(name())
                .map(|((_, name), var)| Op::Eval { name, var }),
            just("free_args")
                .then_ignore(whitespace())
                .then(name())
                .map(|(_, var)| Op::FreeArgs { var }),
            just("free_term")
                .then_ignore(whitespace())
                .then(name())
                .map(|(_, var)| Op::FreeTerm { var }),
            just("return_symbol")
                .then_ignore(whitespace())
                .then(name())
                .map(|(_, var)| Op::ReturnSymbol { var }),
            just("return")
                .then_ignore(whitespace())
                .then(name())
                .map(|(_, var)| Op::Return { var }),
            just("switch")
                .then_ignore(whitespace())
                .then(name())
                .then_ignore(whitespace())
                .then(cases)
                .map(|((_, var), cases)| Op::Switch { var, cases }),
            just("todo").map(|_| Op::Todo),
        ))
        .labelled("instruction");

        op.separated_by(whitespace())
            .collect::<Vec<Op>>()
            .padded()
            .delimited_by(just('{'), just('}'))
            .map(Block)
            .labelled("block")
            .boxed()
    })
}

fn args<'a>() -> impl Parser<'a, &'a str, Vec<Name>, Err<Rich<'a, char>>> {
    name()
        .separated_by(whitespace())
        .collect::<Vec<Name>>()
        .padded()
        .delimited_by(just('{'), just('}'))
        .labelled("args")
}

fn name<'a>() -> impl Parser<'a, &'a str, Name, Err<Rich<'a, char>>> {
    ident().map(|s: &str| s.to_string()).labelled("name")
}

fn arity<'a>() -> impl Parser<'a, &'a str, Arity, Err<Rich<'a, char>>> {
    text::int(10)
        .try_map(|s: &str, span| s.parse::<Arity>().map_err(|e| Rich::custom(span, e)))
        .labelled("arity (16 bit integer)")
}

fn symbol<'a>() -> impl Parser<'a, &'a str, Symbol, Err<Rich<'a, char>>> {
    text::int(10)
        .try_map(|s: &str, span| s.parse::<Symbol>().map_err(|e| Rich::custom(span, e)))
        .labelled("symbol (32 bit integer)")
}

fn index<'a>() -> impl Parser<'a, &'a str, Index, Err<Rich<'a, char>>> {
    text::int(10)
        .try_map(|s: &str, span| s.parse::<Index>().map_err(|e| Rich::custom(span, e)))
        .labelled("index (64 bit integer)")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_program() {
        let result = program().parse("fun main 0 {}");
        assert_eq!(
            result.unwrap(),
            Program {
                globals: vec![],
                main: Block(vec![]),
            }
        );
    }

    #[test]
    fn test_global() {
        let result = global().parse("const True 0 1");
        assert_eq!(
            result.unwrap(),
            Global::Const {
                name: "True".to_string(),
                arity: 0,
                symbol: 1
            }
        );

        let result = global().parse("fun f 1 { return x }");
        assert_eq!(
            result.unwrap(),
            Global::Fun {
                name: "f".to_string(),
                arity: 1,
                block: Block(vec![Op::Return {
                    var: "x".to_string()
                }]),
            }
        );
    }

    #[test]
    fn test_block() {
        let result = block().parse("{ return x }");
        assert_eq!(
            result.unwrap(),
            Block(vec![Op::Return {
                var: "x".to_string()
            }])
        );

        let result = block().parse(
            r"{
                switch x {
                    0 {
                        return a
                    }
                    1 {
                        return b
                    }
                }
            }",
        );
        assert_eq!(
            result.unwrap(),
            Block(vec![Op::Switch {
                var: "x".to_string(),
                cases: vec![
                    Case {
                        symbol: 0,
                        block: Block(vec![Op::Return {
                            var: "a".to_string()
                        },])
                    },
                    Case {
                        symbol: 1,
                        block: Block(vec![Op::Return {
                            var: "b".to_string()
                        },])
                    },
                ],
            }])
        );
    }
}
