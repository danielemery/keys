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
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [ bashInteractive pkg-config ];
            buildInputs = with pkgs; [
              deno
              doppler
              rustc
              cargo
              openssl
              gnupg
            ];
          };
        });
    };
}
