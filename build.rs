fn main() {
    let status = std::process::Command::new("rustc")
        .args([
            "--crate-type=lib",
            "--emit=llvm-bc",
            "-O",
            "-o",
            "target/rts.bc",
            "src/rts.rs",
        ])
        .status()
        .expect("Failed to execute rustc command");

    if !status.success() {
        panic!("Failed to compile rts");
    }

    println!("cargo:rerun-if-changed=src/rts.rs");
}
