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
            version = "unstable-2024-11-04";
            src = pkgs.fetchFromGitHub {
              owner = "FedericoBruzzone";
              repo = "tgt";
              rev = "39fb4acec241e2db384e268c77e875bd13a48c12";
              sha256 = "sha256-McZEnRwtGEuhDA1uJ1FgUl6QiPfzCDr/Pl2haF9+MRw=";
            };

            nativeBuildInputs =
              rlinkLibs
              ++ lib.optional isDarwin (builtins.attrValues { inherit (pkgs) apple-sdk_12; });

            buildInputs = rlinkLibs;

            patches = [ ./patches/0001-check-filesystem-writability-before-operations.patch ];

            # Tests are broken on nix
            doCheck = false;

            cargoHash = "sha256-WIs9rVhTQn217DHIw1SPnQrkDtozEl2jfqVjTwJHF2w=";
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
