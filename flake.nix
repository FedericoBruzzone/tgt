{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    inputs:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];
      forEachSystem = inputs.nixpkgs.lib.genAttrs systems;
    in
    {
      packages = forEachSystem (
        system:
        let
          pkgs = inputs.nixpkgs.legacyPackages.${system};
          inherit (pkgs) lib;
          inherit (pkgs.darwin.apple_sdk) frameworks;
          inherit (pkgs.stdenvNoCC.hostPlatform) isDarwin;
          tdlib = pkgs.tdlib.overrideAttrs {
            version = "1.8.29";
            src = pkgs.fetchFromGitHub {
              owner = "tdlib";
              repo = "td";
              rev = "af69dd4397b6dc1bf23ba0fd0bf429fcba6454f6";
              hash = "sha256-2RhKSxy0AvuA74LHI86pqUxv9oJZ+ZxxDe4TPI5UYxE=";
            };
          };
          rlinkLibs = builtins.attrValues {
            inherit (pkgs)
              pkg-config
              openssl
              ;
            inherit tdlib;
          };
        in
        {
          default = pkgs.rustPlatform.buildRustPackage {
            pname = "tgt";
            version = "unstable-2024-10-21";
            src = pkgs.fetchFromGitHub {
              owner = "FedericoBruzzone";
              repo = "tgt";
              rev = "470b6052dd66ff55f6039bbf940902f503fb67e2";
              sha256 = "sha256-TDxzQpir9KY6rl34YJ5IHFjfMRYzbGlPI58M9i+G9+Y=";
            };

            nativeBuildInputs = rlinkLibs;

            buildInputs =
              rlinkLibs
              ++ lib.optional isDarwin (
                builtins.attrValues {
                  inherit (frameworks)
                    AppKit
                    CoreGraphics
                    Security
                    SystemConfiguration
                    ;
                }
              );

            patches = [ ./patches/0001-check-filesystem-writability-before-operations.patch ];

            # Tests are broken on nix
            doCheck = false;

            cargoHash = "sha256-QqvP/ULAEn+N8w01kDq4pltP4xHoUhNJPZgP/76hhBo=";
            buildNoDefaultFeatures = true;
            buildFeatures = [ "pkg-config" ];

            env = {
              RUSTFLAGS = "-C link-arg=-Wl,-rpath,${tdlib}/lib -L ${pkgs.openssl}/lib";
              LOCAL_TDLIB_PATH = "${tdlib}/lib";
            };

            meta = {
              description = "TUI for Telegram written in Rust";
              homepage = "https://github.com/FedericoBruzzone/tgt";
              license = lib.licenses.free;
              maintainers = with lib.maintainers; [ donteatoreo ];
            };
          };
        }
      );
    };
}
