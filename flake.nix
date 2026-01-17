{
  description = "The api and background process of benben.sbs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    (flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        commonAttrs =
          baseDir:
          let
            cargoToml = fromTOML (builtins.readFile "${toString baseDir}/Cargo.toml");
          in
          {
            pname = cargoToml.package.name;
            version = cargoToml.package.version;

            buildAndTestSubdir = baseNameOf (toString baseDir);

            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter =
                filepath: type:
                let
                  baseName = baseNameOf filepath;
                in
                !(pkgs.lib.hasSuffix ".nix" baseName)
                && baseName != ".git"
                && baseName != "target"
                && baseName != "result";
            };

            cargoLock.lockFile = ./Cargo.lock;

            buildInputs = with pkgs; [ openssl ];
            nativeBuildInputs = with pkgs; [ pkg-config ];
          };
      in
      {
        packages = rec {
          benben-api-next = pkgs.rustPlatform.buildRustPackage (commonAttrs ./api);
          circle-preprocessor = pkgs.rustPlatform.buildRustPackage (commonAttrs ./circle-preprocessor);

          default = pkgs.symlinkJoin {
            name = "benben-api-all";
            paths = [
              benben-api-next
              circle-preprocessor
            ];
          };
        };

      }
    ))
    // {
      nixosModules.default = import ./module.nix { inherit self; };
    };
}
