use crate::config::Config;
use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use std::{fs, sync::Arc};

pub mod config;
pub type OpentonicError = Box<dyn std::error::Error>;

struct ServerState {
    pub config: Config,
}

#[tokio::main]
pub async fn run_server(config: Config) {
    let host_socket = std::net::SocketAddr::new(config.host_address, config.host_port);

    let server_state = Arc::new(ServerState { config });
    let server = axum::Router::new()
        .route("/", get(index_page))
        .route("/static/css/{path}", get(serve_css))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(host_socket)
        .await
        .expect("Failed to bind to socket");
    println!("Starting server...");
    axum::serve(listener, server).await.unwrap();
}

async fn index_page(State(state): State<Arc<ServerState>>) -> Html<String> {
    let path = &state.config.html_path;
    let index_page_contents = fs::read_to_string(path).expect("Can not read index page file");
    Html(index_page_contents)
}

async fn serve_css(
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Css, StatusCode> {
    let full_path = state.config.static_path.join("css").join(path);
    let css_str = fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Css(css_str))
}

struct Css(String);

impl IntoResponse for Css {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header("Content-Type", "text/css; charset=utf-8")
            .body(self.0.into())
            .unwrap()
    }
}
