use crate::config::Config;
use axum::{
    extract::{self, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use memchr::memmem;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use sqlx::{Sqlite, sqlite::SqliteConnectOptions};
use std::{
    fs, io,
    path::{Path, PathBuf},
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use tower_http::services::ServeDir;
use walkdir::WalkDir;

mod api;
pub mod config;

pub type OpentonicError = Box<dyn std::error::Error>;

pub struct List;

struct ServerState {
    pub config: Config,
    pub db_conn: sqlx::Pool<Sqlite>,
}

#[tokio::main]
pub async fn run_server(mut config: Config) {
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
    let html_dir = config.html_dir.clone();

    copy_dir_all(&html_dir, &html_dir.parent().unwrap().join(".run")).unwrap();

    let html_dir = html_dir.parent().unwrap().join(".run");
    config.html_dir = html_dir.clone();

    let html_dir_clone = html_dir.clone();
    ctrlc::set_handler(move || {
        fs::remove_dir_all(html_dir_clone.clone()).unwrap();
        std::process::exit(0);
    })
    .unwrap();

    replace_in_dir(&html_dir, "/{url_pfx}/", &url_pfx);
    replace_in_dir(&html_dir, "%7Burl_pfx%7D", &url_pfx);

    let server_state = Arc::new(ServerState { config, db_conn });

    let mut server = axum::Router::new()
        .route(&(url_pfx.clone() + "static/icons/{path}"), get(serve_icons))
        .route(&(url_pfx.clone() + "api/lists"), get(api::lists::get_lists))
        .route(
            &(url_pfx.clone() + "api/lists"),
            post(api::lists::create_list),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}"),
            get(api::lists::get_list),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}"),
            put(api::lists::update_list),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}"),
            delete(api::lists::delete_list),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/items"),
            get(api::list_items::get_list_items),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/items"),
            post(api::list_items::create_list_item),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/items/{list_item_id}"),
            get(api::list_items::get_list_item),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/items/{list_item_id}"),
            put(api::list_items::update_list_item),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/items/{list_item_id}"),
            delete(api::list_items::delete_list_item),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/products"),
            get(api::products::query_products),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/products/{product_id}"),
            get(api::products::get_product),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/products/{product_id}"),
            put(api::products::update_product),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/products/{product_id}"),
            delete(api::products::delete_product),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/categories"),
            get(api::categories::get_categories),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/categories/{category_id}"),
            get(api::categories::get_category),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/categories"),
            post(api::categories::create_category),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/categories/{category_id}"),
            put(api::categories::update_category),
        )
        .route(
            &(url_pfx.clone() + "api/lists/{list_id}/categories/{category_id}"),
            delete(api::categories::delete_category),
        )
        .fallback_service(ServeDir::new(&html_dir))
        .with_state(server_state);

    if !url_pfx.is_empty() {
        server = server.nest_service(&url_pfx, ServeDir::new(&html_dir));
    }

    let listener = tokio::net::TcpListener::bind(host_socket)
        .await
        .expect("Failed to bind to socket");
    println!("Starting server...");
    axum::serve(listener, server).await.unwrap();
}

fn replace_in_dir(root: &Path, from: &str, to: &str) -> usize {
    let finder = memmem::Finder::new(from.as_bytes()); // built once, shared across threads
    let replacement = to.as_bytes();
    let changed = AtomicUsize::new(0);

    let files: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| e.file_name() != ".git") // never touch git internals
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file()) // skips directories and symlinks
        .map(|e| e.into_path())
        .collect();

    files.par_iter().for_each(|path| {
        let result = (|| -> io::Result<()> {
            let data = fs::read(path)?;

            let mut matches = finder.find_iter(&data).peekable();
            if matches.peek().is_none() {
                return Ok(()); // nothing to do: don't rewrite the file
            }

            let mut out = Vec::with_capacity(data.len() + replacement.len());
            let mut last = 0;
            for pos in matches {
                out.extend_from_slice(&data[last..pos]);
                out.extend_from_slice(replacement);
                last = pos + from.len();
            }
            out.extend_from_slice(&data[last..]);

            fs::write(path, out)?;
            changed.fetch_add(1, Ordering::Relaxed);
            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("skipping {}: {e}", path.display());
        }
    });

    changed.into_inner()
}

fn copy_dir_all(src: &Path, dst: &Path) -> io::Result<()> {
    if dst.parent().unwrap() == src {
        return Ok(());
    };
    let iter = WalkDir::new(src);
    for entry in iter {
        let entry = entry?; // walkdir::Error converts into io::Error
        let rel = entry.path().strip_prefix(src).expect("entry is under src");
        let target = dst.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            // the parent should exist already, since the walk yields directories first
            fs::copy(entry.path(), &target).unwrap();
        }
    }
    Ok(())
}

async fn serve_icons(
    State(state): State<Arc<ServerState>>,
    extract::Path(path): extract::Path<String>,
) -> Result<Svg, StatusCode> {
    let full_path = state.config.icons_dir.join(path + ".svg");
    let svg = fs::read(full_path).map_err(|_| StatusCode::NOT_FOUND)?;
    Ok(Svg(svg))
}

struct Svg(Vec<u8>);

impl IntoResponse for Svg {
    fn into_response(self) -> axum::response::Response {
        Response::builder()
            .header("Content-Type", "image/svg+xml")
            .body(self.0.into())
            .unwrap()
    }
}
