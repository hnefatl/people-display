{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs";
    naersk.url = "github:nix-community/naersk";
    # Just used for rustChannelOf
    nixpkgs-mozilla.url = "github:mozilla/nixpkgs-mozilla";
  };

  outputs =
    inputs@{
      self,
      nixpkgs,
      flake-parts,
      naersk,
      nixpkgs-mozilla,
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import nixpkgs-mozilla)
        ];
        config = { };
      };

      toolchain =
        (pkgs.rustChannelOf {
          rustToolchain = ./rust-toolchain.toml;
          sha256 = "sha256-27Fpm3bG8ax9gh0qzwqg1ef3Y/HDX73DplSkJUzmCyc=";
        }).rust;

      naersk' = pkgs.callPackage naersk {
        cargo = toolchain;
        rustc = toolchain;
      };
    in
    rec {
      packages.${system}.default = naersk'.buildPackage rec {
        src = ./.;
        nativeBuildInputs = with pkgs; [
          openssl
          SDL2
          SDL2_image
          protobuf
        ];
        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
      };

      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs =
          with pkgs;
          [
            cargo
            pkg-config
          ]
          ++ packages.${system}.default.nativeBuildInputs;

        RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
          pkgs.SDL2
          pkgs.SDL2_image
          pkgs.openssl
        ];
      };
    };
}
