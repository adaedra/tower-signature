{
  inputs = {
    nixpkgs.url = "nixpkgs";
  };

  outputs = { self, nixpkgs }: {
    devShells.x86_64-darwin.default = let
      pkgs = import nixpkgs { system = "x86_64-darwin"; };
    in pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        rustc
        cargo
        rustfmt
        libiconv
        darwin.apple_sdk.frameworks.Security
      ];

      RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
    };
  };
}
