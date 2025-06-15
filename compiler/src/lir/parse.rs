use super::*;
use grammar::*;
use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(grammar, "/lir/grammar.rs");

pub fn parse(input: &str) -> Result<Program, ParseError<usize, Token, &str>> {
    ProgramParser::new().parse(input)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_program() {
        assert_eq!(
            parse("main = 0 {}").unwrap(),
            HashMap::from([(
                "main".to_string(),
                Global::Fun {
                    arity: 0,
                    block: vec![],
                }
            )])
        );

        assert_eq!(
            parse("True = 0 1").unwrap(),
            HashMap::from([((
                "True".to_string(),
                Global::Ctor {
                    arity: 0,
                    symbol: 1
                }
            ))])
        );

        assert_eq!(
            parse("f = 1 { return x }").unwrap(),
            HashMap::from([((
                "f".to_string(),
                Global::Fun {
                    arity: 1,
                    block: vec![Op::Return {
                        var: "x".to_string()
                    }],
                }
            ))])
        );
    }
}
