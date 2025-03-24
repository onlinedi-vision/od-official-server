pub static SELECT_USER_USERNAME: &str = r#"
    SELECT username FROM division_online.users
        WHERE username = ?;
"#;

pub static CHECK_TOKEN: &str = r#"
    SELECT key FROM division_online.users 
        WHERE key = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_CHANNELS: &str = r#"
    SELECT channel_name FROM division_online.o_server_channels
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static INSERT_NEW_USER: &str = r#"
    INSERT INTO division_online.users (username, password_hash, email, key, bio)
        VALUES (?,?,?,?,?);
"#;

pub static UPDATE_USER_KEY: &str = r#"
    UPDATE division_online.users SET key=?
        WHERE username = ?;
"#;

pub static SELECT_USER_PASSWORD_HASH: &str = r#"
    SELECT password_hash FROM division_online.users
        WHERE username = ?;
"#;

pub static INSERT_NEW_SERVER_CHANNEL_MESSAGE: &str = r#"
    INSERT INTO division_online.o_server_messages(mid,channel_name,datetime,m_content,sid,username) 
        VALUES(?,?,dateof(now()),?,?,?); 
"#;

pub static SELECT_SERVER_CHANNEL_MESSAGES: &str = r#"
    SELECT username, datetime, m_content FROM division_online.o_server_messages 
        WHERE sid=? AND channel_name=? 
        ALLOW FILTERING; 
"#;
