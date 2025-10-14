#![allow(unused_variables)]
#![allow(dead_code)]

pub static SELECT_USER_USERNAME: &str = r#"
    SELECT username FROM division_online.users
        WHERE username = ?;
"#;

pub static CHECK_TOKEN: &str = r#"
    SELECT key FROM division_online.o_user_tokens
        WHERE key = ?
        ALLOW FILTERING;
"#;

pub static CHECK_TOKEN_USER: &str = r#"
    SELECT key FROM division_online.o_user_tokens
        WHERE key = ? AND username = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_CHANNELS: &str = r#"
    SELECT channel_name FROM division_online.o_server_channels
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static INSERT_NEW_USER: &str = r#"
    INSERT INTO division_online.users (username, password_hash, email, key, bio, user_salt, password_salt, pfp)
        VALUES (?,?,?,?,?,?,?,'');
"#;

pub static INSERT_NEW_TOKEN: &str = r#"
    INSERT INTO division_online.o_user_tokens (username, key, datetime)
        VALUES (?,?,dateof(now()));
"#;

pub static UPDATE_USER_KEY: &str = r#"
    UPDATE division_online.users SET key=?
        WHERE username = ?;
"#;

pub static SELECT_USER_PASSWORD_HASH: &str = r#"
    SELECT password_hash, user_salt, password_salt FROM division_online.users
        WHERE username = ?
        ALLOW FILTERING;
"#;

pub static INSERT_SERVER_CHANNEL_MESSAGE: &str = r#"
    INSERT INTO division_online.o_server_messages_migration(mid,channel_name,datetime,m_content,sid,username) 
        VALUES(?,?,dateof(now()),?,?,?); 
"#;

pub static SELECT_SERVER_CHANNEL_MESSAGES_MIGRATION: &str = r#"
    SELECT username, datetime, m_content FROM division_online.o_server_messages_migration
        WHERE sid=? AND channel_name=?
        ORDER BY datetime DESC
        LIMIT ?
        ALLOW FILTERING; 
"#;

pub static SELECT_SERVER_CHANNEL_MESSAGES: &str = r#"
    SELECT username, datetime, m_content FROM division_online.o_server_messages 
        WHERE sid=? AND channel_name=? 
        ALLOW FILTERING; 
"#;

pub static INSERT_SERVER: &str = r#"
    INSERT INTO division_online.o_servers(sid, desc, name, owner) 
        VALUES(?,?,?,?);
"#;

pub static SELECT_USER_SID_LIST: &str = r#"
    SELECT sid FROM division_online.o_server_users
        WHERE username = ?
        ALLOW FILTERING;
"#;

pub static INSERT_SERVER_CHANNEL: &str = r#"
    INSERT INTO division_online.o_server_channels(sid, channel_name)
        VALUES(?,?);
"#;

pub static SELECT_SERVER_USER: &str = r#"
    SELECT username FROM division_online.o_server_users
        WHERE sid = ? AND username = ?
        ALLOW FILTERING;
"#;

pub static INSERT_NEW_SERVER: &str = r#"
    INSERT INTO division_online.o_servers(sid, desc, img_url, name, owner)
        VALUES(?,?,?,?,?);
"#;

pub static INSERT_USER_INTO_SERVER: &str = r#"
    INSERT INTO division_online.o_server_users(sid, username)
        VALUES(?,?);
"#;

pub static SELECT_SERVER_USERS: &str = r#"
    SELECT username FROM division_online.o_server_users
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static SELECT_USER_INFO: &str = r#"
    SELECT pfp, bio FROM division_online.users
        WHERE username = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_INFO: &str = r#"
    SELECT name, desc, img_url FROM division_online.o_servers
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_ROLES: &str = r#"
   SELECT role_name, color, permissions FROM division_online.o_server_roles
       WHERE server_id = ?; 
"#;

pub static SELECT_USER_ROLES: &str = r#"
   SELECT role_name FROM division_online.o_user_server_roles
       WHERE server_id = ? AND username = ?;
"#;

pub static INSERT_SERVER_ROLE: &str = r#"
   INSERT INTO division_online.o_server_roles (server_id, role_name, color, permissions)
       VALUES(?, ?, ?, ?); 
"#;

pub static DELETE_SERVER_ROLE: &str = r#"
    DELETE FROM division_online.o_server_roles
        WHERE server_id = ? AND role_name = ?;
"#;

pub static ASSIGN_ROLE_TO_USER: &str = r#"
   INSERT INTO division_online.o_user_server_roles (server_id, username, role_name)
       VALUES (?, ?, ?); 
