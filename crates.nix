{...}: {
  perSystem = {
    pkgs,
    config,
    ...
  }: let
    crateName = "nix-service-manager";
  in {
    nci.projects.${crateName}.path = ./.;

    nci.crates.${crateName} =
    let
      sysPkgs = [ pkgs.pkg-config pkgs.openssl ];
    in {
        depsDrvConfig = {
          mkDerivation = {
            buildInputs = sysPkgs;
          };
        };

        drvConfig = {
          mkDerivation = {
            buildInputs = sysPkgs;
          };
        };
    };
  };
}
