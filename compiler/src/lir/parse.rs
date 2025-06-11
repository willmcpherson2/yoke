use super::*;
use ariadne::{Label, Report, ReportKind, Source};
use grammar::*;
use lalrpop_util::{lalrpop_mod, ParseError};

lalrpop_mod!(grammar, "/lir/grammar.rs");

pub fn parse(input: &str) -> Result<Program, ParseError<usize, Token, &str>> {
    ProgramParser::new().parse(input)
}

pub fn print_parse_errors(file: &str, input: &str, error: ParseError<usize, Token, &str>) {
    match error {
        ParseError::InvalidToken { location } => {
            build_report(file, input, "unrecognized token", location, location)
        }
        ParseError::UnrecognizedEof { location, expected } => build_report(
            file,
            input,
            &format!("unexpected eof. expected {}", expected.join(" or ")),
            location,
            location,
        ),
        ParseError::UnrecognizedToken {
            token: (start, token, end),
            expected,
        } => build_report(
            file,
            input,
            &format!(
                "unrecognized token {}. expected {}",
                token,
                expected.join(" or ")
            ),
            start,
            end,
        ),
        ParseError::ExtraToken {
            token: (start, token, end),
        } => build_report(
            file,
            input,
            &format!("unexpected extra token: {}", token),
            start,
            end,
        ),
        ParseError::User { error } => build_report(file, input, error, 0, 0),
    }
}

fn build_report(file: &str, input: &str, reason: &str, start: usize, end: usize) {
    let range = start..end;
    let label = Label::new((file, range.clone())).with_message(reason);
    Report::build(ReportKind::Error, (file, range))
        .with_message("Parse error")
        .with_label(label)
        .finish()
        .eprint((file, Source::from(input)))
        .unwrap();
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
