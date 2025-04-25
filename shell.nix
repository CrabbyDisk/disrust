{
  pkgs ?
    import <nixpkgs> {
      overlays = [(import "${fetchTarball "https://github.com/nix-community/fenix/archive/monthly.tar.gz"}/overlay.nix")];
      config.allowUnfree = true;
    },
}:
pkgs.mkShell.override {
  stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.clangStdenv;
} rec {
  # Get dependencies from the main package
  # Additional tooling
  buildInputs = with pkgs; [
    (fenix.default.toolchain)
    openssl.dev
    pkg-config
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath buildInputs;
  WINIT_UNIX_BACKEND = "wayland";
}
