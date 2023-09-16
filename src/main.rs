mod assets;
mod backend;
mod config;
mod content;
mod proxy;
mod resolver;

// include generated hash file
include!(concat!(env!("OUT_DIR"), "/hash.rs"));

use std::{env, path::Path as StdPath, sync::Arc};

use axum::{
    extract::Path,
    http::{header, HeaderName, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use axum_server::tls_rustls::RustlsConfig;
use include_dir::{include_dir, Dir};

use crate::content::get_content_type;

// the entire website files
static PIPED_SRC: Dir = include_dir!("$CARGO_MANIFEST_DIR/piped/dist");

static PIPED_JAR: &[u8] = include_bytes!("../piped-backend/build/libs/piped-1.0-all.jar");

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Arc::new(config::Config::get_config()?);

    // build patched runtime assets for the frontend
    assets::patch_assets(&config);

    // start backend, but keep it open as long as the frontend is open for
    backend::run_backend(config.clone())?;

    // frontend
    let config2 = config.clone();
    tokio::spawn(async move {
        // index.html @ /
        let app = Router::new()
            .route("/", get(get_index))
            .route("/*file", get(get_file));

        let frontend_addr = resolver::get_addresses(&config2.addresses.frontend)
            .expect("Failed to resolve frontend address");
        #[allow(clippy::if_same_then_else)]
        let frontend_addr = if config2.addresses.use_ipv6.as_ref().is_some_and(|i| *i) {
            frontend_addr.ipv6.as_ref()
        } else if frontend_addr.ipv6.as_ref().is_some() {
            frontend_addr.ipv6.as_ref()
        } else {
            frontend_addr.ipv4.as_ref()
        }
        .expect("Failed to resolve frontend address");

        if config2.addresses.use_ssl.as_ref().is_some_and(|i| *i) {
            let exe_path = env::current_exe().unwrap();
            let exe_path = exe_path.parent().unwrap();

            let config = RustlsConfig::from_pem_file(
                exe_path.join(config2.addresses.ssl_cert.as_ref().unwrap()),
                exe_path.join(config2.addresses.ssl_key.as_ref().unwrap()),
            )
            .await
            .unwrap();

            axum_server::bind_rustls(*frontend_addr, config)
                .serve(app.into_make_service())
                .await
                .unwrap();
        } else {
            axum::Server::bind(frontend_addr)
                .serve(app.into_make_service())
                .await
                .unwrap();
        }
    });

    // proxy
    proxy::start_proxy(&config).await?;

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
        if let Some(asset) = assets::get_patched_asset(&path) {
            // these are only strings due to how they were patched, so just default ot false
            (StatusCode::OK, asset.as_bytes(), false)
        } else if let Some(file) = PIPED_SRC.get_file(&path) {
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
