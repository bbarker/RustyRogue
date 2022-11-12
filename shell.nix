#
# Adapted from: https://nixos.wiki/wiki/Rust
#
{ pkgs ? import <nixpkgs> {} }:
  pkgs.mkShell rec {
    buildInputs = with pkgs; [
      llvmPackages_latest.llvm
      llvmPackages_latest.bintools
      zlib.out
      cmake # used by cargo?
      pkg-config
      fontconfig
      rustup
      # xorriso
      # grub2
      # qemu
      llvmPackages_latest.lld
      python3
    ];
    RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    LIBCLANG_PATH = pkgs.lib.makeLibraryPath [ pkgs.llvmPackages_latest.libclang.lib ];
    HISTFILE = toString ./.history;
    shellHook = ''
      export PATH=$PATH:''${CARGO_HOME:-~/.cargo}/bin
      export PATH=$PATH:''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-x86_64-unknown-linux-gnu/bin/
      export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.xorg.libX11}/lib
      export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.xorg.libXcursor}/lib
      export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.xorg.libXrandr}/lib
      export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.xorg.libXi}/lib
      export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:${pkgs.libGL}/lib
      '';
    # Add libvmi precompiled library to rustc search path
    RUSTFLAGS = (builtins.map (a: ''-L ${a}/lib'') [
      # pkgs.libvmi
    ]);
    # Add libvmi, glibc, clang, glib headers to bindgen search path
    BINDGEN_EXTRA_CLANG_ARGS = 
    # Includes with normal include path
    (builtins.map (a: ''-I"${a}/include"'') [
      # pkgs.libvmi
      pkgs.glibc.dev 
    ])
    # Includes with special directory paths
    ++ [
      ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
      ''-I"${pkgs.glib.dev}/include/glib-2.0"''
      ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
    ];

  }
