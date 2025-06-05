mod lir;

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
        Err(e) => {
            eprintln!("Failed to parse program: {}", e);
            std::process::exit(3);
        }
    };

    let output = program.compile(lir::compile::Config::default());
    println!("{:?}", output);
}
