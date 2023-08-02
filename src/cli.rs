use std::net::{IpAddr, Ipv4Addr};

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// ip to listen on
    #[arg(long, default_value_t = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)))]
    pub ip: IpAddr,

    /// port to listen on
    #[arg(short, default_value_t = 8080)]
    pub port: u16,
}
