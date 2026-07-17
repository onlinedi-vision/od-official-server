pub struct ScyllaSession {
    pub session: scylla::client::session::Session,
}

pub struct MokaCache {
    pub cache: moka::future::Cache<String, String>,
}
