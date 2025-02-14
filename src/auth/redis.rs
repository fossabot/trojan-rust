use super::Auth;
use anyhow::{Context, Result};
use async_trait::async_trait;
use log::info;
use redis::AsyncCommands;

pub struct RedisAuthenticator {
    client: redis::Client,
}

impl RedisAuthenticator {
    pub fn new(server: &str) -> Result<RedisAuthenticator> {
        let client = redis::Client::open(format!("redis://{}/", server))?;
        client.get_connection()?;
        info!("Using redis auth: {}", server);
        Ok(RedisAuthenticator { client })
    }
}

#[async_trait]
impl Auth for RedisAuthenticator {
    async fn auth(&self, password: &str) -> Result<bool> {
        let mut con = self
            .client
            .get_async_connection()
            .await
            .context("Cannot create connection to redis server.")?;
        Ok(con
            .exists::<_, bool>(password)
            .await
            .context("Executing command EXISTS failed.")?)
    }

    async fn stat(&self, password: &str, upload: u64, download: u64) -> Result<()> {
        let mut con = self
            .client
            .get_async_connection()
            .await
            .context("Cannot create connection to redis server.")?;
        Ok(redis::pipe()
            .atomic()
            .hincr(password, "upload", upload)
            .hincr(password, "download", download)
            .query_async::<_, ()>(&mut con)
            .await
            .context("Executing command MULTI failed.")?)
    }
}
