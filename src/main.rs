mod cli;
mod content;

use std::net::IpAddr;
use std::path::Path as StdPath;

use axum::{
    extract::Path,
    http::{header, HeaderName, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use include_dir::{include_dir, Dir};

use cli::Cli;

use crate::content::get_content_type;

// the entire website files
static PIPED_SRC: Dir = include_dir!("$CARGO_MANIFEST_DIR/piped/dist");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let ip = match cli.ip {
        IpAddr::V4(ip) => ip.to_string(),
        // must be in brackets for the port to work
        IpAddr::V6(ip) => format!("[{}]", ip),
    };

    let port = cli.port;

    // index.html @ /
    let app = Router::new()
        .route("/", get(get_index))
        .route("/*file", get(get_file));

    axum::Server::bind(&format!("{ip}:{port}").parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

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
