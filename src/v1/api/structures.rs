#[derive(serde::Deserialize)]
pub struct UpdateServerFrontend {
    pub config: String, 
    pub token: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct Status {
    pub status: String,
}
