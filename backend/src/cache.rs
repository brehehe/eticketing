use redis::{Client, aio::ConnectionManager, AsyncCommands};
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

#[derive(Clone)]
pub struct Cache {
    manager: ConnectionManager,
}

impl Cache {
    pub async fn connect(url: &str) -> Result<Self> {
        let client = Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        Ok(Self { manager })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let mut conn = self.manager.clone();
        let val: Option<String> = conn.get(key).await.ok()?;
        val.and_then(|s| serde_json::from_str(&s).ok())
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T, ttl_sec: u64) -> Result<()> {
        let mut conn = self.manager.clone();
        let serialized = serde_json::to_string(value)?;
        let _: () = conn.set_ex(key, serialized, ttl_sec as usize).await?;
        Ok(())
    }

    pub async fn del(&self, key: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let _: () = conn.del(key).await?;
        Ok(())
    }

    pub async fn invalidate_prefix(&self, prefix: &str) -> Result<()> {
        let mut conn = self.manager.clone();
        let pattern = format!("{}*", prefix);
        let keys: Vec<String> = conn.keys(pattern).await?;
        if !keys.is_empty() {
            let _: () = conn.del(keys).await?;
        }
        Ok(())
    }
}
