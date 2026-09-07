use std::net::Ipv4Addr;

use opentonic::config::{self};

#[test]
fn load_config() {
    let config_path = std::path::Path::new("test_config.toml");

    std::fs::write(
        config_path,
        "
    host_address=\"1.2.3.4\"
    host_port=1234
        ",
    )
    .unwrap();

    let config = config::Config::get(config_path.to_str()).unwrap();
    assert_eq!(config.host_port, 1234);
    assert_eq!(config.host_address, Ipv4Addr::new(1, 2, 3, 4))
}
