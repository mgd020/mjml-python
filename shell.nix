{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
    nativeBuildInputs = with pkgs.buildPackages; [
        # https://search.nixos.org/packages
        python3
        rustc
        cargo
    ] ++ lib.optional stdenv.isDarwin libiconv;
    shellHook = ''
        export VIRTUAL_ENV_DISABLE_PROMPT=1
        python3 -m venv .env
        . .env/bin/activate
        pip install -r requirements.txt
        maturin develop
    '';
}
