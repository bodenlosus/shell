{
  description = "A Nix-flake-based Rust development environment";
  
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        loaders = pkgs.callPackage ./pixbufmodfile.nix {};
      in
      {
        # packages.default = pkgs.callPackage ./. {};  
        devShells.default = pkgs.mkShell {
          shellHook = ''
            export GDK_PIXBUF_MODULE_FILE=${loaders}/loaders.cache
          '';
          packages = with pkgs; [
            loaders
            gdk-pixbuf
            webp-pixbuf-loader
            gtk4
            rustToolchain
            openssl
            pkg-config
            cargo-deny
            cargo-edit
            cargo-watch
            rust-analyzer
            gtk4-layer-shell
            libadwaita
            clang
            inspector
            # ffmpeg
            rustPlatform.bindgenHook
            llvmPackages.libclang
        ];
          env = {
            LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}";
            # BINDGEN_EXTRA_CLANG_ARGS = "-isystem ${pkgs.llvmPackages.libclang.lib}/lib/clang/${pkgs.lib.getVersion pkgs.clang}/include";
            RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
            GDK_PIXBUF_MODULE_FILE = "${loaders}/loaders.cache";
          };
        };
      });
}