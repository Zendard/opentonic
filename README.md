# Opentonic
An open source, simple, self-hosted, shared grocery list built in rust

### WIP

## Config
The config is loaded from `/etc/opentonic/config.toml` unless specified otherwise

|Value|Type|Default|Description|
|-----|----|-------|-----------|
|`host_address`|String|`127.0.0.1`|Address on which to host the web server, can be IPv4 or IPv6 (It
is recommended to leave this as the default to not expose opentonic to the internet withouth a
reverse proxy)|
|`host_port`|u16|`80`|Port on which to host the web server|
|`html_dir`|String|`./html`|Path to the HTML files|
|`static`|String|`./static`|Path to the static files|
|`db_file`|String|`./db.sqlite`|Path to the SQLite database file|
|`url_prefix`|String|`/`|Url prefix on which to receive requests (for running under a subdirectory)|

## Authentication
**Opentonic does not provide authentication!**
Please use a reverse proxy like nginx for authentication. Opentonic expects the `X-Forwarded-User`
header to contain the username of the current user. In nginx this can be achieved with 
`proxy_set_header X-Forwarded-User $remote_user;`

### Example nginx config
```
worker_processes  1;

# Load all installed modules
include /etc/nginx/modules.d/*.conf;

events {
    worker_connections  1024;
}

http {
  include       /etc/nginx/mime.types;
  default_type  application/octet-stream;
  sendfile        on;
  keepalive_timeout  65;
  gzip  on;

  server{
    listen 80;
    location / {
      auth_basic "Opentonic";
      auth_basic_user_file htpasswd;
      proxy_pass_request_headers on;
      proxy_set_header X-Forwarded-User $remote_user;
      proxy_pass http://localhost:8080;
    }
  }
}
```
