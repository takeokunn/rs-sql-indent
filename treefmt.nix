{
  projectRootFile = "flake.nix";
  programs.rustfmt.enable = true;
  programs.nixfmt.enable = true;
  programs.prettier = {
    enable = true;
    excludes = [ "**/pnpm-lock.yaml" ];
  };
  programs.taplo.enable = true;
}
