{
  description = "Developing Rust CEC/HTTP applications for ARM (RPI)";

  inputs = {
    nixpkgs.url = "https://channels.nixos.org/nixpkgs-unstable/nixexprs.tar.zst";
  };

  outputs =
    { nixpkgs, self }@inputs:
    {
      devShells = builtins.mapAttrs (
        system: pkgs:
        let
          cross = pkgs.pkgsCross.aarch64-multiplatform-musl;
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustup
              lld
              cross.stdenv.cc
              cross.stdenv.cc.bintools
            ];
            shellHook = "";
          };
        }
      ) inputs.nixpkgs.legacyPackages;

      packages =
        let
          tomlFile = builtins.fromTOML (builtins.readFile ./Cargo.toml);
          mapBinaryToPlatform =
            buildBinary:
            builtins.mapAttrs (
              system: pkgs:
              let
                fs = pkgs.lib.fileset;
                project_files = fs.gitTracked ./.;
                binaries = fs.toList (fs.intersection project_files ./src/bin);
                removeSuffix = pkgs.lib.removeSuffix;
                src = fs.toSource {
                  root = ./.;
                  fileset = project_files;
                };
              in
              builtins.listToAttrs (
                builtins.map (bin_name: {
                  name = bin_name;
                  value = buildBinary { inherit bin_name pkgs src; };
                }) (map (path: removeSuffix ".rs" (builtins.baseNameOf path)) binaries)
              )
            ) inputs.nixpkgs.legacyPackages;
        in
        mapBinaryToPlatform (
          {
            bin_name,
            pkgs,
            src,
          }:
          let
            cross = pkgs.pkgsCross.aarch64-multiplatform-musl;
            target = "aarch64-unknown-linux-musl";
            name = "http-tv-${bin_name}";
            version = tomlFile.package.version;
          in
          cross.rustPlatform.buildRustPackage {
            inherit name version src;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with cross; [
              lld
              stdenv.cc
              stdenv.cc.bintools
            ];

            buildPhase = ''
              cargo build --release --target=${target} --bin=${bin_name}
            '';

            installPhase = ''
              mkdir -p $out/bin/
              cp target/${target}/release/${bin_name} $out/bin/
            '';

            RUSTFLAGS = "-C target-feature=+crt-static";
          }
        );
    };
}
