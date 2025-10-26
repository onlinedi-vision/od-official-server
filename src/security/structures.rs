pub struct ScyllaSession {
    pub lock: std::sync::Mutex<scylla::client::session::Session>
}

pub struct MokaCache {
    pub lock: std::sync::Mutex<moka::future::Cache<String, String>>
}
