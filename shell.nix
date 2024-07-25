{ pkgs ? import <nixpkgs> {} }:
let
    # rust-rover things
    rust-toolchain =
        pkgs.symlinkJoin {
            name = "rust-toolchain";
            paths = with pkgs; [rustc cargo rustPlatform.rustcSrc clippy rustfmt gcc rust-analyzer];
        };
in
pkgs.mkShell rec {
    buildInputs = with pkgs;[
        openssl
        pkg-config
        cmake
        zlib
        rust-toolchain

        # common glutin
        libxkbcommon
        libGL

        # winit wayland
        wayland

        # winit x11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libX11
    ];
    nativeBuildInputs = with pkgs; [
        pkg-config
        fontconfig
    ];
    LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath buildInputs}";
    OPENSSL_DIR="${pkgs.openssl.dev}";
    OPENSSL_LIB_DIR="${pkgs.openssl.out}/lib";
    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
