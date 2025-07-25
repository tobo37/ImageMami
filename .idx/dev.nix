{pkgs}: {
  channel = "stable-24.05";
  packages = [
    pkgs.nodejs_20
    pkgs.rustc
    pkgs.cargo
  ];
  idx.extensions = [
    "vue.volar"
  ];
  idx.previews = {
    previews = {
      web = {
        command = [
          "npm"
          "run"
          "dev"
          "--"
          "--port"
          "$PORT"
          "--host"
          "0.0.0.0"
        ];
        manager = "web";
      };
    };
  };
}