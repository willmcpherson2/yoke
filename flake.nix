{
  description = "Rust + LLVM";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        llvm = pkgs.llvmPackages_18.llvm;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            llvm
            pkgs.libffi
            pkgs.libxml2
            (pkgs.rust-bin.nightly."2024-07-31".default.override {
              extensions = [
                "rust-src"
                "rust-analyzer-preview"
                "miri"
              ];
            })
          ];
        };
      }
    );
}
