pub static SELECT_SERVER_INFO: &str = r"
    SELECT name, desc, img_url, fe_config FROM division_online.o_servers
        WHERE sid = ?
        ALLOW FILTERING;
";

pub static SELECT_SERVER_FE_CONFIG: &str = r"
    SELECT fe_config FROM division_online.o_servers
        WHERE sid = ?
        ALLOW FILTERING;
";

pub static UPDATE_FE_CONFIG: &str = r"
    UPDATE division_online.o_servers SET fe_config = ?
        WHERE sid = ?;
";
