pub static SELECT_USER_USERNAME: &str = r#"
    SELECT username FROM division_online.users
        WHERE username = ?;
"#;

pub static CHECK_TOKEN: &str = r#"
    SELECT key FROM division_online.users 
        WHER key = ?;
"#;

pub static SELECT_SERVER_CHANNELS: &str = r#"
    SELECT channel_name WHERE sid = ?;
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
