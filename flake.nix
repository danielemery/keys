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
      devShells = forAllSystems (system:
        let pkgs = import nixpkgs { inherit system; };
        in {
          default = pkgs.mkShell {
            nativeBuildInputs = [ pkgs.bashInteractive ];
            buildInputs = with pkgs; [
              deno
              doppler
            ];
          };
        });
    };
}
