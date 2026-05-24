use std::env;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl ApiConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: env::var("API_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: env::var("API_PORT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(8080),
            database_url: env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgres://netchronicle:netchronicle@localhost:5432/netchronicle".into()
            }),
        })
    }

    pub fn socket_addr(&self) -> SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("valid socket address")
    }
}
