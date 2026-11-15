# Nix shell for building/flashing. The prebuilt ESP-IDF tools and the espup
# Rust toolchain are glibc binaries, so run them inside an FHS environment
# instead of requiring nix-ld.
{ pkgs ? import <nixpkgs> { } }:

(pkgs.buildFHSEnv {
  name = "water-management";
  targetPkgs = pkgs: with pkgs; [
    rustup
    espflash
    ldproxy
    cmake
    ninja
    git
    python3
    pkg-config
    openssl
    gcc
  ];
  profile = ''
    export PATH="$HOME/.cargo/bin:$PATH"
    [ -f "$HOME/export-esp.sh" ] && . "$HOME/export-esp.sh"
  '';
  runScript = "bash";
}).env
