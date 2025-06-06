mod lir;

use ariadne::{Label, Report, ReportKind, Source};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let input = match std::fs::read_to_string(input_file) {
        Ok(input) => input,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(2);
        }
    };

    let program = match lir::parse::parse(&input) {
        Ok(program) => program,
        Err(errors) => {
            for error in errors {
                let range = error.span().into_range();
                let reason = error.reason().to_string();
                let label = Label::new((input_file, range.clone())).with_message(reason);
                Report::build(ReportKind::Error, (input_file, range))
                    .with_message("Parse error")
                    .with_label(label)
                    .finish()
                    .eprint((input_file, Source::from(&input)))
                    .unwrap();
            }
            std::process::exit(3);
        }
    };

    let output = program.compile(lir::compile::Config::default());
    match output {
        lir::compile::Output::ExitCode(n) => std::process::exit(n),
        lir::compile::Output::Binary => {}
    }
}
