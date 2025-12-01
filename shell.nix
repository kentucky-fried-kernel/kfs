{
  pkgs ? import <nixpkgs> { },
}:
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.callPackage (
  {
    stdenv,
    mkShell,
    rustup,
    rustPlatform,
  }:
  mkShell {
    strictDeps = true;
    nativeBuildInputs = [
      rustup
      pkgs.rust-analyzer
      rustPlatform.bindgenHook
      pkgs.busybox
      pkgs.qemu
      pkgs.grub2
      pkgs.xorriso
      pkgs.mtools
      pkgs.xz
      pkgs.coreboot-toolchain.i386
    ];
    # libraries here
    buildInputs = with pkgs; [
    ];

    RUSTC_VERSION = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  }
) { }
