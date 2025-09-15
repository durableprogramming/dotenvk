{ pkgs, lib, config, inputs, ... }:

{
  # https://devenv.sh/packages/
  packages = with pkgs; [ git libyaml openssl cargo-tarpaulin grcov ];

  languages.rust = {
    enable = true;
  };


  enterShell = ''

  '';

}