"#;

pub static REMOVE_ROLE_FROM_USER: &str = r#"
    DELETE FROM division_online.o_user_server_roles
        WHERE server_id = ? AND username = ? AND role_name = ?;
"#;

pub static SELECT_USERS_BY_ROLE: &str = r#"
    SELECT username FROM division_online.o_user_server_roles
        WHERE server_id = ? AND role_name = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_ROLE_BY_NAME: &str = r#"
    SELECT role_name FROM division_online.o_server_roles
        WHERE server_id = ? AND role_name = ?;
"#;

pub static DELETE_SERVER_BY_SID: &str = r#"
        DELETE FROM division_online.o_servers WHERE sid = ?;
"#;

pub static DELETE_SERVER_CHANNELS_BY_SID: &str = r#"
    DELETE FROM division_online.o_server_channels WHERE sid = ?;
"#;

pub static DELETE_SERVER_USERS_BY_SID: &str = r#"
    DELETE FROM division_online.o_server_users WHERE sid = ?;
"#;

pub static DELETE_SERVER_MESSAGES_MIGRATION_BY_SID: &str = r#"
    DELETE FROM division_online.o_server_messages_migration WHERE sid = ?;
"#;

pub static DELETE_SERVER_MESSAGES_MIGRATIONS_BY_SID_AND_CHANNEL: &str = r#"
    DELETE FROM division_online.o_server_messages_migration WHERE sid = ? AND channel_name = ?;
"#;

pub static DELETE_SERVER_ROLES_BY_SID: &str = r#"
    DELETE FROM division_online.o_server_roles WHERE server_id = ?;
"#;

pub static DELETE_USER_ROLES_BY_SID: &str = r#"
    DELETE FROM division_online.o_user_server_roles WHERE server_id = ?;
"#;

pub static SELECT_SERVER_MESSAGES_BY_SID: &str = r#"
    SELECT mid FROM division_online.o_server_messages
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static SELECT_SERVER_MESSAGES_BY_SID_AND_CHANNEL: &str = r#"
    SELECT mid FROM division_online.o_server_messages
        WHERE sid = ? AND channel_name = ?
        ALLOW FILTERING;
"#;

pub static DELETE_SERVER_MESSAGES_MIGRATION: &str = r#"
    DELETE FROM division_online.o_server_messages_migration
        WHERE sid = ? AND channel_name = ? AND datetime = ?;
"#;

pub static DELETE_SERVER_MESSAGE_BY_MID: &str = r#"
    DELETE FROM division_online.o_server_messages WHERE mid = ?;
"#;

pub static DELETE_CHANNEL : &str = r#"
    DELETE FROM division_online.o_server_channels
        WHERE sid = ? AND channel_name = ?;
"#;

pub static SELECT_SERVER_OWNER: &str = r#"
    SELECT owner FROM division_online.o_servers
        WHERE sid = ?
        ALLOW FILTERING;
"#;

pub static INSERT_DM_INVITE: &str = r#"
    INSERT INTO division_online.o_dm_invites(u1, u2, invite_id, sender)
        VALUES(?,?,?,?)
        IF NOT EXISTS;     
"#;

pub static SELECT_DM_INVITE: &str = r#"
    SELECT invite_id, sender FROM division_online.o_dm_invites
        WHERE u1 = ? AND u2 = ?;
"#;

pub static DELETE_DM_INVITE: &str = r#"
    DELETE FROM division_online.o_dm_invites
        WHERE u1 = ? AND u2 = ?;
"#;

pub static SELECT_PENDING_INVITES_BY_U1: &str = r#"
    SELECT invite_id, sender FROM division_online.o_dm_invites
        WHERE u1 = ? ALLOW FILTERING;
"#;

pub static SELECT_PENDING_INVITES_BY_U2: &str = r#"
    SELECT invite_id, sender FROM division_online.o_dm_invites
        WHERE u2 = ? ALLOW FILTERING;
"#;

pub static INSERT_FRIEND: &str = r#"
    INSERT INTO division_online.o_user_friends(username, friend, created_at)
        VALUES(?,?,?);
"#;

pub static SELECT_FRIENDS: &str = r#"
    SELECT friend, created_at FROM division_online.o_user_friends
        WHERE username = ?;
"#;

pub static DELETE_FRIEND: &str = r#"
    DELETE FROM division_online.o_user_friends
        WHERE username = ? and friend = ?; 
"#;

pub static DELETE_TOKEN: &str = r#"
    DELETE FROM division_online.o_user_tokens
        WHERE username = ? and key = ?; 
"#;
