use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Config {
    host: IpAddr,
    port: u16,
    pub server_dir: PathBuf,
    pub client_dir: PathBuf,
}

impl Config {
    pub fn address(&self) -> SocketAddr {
        SocketAddr::new(self.host, self.port)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            port: 3000,
            server_dir: PathBuf::from("/tmp/storage"),
            client_dir: PathBuf::from("/Users/david/Sync/data_seminar/task3"),
        }
    }
}
