#[derive(serde::Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub desc: String,
    pub img_url: String,
    pub fe_config: String,
}

#[derive(serde::Serialize)]
pub struct ServerFeConfig {
    pub fe_config: String,
}
