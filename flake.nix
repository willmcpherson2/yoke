{
  description = "Haskell 💞 Rust";

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
            pkgs.cabal-install
            pkgs.haskell.compiler.ghc910
            pkgs.haskell.packages.ghc910.haskell-language-server
            pkgs.haskellPackages.cabal-fmt
            pkgs.haskellPackages.ormolu
            (pkgs.rust-bin.stable."1.81.0".default.override {
              extensions = [
                "rust-src"
                "rust-analyzer-preview"
              ];
            })
            llvm
            pkgs.libxml2
          ];
          shellHook = ''
            export LD_LIBRARY_PATH=lib:$LD_LIBRARY_PATH
            export LLVM_SYS_181_PREFIX=${llvm.lib}
          '';
        };
      }
    );
}
