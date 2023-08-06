use std::{env, fs};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub addresses: Addresses,
    pub backend: Backend,
}

impl Config {
    pub fn get_config() -> anyhow::Result<Self> {
        let current_folder = env::current_exe()?;
        let config_path = current_folder
            .parent()
            .ok_or(anyhow!("failed to get path"))?
            .join("config.toml");

        let config = if let Ok(data) = fs::read_to_string(&config_path) {
            toml::from_str::<Self>(&data)?
        } else {
            let cfg = Self::default();
            let mut data = toml::to_string(&cfg)?;
            data.insert_str(0, "# For more options, please see `config.rs` and/or\n# https://github.com/TeamPiped/Piped-Backend/blob/master/config.properties\n\n");
            fs::write(config_path, data)?;
            cfg
        };

        // set env vars for backend
        env::set_var("DISABLE_SERVER", config.backend.disable_server.to_string());
        let port = config
            .addresses
            .backend
            .split(':')
            .last()
            .map(|l| l.parse::<u16>().unwrap_or(80))
            .ok_or(anyhow!("http part missing"))?;
        env::set_var("PORT", port.to_string());
        env::set_var("HTTP_WORKERS", config.backend.http_workers.to_string());
        env::set_var("PROXY_PART", &config.addresses.proxy);
        if let Some(image_proxy) = &config.backend.image_proxy_part {
            env::set_var("IMAGE_PROXY_PART", image_proxy);
        }
        if let Some(base_url) = &config.backend.captcha_base_url {
            env::set_var("CAPTCHA_BASE_URL", base_url);
        }
        if let Some(api_key) = &config.backend.captcha_api_key {
            env::set_var("CAPTCHA_API_KEY", api_key);
        }
        env::set_var("API_URL", &config.addresses.backend);
        env::set_var("FRONTEND_URL", &config.addresses.frontend);
        if let Some(url) = &config.backend.pubsub_url {
            env::set_var("PUBSUB_URL", url);
        }
        if let Some(url) = &config.backend.pubsub_hub_url {
            env::set_var("PUBSUB_HUB_URL", url);
        }
        if let Some(url) = &config.backend.reqwest_proxy {
            env::set_var("REQWEST_PROXY", url);
        }
        env::set_var(
            "COMPROMISED_PASSWORD_CHECK",
            config.backend.compromised_password_check.to_string(),
        );
        env::set_var(
            "DISABLE_REGISTRATION",
            config.backend.disable_registration.to_string(),
        );
        env::set_var("FEED_RETENTION", config.backend.feed_retention.to_string());
        if let Some(state) = config.backend.disable_timers {
            env::set_var("DISABLE_TIMERS", state.to_string());
        }
        if let Some(url) = &config.backend.ryd_proxy_url {
            env::set_var("RYD_PROXY_URL", url);
        }
        if let Some(servers) = &config.backend.sponsorblock_servers {
            env::set_var("SPONSORBLOCK_SERVERS", servers);
        }
        if let Some(state) = config.backend.disable_ryd {
            env::set_var("DISABLE_RYD", state.to_string());
        }
        if let Some(state) = config.backend.disable_lbry {
            env::set_var("DISABLE_LBRY", state.to_string());
        }
        if let Some(expiry) = config.backend.subscriptions_expiry {
            env::set_var("SUBSCRIPTIONS_EXPIRY", expiry.to_string());
        }
        if let Some(dsn) = &config.backend.sentry_dsn {
            env::set_var("SENTRY_DSN", dsn);
        }
        if let Some(endpoint) = &config.backend.s3_endpoint {
            env::set_var("S3_ENDPOINT", endpoint);
        }
        if let Some(access_key) = &config.backend.s3_access_key {
            env::set_var("S3_ACCESS_KEY", access_key);
        }
        if let Some(secret_key) = &config.backend.s3_secret_key {
            env::set_var("S3_SECRET_KEY", secret_key);
        }
        if let Some(bucket) = &config.backend.s3_bucket {
            env::set_var("S3_BUCKET", bucket);
        }
        env::set_var(
            "hibernate.connection.url",
            &config.backend.db_connection_url,
        );
        env::set_var("hibernate.connection.username", &config.backend.db_username);
        env::set_var("hibernate.connection.password", &config.backend.db_password);
        env::set_var("hibernate.connection.driver_class", "org.postgresql.Driver");
        env::set_var(
            "hibernate.dialect",
            "org.hibernate.dialect.PostgreSQLDialect",
        );

        Ok(config)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Backend {
    pub disable: bool,
    // Disable API server (node just runs timers if enabled)
    pub disable_server: bool,
    // The number of workers to use for the server
    pub http_workers: u32,
    // Captcha Parameters
    pub captcha_base_url: Option<String>,
    pub captcha_api_key: Option<String>,
    // Enable haveibeenpwned compromised password API
    pub compromised_password_check: bool,
    pub image_proxy_part: Option<String>,
    pub pubsub_url: Option<String>,
    pub pubsub_hub_url: Option<String>,
    // Outgoing proxy to be used by reqwest4j - eg: socks5://127.0.0.1:1080
    pub reqwest_proxy: Option<String>,
    // RYD Proxy URL (see https://github.com/TeamPiped/RYD-Proxy)
    pub ryd_proxy_url: Option<String>,
    // SponsorBlock Servers(s)
    // Comma separated list of SponsorBlock Servers to use
    pub sponsorblock_servers: Option<String>,
    // Geo Restriction Checker for federated bypassing of Geo Restrictions
    pub geo_restriction_checker_url: Option<String>,
    // Disable Registration
    pub disable_registration: bool,
    pub disable_timers: Option<bool>,
    // Disable the usage of RYD
    pub disable_ryd: Option<bool>,
    // Disable the inclusion of LBRY streams
    pub disable_lbry: Option<bool>,
    // How long should unauthenticated subscriptions last for
    pub subscriptions_expiry: Option<u32>,
    // Sentry DSN
    // Use Sentry to log errors and trace performance
    pub sentry_dsn: Option<String>,
    // S3 Configuration Data (compatible with any provider that offers an S3 compatible API)
    pub s3_endpoint: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub s3_bucket: Option<String>,
    // Matrix Client Server URL
    pub matrix_server: Option<String>,
    // Matrix Access Token
    // If not present, will work in anon mode
    pub matrix_token: Option<String>,
    // Feed Retention Time in Days
    pub feed_retention: u32,
    // database connection settings
    pub db_connection_url: String,
    pub db_username: String,
    pub db_password: String,
}

impl Default for Backend {
    fn default() -> Self {
        Self {
            disable: false,
            disable_server: false,
            http_workers: 2,
            captcha_base_url: None,
            captcha_api_key: None,
            compromised_password_check: true,
            image_proxy_part: None,
            pubsub_url: None,
            pubsub_hub_url: None,
            reqwest_proxy: None,
            ryd_proxy_url: None,
            sponsorblock_servers: None,
            geo_restriction_checker_url: None,
            disable_registration: false,
            disable_timers: Some(false),
            disable_ryd: Some(false),
            disable_lbry: Some(false),
            subscriptions_expiry: Some(30),
            sentry_dsn: Default::default(),
            s3_endpoint: Default::default(),
            s3_access_key: Default::default(),
            s3_secret_key: Default::default(),
            s3_bucket: Default::default(),
            matrix_server: Default::default(),
            matrix_token: Default::default(),
            feed_retention: 30,
            db_connection_url: "jdbc:postgresql://localhost:5432/piped".to_string(),
            db_username: "piped".to_string(),
            db_password: "piped".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addresses {
    // Frontend address (MUST contain http/https prefix, with no ending /)
    // Can contain host addresses as well
    //- eg: http://127.0.0.1:8080 , http://myaddr.com:8080
    pub frontend: String,
    // Backend address (MUST contain http/https prefix, with no ending /)
    // Can contain host addresses as well
    //- eg: http://127.0.0.1:8081 , http://myaddr.com:8081
    pub backend: String,
    // Proxy (MUST contain http/https prefix, with no ending /)
    // Can contain host addresses as well
    //- eg: http://127.0.0.1:8082 , http://myaddr.com:8082
    pub proxy: String,
    // Use ipv6 address (will error if non exists). By default it sticks to ipv4
    pub use_ipv6: Option<bool>,
}

impl Default for Addresses {
    fn default() -> Self {
        Self {
            use_ipv6: None,
            frontend: "http://localhost:8080".to_string(),
            backend: "http://localhost:8081".to_string(),
            proxy: "http://localhost:8082".to_string(),
        }
    }
}
