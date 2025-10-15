{
  description = "Development environment for woplanner written in rust with htmx";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # gives us access to more rust versions
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    # utils to build flakes for different systems
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];

        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rustfmt"
            "clippy"
          ];
        };

        # Build-only toolchain (without dev extensions for smaller builds)
        rustToolchainBuild = pkgs.rust-bin.stable.latest.default;

        nativeBuildInputs = with pkgs; [
          rustToolchain
          sqlite
          cargo-watch
          cargo-edit
          tailwindcss
          tailwindcss-language-server
          pkg-config
          openssl
        ];

        buildInputs = with pkgs; [
          openssl
          sqlite
        ] ++ lib.optionals pkgs.stdenv.isDarwin [
          pkgs.darwin.apple_sdk.frameworks.Security
          pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
        ];

      in
      {
        # Production package build
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "woplanner";
          version = "0.1.0";

          src = ./.;

          cargoLock = {
            lockFile = ./Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            rustToolchainBuild
          ];

          buildInputs = with pkgs; [
            openssl
            sqlite
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            pkgs.darwin.apple_sdk.frameworks.Security
            pkgs.darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          # Copy static files, templates, migrations to output
          postInstall = ''
            mkdir -p $out/share/woplanner
            cp -r static templates migrations seeds $out/share/woplanner/

            # Create wrapper script that sets correct paths
            mv $out/bin/woplanner $out/bin/.woplanner-wrapped
            cat > $out/bin/woplanner <<EOF
            #!/bin/sh
            cd $out/share/woplanner
            exec $out/bin/.woplanner-wrapped "\$@"
            EOF
            chmod +x $out/bin/woplanner
          '';

          meta = with pkgs.lib; {
            description = "WOPlanner - Progressive Web App for strength training";
            homepage = "https://github.com/yourusername/woplanner";
            license = licenses.mit;
            maintainers = [ "David Dähne" ];
          };
        };

        # OCI/Docker image
        packages.docker = pkgs.dockerTools.buildImage {
          name = "woplanner";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "woplanner-env";
            paths = with pkgs; [
              self.packages.${system}.default
              sqlite
              cacert  # For HTTPS
              coreutils
              bashInteractive
            ];
            pathsToLink = [ "/bin" "/share" ];
          };

          config = {
            Cmd = [ "${self.packages.${system}.default}/bin/woplanner" ];
            ExposedPorts = {
              "3000/tcp" = {};
            };
            Env = [
              "DATABASE_URL=sqlite:/data/woplanner.db"
              "SEED_DATABASE=false"
              "PORT=3000"
            ];
            WorkingDir = "/data";
            Volumes = {
              "/data" = {};
            };
          };
        };

        # Flake app for easy running
        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/woplanner";
        };

        # Development shell
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          shellHook = ''
            echo "🏋️ WOPlanner Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo run            - Run the application"
            echo "  cargo build          - Build the application"
            echo "  cargo watch -x run   - Auto-restart on changes"
            echo "  cargo test           - Run tests"
            echo "  cargo clippy         - Run linter"
            echo "  cargo fmt            - Format code"
            echo "  tailwindcss --help   - TailwindCSS commands"
            echo ""
            echo "Nix commands:"
            echo "  nix build            - Build production package"
            echo "  nix build .#docker   - Build OCI/Docker image"
            echo "  nix run              - Run production build"
            echo ""
            echo "Database:"
            echo "  DATABASE_URL=${"\$DATABASE_URL"}"
            echo "  SEED_DATABASE=${"\$SEED_DATABASE"} (set to 'false' to disable sample data)"
            echo ""

            # Rust-Analyzer für bessere IDE-Unterstützung
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"

            # Database URL (SQLite als Standard)
            export DATABASE_URL="sqlite:./bodybuilding.db"

            # Enable seeds in development by default
            export SEED_DATABASE="''${SEED_DATABASE:-true}"
          '';
        };
      });
}
