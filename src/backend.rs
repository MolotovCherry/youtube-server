use std::{
    env, fs,
    process::Command,
    sync::{Arc, OnceLock},
};

use anyhow::anyhow;
use axum::{
    body::{Body, BoxBody, HttpBody},
    http::{header, Request},
    response::{IntoResponse, Response},
};
use axum_server::tls_rustls::RustlsConfig;
use directories::ProjectDirs;
use reqwest::{redirect::Policy, Client, StatusCode};
use tokio::task::{self, JoinHandle};
use tower::make::Shared;

use crate::{
    config::Config,
    error_page::{format_error_page, E502},
    hash::JAR_HASH,
    resolver,
};

static REQUEST_DATA: OnceLock<RequestData> = OnceLock::new();

#[derive(Debug)]
struct RequestData {
    client: Client,
    config: Arc<Config>,
}

pub fn run_backend(config: Arc<Config>) -> anyhow::Result<JoinHandle<anyhow::Result<()>>> {
    let handle = task::spawn(async move {
        // it would've been great to use graalvm for this and compile to a shared lib,
        // which I spent an entire day on, but it turns out there's just too many resource configs,
        // proxy configs, etc etc etc, it's too much and way too hard to get working.
        // at least the following works, you know?

        let project_dir =
            ProjectDirs::from("", "", "youtube-server").expect("Failed to get project directory");

        let data_local = project_dir.data_local_dir();

        if !data_local.exists() {
            fs::create_dir_all(data_local).expect("Failed to create project dir");
        }

        let jar_path = data_local.join(format!("piped-{JAR_HASH}.jar"));

        if !jar_path.exists() {
            fs::write(&jar_path, crate::PIPED_JAR).expect("Failed to write jar");
        }

        let jar_path = jar_path.to_str().ok_or(anyhow!("failed to make string"))?;

        // java needs to be on PATH
        let mut child = Command::new("java")
            .args([
                "-server",
                "-Xmx1G",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+HeapDumpOnOutOfMemoryError",
                "-XX:+OptimizeStringConcat",
                "-XX:+UseStringDeduplication",
                "-XX:+UseCompressedOops",
                "-XX:+UseNUMA",
                "-XX:+UseG1GC",
                "-jar",
                jar_path,
            ])
            .current_dir(data_local)
            .spawn()?;

        if config.addresses.use_ssl.as_ref().is_some_and(|i| *i) {
            REQUEST_DATA
                .set(RequestData {
                    // important, disable all redirects so we can be as transparent as possible
                    client: Client::builder().redirect(Policy::none()).build().unwrap(),
                    config: config.clone(),
                })
                .unwrap();

            // make backend address
            let backend_addr =
                resolver::get_addresses(config.addresses.backend_ssl_proxy.as_ref().unwrap())
                    .expect("Failed to resolve frontend address");
            #[allow(clippy::if_same_then_else)]
            let backend_addr = if config.addresses.use_ipv6.as_ref().is_some_and(|i| *i) {
                backend_addr.ipv6.as_ref()
            } else if matches!(backend_addr.ipv6.as_ref(), Some(_)) {
                backend_addr.ipv6.as_ref()
            } else {
                backend_addr.ipv4.as_ref()
            }
            .expect("Failed to resolve frontend address");

            // get server config for rust
            let exe_path = env::current_exe()?;
            let exe_path = exe_path.parent().ok_or(anyhow!("Failed to get parent"))?;

            let config = RustlsConfig::from_pem_file(
                exe_path.join(
                    config
                        .addresses
                        .ssl_cert
                        .as_ref()
                        .ok_or(anyhow!("ssl_cert missing"))?,
                ),
                exe_path.join(
                    config
                        .addresses
                        .ssl_key
                        .as_ref()
                        .ok_or(anyhow!("ssl_key missing"))?,
                ),
            )
            .await?;
            //

            let service = tower::service_fn(backend_ssl_proxy);

            axum_server::bind_rustls(*backend_addr, config)
                .serve(Shared::new(service))
                .await?;
        } else {
            task::spawn_blocking(move || {
                child.wait()?;
                Ok::<_, anyhow::Error>(())
            })
            .await??;
        }

        Ok(())
    });

    Ok(handle)
}

async fn backend_ssl_proxy(req: Request<Body>) -> anyhow::Result<Response<BoxBody>> {
    let data = REQUEST_DATA.get().unwrap();

    let (parts, body) = req.into_parts();

    let path = parts
        .uri
        .path_and_query()
        .map(|i| i.as_str())
        .unwrap_or("/");
    let method = parts.method;
    let headers = parts.headers;

    let url = format!("{}{path}", data.config.addresses.backend_uri());

    let reqwest = match data
        .client
        .request(method, url)
        .headers(headers)
        .body(body)
        .send()
        .await
    {
        Ok(res) => res,
        Err(e) => {
            return Ok((
                StatusCode::BAD_GATEWAY,
                [(header::CONTENT_TYPE, "text/html")],
                format_error_page(E502, e),
            )
                .into_response());
        }
    };

    let mut response = Response::builder();

    *response.headers_mut().unwrap() = reqwest.headers().clone();

    let response = response
        .status(reqwest.status())
        .body(Body::wrap_stream(reqwest.bytes_stream()))
        .unwrap()
        .map(|b| BoxBody::new(b.map_err(axum::Error::new)));

    Ok(response)
}
