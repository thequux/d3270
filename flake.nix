{
  description = "A very basic flake";

  inputs.nixpkgs.url = "nixpkgs";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { self, nixpkgs, flake-utils }: flake-utils.lib.eachDefaultSystem (system:
    let 
     pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [ 
          clang
          rustup
        ];
        nativeBuildInputs = with pkgs; [
        ];
      };

    });
}
