/// Structure used for inter-opping with a given scylla instance.
pub struct ScyllaSession {
    pub session: scylla::client::session::Session,
}

/// Structure used for inter-opping with the API's running cache.
///
/// # About API's caching
/// We currently use caching for rapid access to user data that is commonly
/// accessed during normal runtime. Such data can be for example: tokens.
pub struct MokaCache {
    pub cache: moka::future::Cache<String, String>,
}
