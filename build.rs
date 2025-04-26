fn main() {
    std::process::Command::new("rustc")
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

    println!("cargo:rerun-if-changed=src/rts.rs");
}
