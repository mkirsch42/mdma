{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    utils,
    naersk,
  }:
    utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {inherit system;};
        naersk-lib = pkgs.callPackage naersk {};
      in {
        defaultPackage = naersk-lib.buildPackage ./.;
        devShell = with pkgs;
          mkShell {
            buildInputs = [cargo rust-analyzer rustc rustfmt pre-commit rustPackages.clippy openssl pkg-config postgresql nodejs];
            RUST_SRC_PATH = rustPlatform.rustLibSrc;
            shellHook = ''
              export NIX_SHELL_DIR=$PWD/.nixshell
              mkdir $NIX_SHELL_DIR
              echo "*" > $NIX_SHELL_DIR/.gitignore

              export PGDATA=$NIX_SHELL_DIR/db
              trap \
                "
                  pg_ctl -D $PGDATA stop
                  cd $PWD
                  rm -rf $NIX_SHELL_DIR
                " \
                EXIT

              if ! [[ -d $PGDATA ]]; then
                pg_ctl initdb -D $PGDATA
              fi

              pg_ctl \
                -D $PGDATA \
                -l $PGDATA/postgres.log \
                -o "-c unix_socket_directories='$PGDATA'" \
                start
            '';
          };
      }
    );
}
