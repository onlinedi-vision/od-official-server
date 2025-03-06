pub struct ScyllaSession {
    pub lock: std::sync::Mutex<scylla::client::session::Session>
}
