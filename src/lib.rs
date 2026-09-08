use crate::config::Config;
use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use sqlx::{Pool, Sqlite};
use std::{fs, sync::Arc};

mod api;
pub mod config;

pub type OpentonicError = Box<dyn std::error::Error>;

pub struct List;

struct ServerState {
    pub config: Config,
    pub db_conn: sqlx::Pool<Sqlite>,
}

#[tokio::main]
pub async fn run_server(config: Config) {
    let host_socket = std::net::SocketAddr::new(config.host_address, config.host_port);

    let db_url = "sqlite://".to_string() + config.db_file.to_str().unwrap();
    let db_conn = sqlx::Pool::connect(&db_url)
        .await
        .expect("Could not connect to database");
    init_db(&db_conn).await;

    let url_pfx = config.url_prefix.clone();
    let server_state = Arc::new(ServerState { config, db_conn });
    let server = axum::Router::new()
        .route(&(url_pfx.clone() + "/"), get(index_page))
        .route(&(url_pfx.clone() + "/static/css/{path}"), get(serve_css))
        .route(&(url_pfx.clone() + "/static/js/{path}"), get(serve_js))
        .route(&(url_pfx.clone() + "/api/lists"), get(api::get_lists))
        .route(&(url_pfx + "/api/create-list"), post(api::create_list))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(host_socket)
        .await
        .expect("Failed to bind to socket");
    println!("Starting server...");
    axum::serve(listener, server).await.unwrap();
}

async fn init_db(db: &Pool<Sqlite>) {
    sqlx::query!(
        "
        PRAGMA foreign_keys=on;
        CREATE TABLE IF NOT EXISTS Lists(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            users INTEGER NOT NULL, 
            products INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS List_Accesses(
            list_id INTEGER NOT NULL,
            user TEXT NOT NULL,
            PRIMARY KEY(list_id,user)
            FOREIGN KEY(list_id) REFERENCES Lists(id)
        );
        CREATE TABLE IF NOT EXISTS Categories(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            icon BLOB
        );
        CREATE TABLE IF NOT EXISTS Products(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL, 
            category INTEGER,
            FOREIGN KEY(category) REFERENCES Categories(id)
        );
        CREATE TABLE IF NOT EXISTS ListItems(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checked BOOL DEFAULT false,
            product_id INTEGER NOT NULL,
            FOREIGN KEY(product_id) REFERENCES Products(id)
        )
        "
    )
    .execute(db)
    .await
    .unwrap();
}

async fn index_page(State(state): State<Arc<ServerState>>) -> Html<String> {
    let path = &state.config.html_dir;
    let index_page_contents = fs::read_to_string(path).expect("Can not read index page file");
    Html(index_page_contents)
}

async fn serve_css(
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Css, StatusCode> {
    let full_path = state.config.static_dir.join("css").join(path);
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

async fn serve_js(
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Js, StatusCode> {
    let full_path = state.config.static_dir.join("js").join(path);
    let js_str = fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Js(js_str))
}

struct Js(String);

impl IntoResponse for Js {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header("Content-Type", "text/javascript; charset=utf-8")
            .body(self.0.into())
            .unwrap()
    }
}
