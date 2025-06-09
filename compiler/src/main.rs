mod lir;

use clap::Parser;

/// The Yoke compiler
#[derive(Parser, Debug)]
#[command(name = "yoke", version, about, long_about = None)]
struct Args {
    /// File to compile
    input: String,

    /// Interpret input as code instead of a filename
    #[arg(short, long)]
    code: bool,

    /// Evaluate instead of compile
    #[arg(short, long)]
    eval: bool,

    /// Optimization level
    #[arg(
        short = 'O',
        long,
        value_name = "LEVEL",
        value_parser = clap::value_parser!(u8).range(0..=3),
        default_value_t = 0,
    )]
    optimize: u8,
}

fn main() {
    let args = Args::parse();

    let (file, input) = if args.code {
        ("<cli>", args.input)
    } else {
        match std::fs::read_to_string(&args.input) {
            Ok(input) => (args.input.as_str(), input),
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                std::process::exit(1);
            }
        }
    };

    let program = match lir::parse::parse(&input) {
        Ok(program) => program,
        Err(errors) => {
            lir::parse::print_parse_errors(file, &input, errors);
            std::process::exit(2);
        }
    };

    let config = lir::compile::Config {
        mode: if args.eval {
            lir::compile::Mode::Jit
        } else {
            lir::compile::Mode::Aot
        },
        opt_level: match args.optimize {
            0 => lir::compile::OptLevel::O0,
            1 => lir::compile::OptLevel::O1,
            2 => lir::compile::OptLevel::O2,
            3 => lir::compile::OptLevel::O3,
            _ => panic!(),
        },
    };

    let output = lir::compile::compile(&program, config);
    match output {
        lir::compile::Output::ExitCode(n) => std::process::exit(n),
        lir::compile::Output::Binary => {}
    }
}
