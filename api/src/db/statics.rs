pub static INSERT_NEW_USER: &str = r#"
    INSERT INTO division_online.users (username, password_hash, email, key, bio)
        VALUES (?,?,?,?,?);
"#;

pub static UPDATE_USER_KEY: &str = r#"
    UPDATE division_online.users SET key=?
        WHERE username = ?;
"#;

pub static SELECT_USER_PASSWORD_HASH: &str = r#"
    SELECT password_hash FORM division_online.users
        WHERE username = ?;
"#;
