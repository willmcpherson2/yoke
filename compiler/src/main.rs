mod lir;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <input_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let input = std::fs::read_to_string(input_file)
        .unwrap_or_else(|_| panic!("Failed to read file: {}", input_file));

    let program =
        lir::parse::parse(&input).unwrap_or_else(|err| panic!("Failed to parse input: {}", err));

    let output = program.compile(lir::compile::Config::default());
    println!("{:?}", output);
}
