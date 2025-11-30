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
      rustPlatform.bindgenHook
      pkgs.busybox
      pkgs.qemu
      pkgs.grub2
      pkgs.xorriso
      pkgs.mtools
      pkgs.xz
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
# let
#  pkgs = import (builtins.fetchTarball {
#     url = "https://github.com/NixOS/nixpkgs/archive/a84b0a7c509bdbaafbe6fe6e947bdaa98acafb99.tar.gz";
#     sha256 = "0m8zrg4rp5mx5v9ar91ncnjhagmcrd3y9h56y48swan6a8gwpq52";
#   }) {};
# in
#
# pkgs.mkShell {
#   name = "kfs";
#
#
#   buildInputs = with pkgs; [
#     gdb
#     git
#     rustup
#     qemu
#     grub2
#     xorriso
#     mtools
#     xz
#   ];
#
#   shellHook = ''
#     # if [ "''${IN_NIX_SHELL:-}" != "pure" ]; then
#     #   echo "❗ Error: This script must be run inside a Nix shell with pure mode. Run 'nix-shell --pure' IN_NIX_SHELL==''${IN_NIX_SHELL:-})"
#     #   exit 1
#     # fi
#
#     set -o vi
#     export PS1="\[\e[0;32m\]\W>\[\e[0m\] "
#
#     rustup default nightly
#     rustup component add cargo rust-analyzer rustfmt
#
#   '';
# }
