{
  description = "An FHS shell for Ideation";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs @ {
    self,
    nixpkgs,
    flake-parts,
    fenix,
    naersk,
    ...
  }:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = ["aarch64-darwin" "x86_64-linux"];
      imports = [];
      perSystem = {system, ...}: let
        pkgs = import nixpkgs {
          inherit system;
          config = {
            allowUnfree = true;
            allowBroken = true;
            allowUnfreePredicate = pkg: true;
            acceptLicense = true;
          };
        };

        # Add naersk-lib definition
        naersk-lib = naersk.lib.${system};

        pkgsStatic = pkgs.pkgsStatic;

        defaultDeps = [
          pkgs.ruff
          pkgs.nodejs
          pkgs.pyright
          pkgs.jq
          pkgs.uv
          pkgs.trufflehog
          pkgs.python311
          pkgs.python311Packages.virtualenv
          pkgs.python311Packages.venvShellHook
          pkgs.nodePackages.pnpm
        ];

        # Rust toolchain
        toolchain = with fenix.packages.${system};
          combine [
            complete.cargo
            complete.rustc
            complete.clippy-preview
            complete.llvm-tools-preview
            complete.rust-analyzer-preview
            complete.rustfmt-preview
            complete.miri-preview
            targets."aarch64-unknown-linux-gnu".latest.rust-std
            targets."x86_64-unknown-linux-gnu".latest.rust-std
            targets."aarch64-apple-darwin".latest.rust-std
          ];

        otherDeps = with pkgs; [
          zig
          toolchain
          pkg-config
          openssl
          openssl.dev
          pkgsStatic.oniguruma
          libiconv
          perl
        ];
      in {
        _module.args = {inherit pkgs;};
        legacyPackages = pkgs;

        # Add packages configuration
        packages = {
          notion2prompt = naersk-lib.buildPackage {
            pname = "notion2prompt";
            root = ./.;
            # For pkg-config
            nativeBuildInputs = with pkgs; [pkg-config perl];
            # For linking against openssl
            buildInputs = with pkgs; [openssl];
            # Skip tests for faster builds or if tests aren't set up
            doCheck = false;
          };
          default = self.packages.${system}.notion2prompt;

          # Add Docker image
          docker = pkgs.dockerTools.buildImage {
            name = "notion2prompt";
            tag = "latest";
            created = "now";

            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [self.packages.${system}.notion2prompt];
              pathsToLink = ["/bin"];
            };

            config = {
              Cmd = ["/bin/notion2prompt"];
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
              ];
            };
          };
        };

        # Add apps configuration
        apps = {
          notion2prompt = {
            type = "app";
            program = "${self.packages.${system}.notion2prompt}/bin/notion2prompt";
          };
          default = self.apps.${system}.notion2prompt;
        };

        devShells = {
          default = (pkgs.mkShell.override {
            stdenv = if pkgs.stdenv.isDarwin then pkgs.stdenv else pkgs.gcc11Stdenv;
          }) {
            name = "ideation-env";
            venvDir = ".venv";
            buildInputs = defaultDeps ++ otherDeps;
            postVenvCreation = ''
              unset SOURCE_DATE_EPOCH
            '';

            packages = defaultDeps ++ otherDeps;
            shellHook = ''
              set -eu
              export UID_DOCKER=$(id -u)
              export GID_DOCKER=$(id -g)
              export TAILSCALE_IP=$(tailscale ip -4 2>/dev/null)
              source .venv/bin/activate
              export PYTHONPATH="$(pwd)/.venv/${pkgs.python311.sitePackages}:$PYTHONPATH:${pkgs.python311}/${pkgs.python311.sitePackages}"
              
              # Use appropriate library path variable based on platform
              if [[ "$(uname)" == "Darwin" ]]; then
                export DYLD_LIBRARY_PATH="${pkgs.openssl.out}/lib:${pkgs.stdenv.cc.cc.lib}/lib/"
              else
                export LD_LIBRARY_PATH="${pkgs.openssl.out}/lib:${pkgs.stdenv.cc.cc.lib}/lib/"
              fi
              
              # Load environment variables from .env file if it exists
              if [ -f .env ]; then
                set -a
                source .env
                set +a
              fi
            '';
          };
        };
      };
    };
}
