{
  description = "Keys Development Environment";

  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; };

  outputs = { self, nixpkgs }:
    let
      inherit (nixpkgs.lib) genAttrs;
      supportedSystems = [
        "aarch64-darwin"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forAllSystems = f: genAttrs supportedSystems (system: f system);
    in {
      packages = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "keys";
            version = "0.0.0";
            src = ./cli;
            cargoLock.lockFile = ./cli/Cargo.lock;
            
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            # `gpg` must be on PATH during the test suite for the PGP import
            # integration test to run.
            nativeCheckInputs = with pkgs; [
              gnupg
            ];
          };
        });

      devShells = forAllSystems (system:
        let
          pkgs = import nixpkgs { inherit system; };
          # cargo-llvm-cov needs llvm-cov/llvm-profdata from the same LLVM major
          # version that rustc bundles (currently LLVM 20).
          llvm = pkgs.llvmPackages_20.llvm;
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ bashInteractive pkg-config ];
            buildInputs = with pkgs; [
              deno
              doppler
              rustc
              cargo
              # Dev tooling to match the checks run in CI (test.yml):
              # `cargo fmt`, `cargo clippy`, and `cargo llvm-cov`.
              rustfmt
              clippy
              cargo-llvm-cov
              # Required to build the crate (reqwest -> openssl-sys).
              openssl
              # Required by the PGP import integration test.
              gnupg
            ];

            # Point cargo-llvm-cov at LLVM's coverage tools, since the nixpkgs
            # rustc does not ship the llvm-tools-preview component.
            LLVM_COV = "${llvm}/bin/llvm-cov";
            LLVM_PROFDATA = "${llvm}/bin/llvm-profdata";
          };
        });
    };
}
