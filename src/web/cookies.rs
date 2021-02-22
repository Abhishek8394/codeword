pub const SESSION_ID: String = "SESSION_ID".to_string();

/// Cookie gen util. By default set path to "/", "secure" to true if in prod.
pub fn gen_cookie(key: &str, value: &str, max_age: u32, path: &str, secure: bool) -> String {
    format!("{}={}; Max-age={}; Path={}; {}", key, value, max_age, path, if secure {"secure" } else {""})
}

/// Gen auth_cookie.
pub fn gen_auth_cookie(sess_id: &str) -> String {
    gen_cookie("SESSION_ID", &sess_id, 3 * 3600, "/", true)
}

