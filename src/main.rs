use opentonic::config;

fn main() {
    let mut args = std::env::args();
    args.next();
    let config_option = args.find(|e| e == "-c" || e == "--config");
    let config_location = if config_option.is_some() {
        Some(args.next().expect("Please provide a config file location"))
    } else {
        None
    };

    let config =
        config::Config::get(config_location.as_deref()).expect("Could not load config file");

    opentonic::run_server(config);
}
