{
  description = "SimpleComicsViewer dev flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        includeList = [
          ''-I"${pkgs.llvmPackages_latest.libclang.lib}/lib/clang/${pkgs.llvmPackages_latest.libclang.version}/include"''
          ''-I"${pkgs.glib.dev}/include/glib-2.0"''
          ''-I${pkgs.glib.out}/lib/glib-2.0/include/''
        ];
        allCflags = builtins.concatStringsSep " " ++ includeList;
      in
        {
          RUST_NIX_CFLAGS = allCflags;
          devShells.default = with pkgs; mkShell {
            LIBCLANG_PATH = "${pkgs.llvmPackages_latest.libclang.lib}/lib";
            LD_LIBRARY_PATH = lib.makeLibraryPath [
              gtk4
              gdk-pixbuf
              cairo
              fontconfig
              freetype
              libarchive
              mupdf
              gumbo
              jbig2dec
              libpng
              libjpeg
              openjpeg
              leptonica
              tesseract
              zxing
              sqlite
              glib
              zlib
              stdenv.cc.cc.lib
              harfbuzz
              pango
              graphene
            ];

            buildInputs = [
              pkg-config
              openssl

              # depend libs
              gtk4
              gdk-pixbuf
              cairo
              freetype
              fontconfig
              libarchive
              mupdf
              gumbo
              jbig2dec
              libpng
              libjpeg
              openjpeg
              leptonica
              tesseract
              zxing
              sqlite

              # linker
              mold

              # c and lib
              clang
              glib
              zlib
              stdenv.cc.cc.lib
              llvmPackages.bintools

              # rust
              rust-bin.stable.latest.default
              rust-analyzer
            ];
            nativeBuildInputs = with pkgs; [
              pkg-config
              glibc.dev
              clang
            ];
            shellHook = ''
            export XDG_DATA_DIRS=$GSETTINGS_SCHEMAS_PATH:$XDG_DATA_DIRS
            exec zsh
            '';
          };
        }
    );
}
