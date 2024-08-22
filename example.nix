{
  # Enable debug logs
  debug = true;

  # Specify a path where all service source files are stored and managed
  root = ./demo-services-root;

  services.a = {
    enabled = true;

    # Use base_dir for services with a source that is local and git_uri for remote services that
    # can be cloned from a git server
    base_dir = ./src;

    # You can also specify environment variables
    env.PORT = "3000";

    run_command = "sh -c 'sleep 100 & python -m http.server $PORT'";
  };

  services.b = {
    enabled = true;
    git_uri = "https://github.com/TheWaWaR/simple-http-server.git";
    run_command = "cargo run -- -i -p 5000 .";
  };
}
