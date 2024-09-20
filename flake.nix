{
  description = "Telegram client for the terminal written in Rust.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    # contains tdlib 1.8.29
    nixpkgs-old.url = "github:NixOS/nixpkgs/c5187508b11177ef4278edf19616f44f21cc8c69";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      nixpkgs-old,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        pkgs-old = import nixpkgs-old { inherit system; };
      in
      {
        packages = rec {
          tgt = pkgs.rustPlatform.buildRustPackage {
            pname = "tgt";
            version = "1.0.0";
            src = ./.;
            cargoHash = "sha256-val5GoLPDo6uuJUpXWeEuvmw6+GuqtQszoY/3lO2JSQ=";
            buildNoDefaultFeatures = true;
            buildFeatures = [ "pkg-config" ];

            nativeBuildInputs = with pkgs; [ pkg-config ];
            buildInputs = [
              pkgs.openssl
              pkgs-old.tdlib
            ];

            configurePhase = ''
              runHook preConfigure
              export HOME=$(mktemp -d)
              runHook postConfigure
            '';
          };

          default = tgt;
        };
      }
    );
}
