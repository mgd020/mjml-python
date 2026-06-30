{ pkgs ? import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-26.05.tar.gz";
    sha256 = "0psddc9n3c56xzs235x1c5pzm1sfcvi2gf70d30c1c0q2vsg16cb";
}) {} }:

pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
        # https://search.nixos.org/packages
        cargo
        python3
        rustc
        uv
    ] ++ lib.optional stdenv.isDarwin libiconv;
    shellHook = ''
        unset VIRTUAL_ENV
        uv sync --locked
        uv run --locked maturin develop
    '';
}
