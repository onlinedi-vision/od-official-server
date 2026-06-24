use tokio;

pub struct ScyllaSession {
    pub lock: tokio::sync::Mutex<scylla::client::session::Session>,
}

pub struct MokaCache {
    pub lock: tokio::sync::Mutex<moka::future::Cache<String, String>>,
}
