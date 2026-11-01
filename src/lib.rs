use crate::config::Config;
use axum::{
    extract::{self, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::{delete, get, post, put},
};
use sqlx::{Sqlite, sqlite::SqliteConnectOptions};
use std::{fs, str::FromStr, sync::Arc};

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
    let db_conn = sqlx::Pool::connect_with(
        SqliteConnectOptions::from_str(&db_url)
            .unwrap()
            .create_if_missing(true),
    )
    .await
    .expect("Could not connect to database");

    sqlx::migrate!().run(&db_conn).await.unwrap();

    let url_pfx = config.url_prefix.clone();
    let server_state = Arc::new(ServerState { config, db_conn });
    let server = axum::Router::new()
        .route(&(url_pfx.clone() + "/"), get(index_page))
        .route(&(url_pfx.clone() + "/static/css/{path}"), get(serve_css))
        .route(&(url_pfx.clone() + "/static/js/{path}"), get(serve_js))
        .route(
            &(url_pfx.clone() + "/static/icons/{path}"),
            get(serve_icons),
        )
        .route(
            &(url_pfx.clone() + "/api/lists"),
            get(api::lists::get_lists),
        )
        .route(
            &(url_pfx.clone() + "/api/lists"),
            post(api::lists::create_list),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}"),
            get(api::lists::get_list),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}"),
            put(api::lists::update_list),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}"),
            delete(api::lists::delete_list),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/items"),
            get(api::list_items::get_list_items),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/items"),
            post(api::list_items::create_list_item),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/items/{list_item_id}"),
            get(api::list_items::get_list_item),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/items/{list_item_id}"),
            put(api::list_items::update_list_item),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/items/{list_item_id}"),
            delete(api::list_items::delete_list_item),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/products"),
            get(api::products::query_products),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/products/{product_id}"),
            get(api::products::get_product),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/products/{product_id}"),
            put(api::products::update_product),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/products/{product_id}"),
            delete(api::products::delete_product),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/categories"),
            get(api::categories::get_categories),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/categories/{category_id}"),
            get(api::categories::get_category),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/categories"),
            post(api::categories::create_category),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/categories/{category_id}"),
            put(api::categories::update_category),
        )
        .route(
            &(url_pfx.clone() + "/api/lists/{list_id}/categories/{category_id}"),
            delete(api::categories::delete_category),
        )
        .route(&(url_pfx.clone() + "/{path}"), get(serve_html))
        .with_state(server_state);

    let listener = tokio::net::TcpListener::bind(host_socket)
        .await
        .expect("Failed to bind to socket");
    println!("Starting server...");
    axum::serve(listener, server).await.unwrap();
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

async fn serve_icons(
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Svg, StatusCode> {
    let full_path = state.config.icons_dir.join(path + ".svg");
    let svg_str = fs::read_to_string(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Svg(svg_str))
}

struct Svg(String);

impl IntoResponse for Svg {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header("Content-Type", "image/svg+xml")
            .body(self.0.into())
            .unwrap()
    }
}
