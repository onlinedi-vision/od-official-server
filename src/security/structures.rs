use tokio;

/// Structure used for inter-opping with a given scylla instance.
pub struct ScyllaSession {
    pub lock: tokio::sync::Mutex<scylla::client::session::Session>,
}

/// Structure used for inter-opping with the API's running cache.
///
/// # About API's caching
/// We currently use caching for rapid access to user data that is commonly
/// accessed during normal runtime. Such data can be for example: tokens.
pub struct MokaCache {
    pub lock: tokio::sync::Mutex<moka::future::Cache<String, String>>,
}
