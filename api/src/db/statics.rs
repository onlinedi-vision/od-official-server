pub static INSERT_NEW_USER: &str = r#"
    INSERT INTO division_online.users (username, password_hash, email, key, bio)
        VALUES (?,?,?,?,?);
"#;

