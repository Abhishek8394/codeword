pub const SESSION_ID: &str = "SESSION_ID";

#[derive(Debug)]
pub enum SameSitePolicy {
    None,
    Lax,
    Strict,
}

impl SameSitePolicy {
    pub fn to_string(&self) -> String {
        match &self {
            Self::None => "None".to_string(),
            Self::Lax => "Lax".to_string(),
            Self::Strict => "Strict".to_string(),
        }
    }
}

/// Cookie gen util. By default set path to "/", "secure" to true if in prod.
pub fn gen_cookie(
    key: &str,
    value: &str,
    max_age: u32,
    path: &str,
    secure: bool,
    same_site: SameSitePolicy,
) -> String {
    format!(
        "{}={};Max-age={};path={};SameSite={};{}",
        key,
        value,
        max_age,
        path,
        same_site.to_string(),
        if secure { "secure" } else { "" }
    )
    // what about HttpOnly
}

/// Gen auth_cookie.
pub fn gen_auth_cookie(sess_id: &str, secure: bool, path: Option<String>) -> String {
    let path = match path {
        None => "/".to_string(),
        Some(p) => p,
    };
    gen_cookie(
        "SESSION_ID",
        &sess_id,
        3 * 3600,
        &path,
        secure,
        SameSitePolicy::Strict,
    )
}
