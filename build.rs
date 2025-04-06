fn main() {
  compile_backend_header();
  compile_rts_bitcode();
}

fn compile_backend_header() {
  let crate_name = std::env::var("CARGO_PKG_NAME").unwrap();
  let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
  let header_path = std::path::PathBuf::from(format!("lib/{}.h", crate_name));

  cbindgen::Builder::new()
    .with_crate(crate_dir)
    .with_language(cbindgen::Language::C)
    .generate()
    .unwrap()
    .write_to_file(header_path);
}

fn compile_rts_bitcode() {
  let status = std::process::Command::new("rustc")
    .args([
      "--crate-type=lib",
      "--emit=llvm-bc",
      "-O",
      "-o",
      "lib/rts.bc",
      "backend/rts.rs",
    ])
    .status()
    .expect("Failed to execute rustc command");

  if !status.success() {
    panic!("Failed to compile rts");
  }

  println!("cargo:rerun-if-changed=backend/rts.rs");
}
