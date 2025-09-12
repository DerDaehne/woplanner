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
        
        nativeBuildInputs = with pkgs; [
          rustToolchain
          sqlite
          cargo-watch
          cargo-edit
          tailwindcss
          tailwindcss-language-server
          pkgs-config
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
        devShells.default = pkgs.mkShell {
          inherit buildInputs nativeBuildInputs;

          shellHook = ''
            echo "🏋️  Bodybuilding Tracker Development Environment"
            echo "Rust version: $(rustc --version)"
            echo "Cargo version: $(cargo --version)"
            echo ""
            echo "Available commands:"
            echo "  cargo run           - Run the application"
            echo "  cargo watch -x run  - Auto-restart on changes"
            echo "  cargo test          - Run tests"
            echo "  cargo clippy        - Run linter"
            echo "  cargo fmt           - Format code"
            echo "  tailwindcss --help  - TailwindCSS commands"
            echo ""
            
            # Rust-Analyzer für bessere IDE-Unterstützung
            export RUST_SRC_PATH="${rustToolchain}/lib/rustlib/src/rust/library"
            
            # Database URL (SQLite als Standard)
            export DATABASE_URL="sqlite:./bodybuilding.db"
          '';
        };
      });
}
