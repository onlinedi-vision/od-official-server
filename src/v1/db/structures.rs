use bitflags::bitflags;

/*
This provides some quality of life overloaded operators which makes the development process much easier. 
More about it at: https://docs.rs/flags/latest/bitflags/ 
*/
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Permissions: i64 {
        const SEND_MESSAGES = 1 << 0;
        const ADD_ROLE =      1 << 1;
        const CHANGE_FE =     1 << 2;
    }
}

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
