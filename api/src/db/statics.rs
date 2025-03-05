pub static INSERT_NEW_USER: &str = r#"
    INSERT INTO division_online.users (username, email, password_hash, key, bio)
        VALUES (?,?,?,?,?);
"#;

