use ariadne::{Label, Report, ReportKind, Source};
use lalrpop_util::ParseError;
use std::fmt::Display;

pub fn print_parse_errors<T: Display>(file: &str, input: &str, error: ParseError<usize, T, &str>) {
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
