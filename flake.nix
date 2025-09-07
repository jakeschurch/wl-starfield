{
  description = "Starfield Rust for NixOS Unified with Wayland + Nvidia + Vulkan";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs =
    {
      self,
      nixpkgs,
      crane,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        # Library dependencies for Wayland/Vulkan/OpenGL
        libDeps = with pkgs; [
          libGL
          libxkbcommon
          wayland
          vulkan-loader
        ];
        libPath = pkgs.lib.makeLibraryPath libDeps;

        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          nativeBuildInputs = with pkgs; [
            pkg-config
            makeWrapper
          ];

          buildInputs =
            libDeps
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];
        };

        wl-starfield = craneLib.buildPackage (
          commonArgs
          // {
            pname = "wl-starfield";
            version = "0.1.0";

            cargoArtifacts = craneLib.buildDepsOnly commonArgs;

            # Additional environment variables or build phases/hooks can be set
            # here *without* rebuilding all dependency crates
            CARGO_BUILD_INCREMENTAL = "false";
            RUST_BACKTRACE = "1";

            postInstall = ''
              wrapProgram $out/bin/wl-starfield \
                --set LD_LIBRARY_PATH "${libPath}"
            '';
          }
        );
      in
      {
        checks = {
          inherit wl-starfield;
        };

        packages.default = wl-starfield;

        apps.default = flake-utils.lib.mkApp {
          drv = wl-starfield;
        };

        devShells.default = craneLib.devShell {
          # Inherit inputs from checks.
          checks = self.checks.${system};

          # Additional dev-shell environment variables
          LD_LIBRARY_PATH = libPath;
          RUST_BACKTRACE = "1";

          # Extra inputs for development
          packages = with pkgs; [
            rust-analyzer
          ];
        };
      }
    );
}
