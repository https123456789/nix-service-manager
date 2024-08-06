{
  services.a = {
    enabled = true;
    base_dir = ./src;
    run_command = "python -m http.server";
  };
  services.b = {
    enabled = true;
    base_dir = ./.;
    run_command = "python -m http.server 5000";
  };
}
