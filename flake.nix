{
  description = "Rust DevShell";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

    rust-overlay.url = "github:oxalica/rust-overlay";
  };
  outputs = {self, nixpkgs, rust-overlay}:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs { inherit system overlays; };
      rust = pkgs.rust-bin.nightly.latest.default.override {
        extensions = [ "rust-src" ];
        targets = [ "x86_64-unknown-uefi" "x86_64-unknown-none" ];
      };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs;[
          rust
          rust-analyzer
          qemu
        ];
      };
    };
}
