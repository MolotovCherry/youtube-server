mod backend;
mod config;
mod content;
mod proxy;
mod resolver;

// include generated hash file
include!(concat!(env!("OUT_DIR"), "/hash.rs"));

use std::path::Path as StdPath;

use axum::{
    extract::Path,
    http::{header, HeaderName, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use include_dir::{include_dir, Dir};

use crate::content::get_content_type;

// the entire website files
static PIPED_SRC: Dir = include_dir!("$CARGO_MANIFEST_DIR/piped/dist");

static PIPED_JAR: &[u8] = include_bytes!("../piped-backend/build/libs/piped-1.0-all.jar");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = config::Config::get_config()?;

    // start backend, but keep it open as long as the frontend is open for
    backend::run_backend()?;

    // frontend
    tokio::spawn(async move {
        // index.html @ /
        let app = Router::new()
            .route("/", get(get_index))
            .route("/*file", get(get_file));

        let frontend_addr = resolver::get_addresses(&config.addresses.frontend)
            .expect("Failed to resolve frontend address");
        #[allow(clippy::if_same_then_else)]
        let frontend_addr = if config.addresses.use_ipv6.as_ref().is_some_and(|i| *i) {
            frontend_addr.ipv6.as_ref()
        } else if matches!(frontend_addr.ipv6.as_ref(), Some(_)) {
            frontend_addr.ipv6.as_ref()
        } else {
            frontend_addr.ipv4.as_ref()
        }
        .expect("Failed to resolve frontend address");

        axum::Server::bind(frontend_addr)
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let proxy_addr =
        resolver::get_addresses(&config.addresses.proxy).expect("Failed to resolve proxy address");
    #[allow(clippy::if_same_then_else)]
    let proxy_addr = if config.addresses.use_ipv6.as_ref().is_some_and(|i| *i) {
        proxy_addr.ipv6.as_ref()
    } else if matches!(proxy_addr.ipv6.as_ref(), Some(_)) {
        proxy_addr.ipv6.as_ref()
    } else {
        proxy_addr.ipv4.as_ref()
    }
    .expect("Failed to resolve frontend address");

    // proxy
    proxy::start_proxy(proxy_addr).await?;

    Ok(())
}

fn _get_index_internal() -> (StatusCode, [(HeaderName, &'static str); 1], &'static [u8]) {
    let (status, content, content_type) = {
        if let Some(file) = PIPED_SRC.get_file("index.html") {
            let is_binary = file.contents_utf8().is_none();

            if is_binary {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "500 Internal Server Error: Your index.html is binary .. What?".as_bytes(),
                    "text/plain",
                )
            } else {
                (StatusCode::OK, file.contents(), "text/html")
            }
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "500 Internal Server Error: Your dist directory doesn't have an index.html. Please fix the dist dir and rebuild".as_bytes(),
                "text/plain",
            )
        }
    };

    (status, [(header::CONTENT_TYPE, content_type)], content)
}

async fn get_index() -> impl IntoResponse {
    _get_index_internal()
}

async fn get_file(Path(path): Path<String>) -> impl IntoResponse {
    let (status, content, is_binary) = {
        if let Some(file) = PIPED_SRC.get_file(&path) {
            (
                StatusCode::OK,
                file.contents(),
                file.contents_utf8().is_none(),
            )
        } else {
            // It's best to just return the index page if not found and let everything else be handled
            return _get_index_internal();
        }
    };

    let content_type = if let Some(ext) = StdPath::new(&path).extension() {
        get_content_type(ext.to_str().unwrap_or_default()).unwrap_or({
            if is_binary {
                "application/octet-stream"
            } else {
                "text/plain"
            }
        })
    } else if is_binary {
        "application/octet-stream"
    } else {
        "text/plain"
    };

    (status, [(header::CONTENT_TYPE, content_type)], content)
}
