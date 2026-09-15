use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    host: IpAddr,
    port: u16,
    pub(crate) location: PathBuf,
}

impl Config {
    pub(crate) fn address(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
            location: PathBuf::from("/tmp/storage"),
        }
    }
}
