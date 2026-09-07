# Opentonic
An open source, simple, self-hosted, shared grocery list built in rust

### WIP

## Config
The config is loaded from `/etc/opentonic/config.toml` unless specified otherwise

|Value|Type|Default|Description|
|-----|----|-----------|
|`host_address`|String|`127.0.0.1`|Address on which to host the web server, can be IPv4 or IPv6|
|`host_port`|u16|`80`|Port on which to host the web server|
