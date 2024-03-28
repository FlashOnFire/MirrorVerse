{
  inputs = {
    nixpkgs.url = "nixpkgs";

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    flake-parts,
    rust-overlay,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin"];

      perSystem = {
        lib,
        system,
        ...
      }: let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [(import rust-overlay)];
        };
      in {
        devShells.default = with pkgs;
          mkShell {
            nativeBuildInputs = [cmake pkg-config];

            buildInputs = [
              (rust-bin.stable.latest.default.override {
                extensions = ["rust-analyzer" "rust-src"];
              })
              fontconfig
            ];

            LD_LIBRARY_PATH = lib.makeLibraryPath [
              vulkan-loader
              libxkbcommon
              wayland
              libGL
            ];
          };

        formatter = pkgs.alejandra;
      };
    };
}
