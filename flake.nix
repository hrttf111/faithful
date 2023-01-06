{
  outputs = { self, nixpkgs }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ self.overlay ];
      };
      faithful-pkg-env = pkgs.faithful-pkg.overrideAttrs (p: p // {
        buildInputs = with pkgs; [
            gdb
            xorg.libX11
            glslang
            shaderc
            shaderc.bin
            clippy
        ] ++ p.buildInputs;
        APPEND_LIBRARY_PATH = with pkgs; lib.strings.makeLibraryPath [
          vulkan-loader
          xorg.libXcursor
          xorg.libXi
          xorg.libXrandr
        ];
        shellHook = ''
          echo ${pkgs.shaderc.bin}
          echo ${pkgs.glslang}
          echo ${pkgs.gdb}
          echo ${pkgs.clippy}
          export RUST_BACKTRACE=1
          export PATH="$PATH:${pkgs.shaderc.bin}/bin:${pkgs.gdb}/bin:${pkgs.clippy}/bin"
          export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:$APPEND_LIBRARY_PATH"
          export WINIT_UNIX_BACKEND=x11
        '';
    });
    in rec {
      overlay = final: prev: {
        faithful-pkg = prev.callPackage ./default.nix {};
      };

      packages.x86_64-linux.faithful-pkg = faithful-pkg-env;
      defaultPackage.x86_64-linux = faithful-pkg-env;
    };
}
