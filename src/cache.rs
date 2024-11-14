use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::task;
use tokio::time::{sleep, Duration};

pub struct Config {
    pub enabled: bool,
    pub ttl: Duration,
}

pub struct Cache<K, V> {
    enabled: bool,
    inner: RwLock<HashMap<K, Arc<V>>>,
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where
    K: Clone + Eq + Hash + Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    pub fn new(config: Config) -> Arc<Self> {
        Arc::new(Self {
            enabled: config.enabled,
            ttl: config.ttl,
            inner: Default::default(),
        })
    }

    pub async fn insert(self: Arc<Self>, key: K, value: V) -> Arc<V> {
        let arcd = Arc::new(value);
        if !self.enabled {
            return arcd;
        }

        self.inner
            .write()
            .await
            .insert(key.clone(), Arc::clone(&arcd));

        let self_clone = Arc::clone(&self);
        task::spawn(async move {
            sleep(self_clone.ttl).await;
            self_clone.inner.write().await.remove(&key);
        });

        arcd
    }

    pub async fn get(&self, key: &K) -> Option<Arc<V>> {
        if !self.enabled {
            return None;
        }

        return self.inner.read().await.get(key).map(Arc::clone);
    }
}
