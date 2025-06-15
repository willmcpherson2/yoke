use super::*;
use grammar::*;
use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(grammar, "/mir/grammar.rs");

pub fn parse(input: &str) -> Result<Program, ParseError<usize, Token, &str>> {
    ProgramParser::new().parse(input)
}
