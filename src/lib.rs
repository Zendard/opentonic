use crate::config::Config;
use axum::{
    extract::{self, State},
    http::{HeaderMap, StatusCode},
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
        .route(
            &(url_pfx.clone() + "/api/create-list"),
            post(api::create_list),
        )
        .route(
            &(url_pfx.clone() + "/api/list/{list_id}"),
            get(api::fetch_list),
        )
        .route(&(url_pfx.clone() + "/list/{list_id}"), get(fetch_list))
        .route(&(url_pfx.clone() + "/{path}"), get(serve_html))
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
            owner TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS List_Accesses(
            list_id INTEGER NOT NULL,
            user TEXT NOT NULL,
            PRIMARY KEY(list_id,user),
            FOREIGN KEY(list_id) REFERENCES Lists(id)
        );
        CREATE TABLE IF NOT EXISTS Categories(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            icon BLOB,
            order_index INTEGER DEFAULT id
        );
        CREATE TABLE IF NOT EXISTS Products(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL, 
            category_id INTEGER,
            FOREIGN KEY(category_id) REFERENCES Categories(id)
        );
        CREATE TABLE IF NOT EXISTS ListItems(
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            checked BOOL NOT NULL DEFAULT false,
            product_id INTEGER NOT NULL,
            list_id INTEGER NOT NULL,
            added_by TEXT NOT NULL,
            added_on DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY(product_id) REFERENCES Products(id),
            FOREIGN KEY(list_id) REFERENCES Lists(id)
        );
        "
    )
    .execute(db)
    .await
    .unwrap();
}

async fn index_page(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
) -> Result<Html<String>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let path = &state.config.html_dir.join("index.html");
    let index_page_contents = fs::read_to_string(path).expect("Can not read index page file");
    let index_page_contents = index_page_contents.replace("{user}", user);
    let index_page_contents = index_page_contents.replace("{url_pfx}", &state.config.url_prefix);
    Ok(Html(index_page_contents))
}

async fn serve_html(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let full_path = state.config.html_dir.join(path + ".html");
    let html_str = fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let html_str = html_str.replace("{user}", user);
    let html_str = html_str.replace("{url_pfx}", &state.config.url_prefix);
    Ok(Html(html_str))
}

async fn fetch_list(
    headers: HeaderMap,
    State(state): State<Arc<ServerState>>,
    extract::Path(list_id): extract::Path<String>,
) -> Result<Html<String>, StatusCode> {
    list_id
        .parse::<i64>()
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let user = headers
        .get("X-Forwarded-User")
        .ok_or(StatusCode::UNAUTHORIZED)?
        .to_str()
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    let full_path = state.config.html_dir.join("list.html");
    let html_str = fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    let html_str = html_str.replace("{user}", user);
    let html_str = html_str.replace("{url_pfx}", &state.config.url_prefix);
    let html_str = html_str.replace("{list_id}", &list_id);
    Ok(Html(html_str))
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
