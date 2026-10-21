//! The `[deew_web]` table of the deew config.

use argon2::PasswordHash;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub user: String,
    /// Argon2 PHC string (validated).
    pub hash: String,
}

const HASH_HINT: &str = "generate one with `deew-web --hash-password`";

/// Parses the config text. `Ok(None)` means no login is required.
pub fn parse_credentials(text: &str) -> Result<Option<Credentials>, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|e| format!("the config is not valid TOML: {e}"))?;
    let Some(section) = table.get("deew_web") else {
        return Ok(None);
    };
    let section = section
        .as_table()
        .ok_or("[deew_web] in the config must be a table")?;
    let get = |key: &str| -> Result<String, String> {
        match section.get(key) {
            None => Ok(String::new()),
            Some(toml::Value::String(s)) => Ok(s.clone()),
            Some(_) => Err(format!("[deew_web] {key} in the config must be a string")),
        }
    };
    let user = get("user")?;
    let pass = get("pass")?;
    match (user.is_empty(), pass.is_empty()) {
        (true, true) => Ok(None),
        (false, true) => Err(format!(
            "[deew_web] user is set but pass is empty; set both or neither ({HASH_HINT})"
        )),
        (true, false) => {
            Err("[deew_web] pass is set but user is empty; set both or neither".into())
        }
        (false, false) => {
            validate_hash(&pass)?;
            Ok(Some(Credentials { user, hash: pass }))
        }
    }
}

/// Checks that `hash` is an argon2 PHC string.
pub fn validate_hash(hash: &str) -> Result<(), String> {
    let parsed = PasswordHash::new(hash).map_err(|_| {
        format!("[deew_web] pass is not a valid argon2 hash (PHC string); {HASH_HINT}")
    })?;
    let algorithm = parsed.algorithm.as_str();
    if !matches!(algorithm, "argon2id" | "argon2i" | "argon2d") {
        return Err(format!(
            "[deew_web] pass uses {algorithm}, only argon2 hashes are supported; {HASH_HINT}"
        ));
    }
    Ok(())
}

/// Reads the credentials from the config file. A missing file means no login.
pub fn load_credentials(path: Option<&Path>) -> Result<Option<Credentials>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("cannot read {}: {e}", path.display())),
    };
    parse_credentials(&text).map_err(|e| format!("{}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hash(pw: &str) -> String {
        crate::auth::hash_password(pw).unwrap()
    }

    #[test]
    fn no_table_or_empty_means_no_login() {
        assert_eq!(parse_credentials("").unwrap(), None);
        assert_eq!(parse_credentials("[other]\nx = 1").unwrap(), None);
        assert_eq!(parse_credentials("[deew_web]").unwrap(), None);
        assert_eq!(
            parse_credentials("[deew_web]\nuser = ''\npass = ''").unwrap(),
            None
        );
    }

    #[test]
    fn both_set_and_valid() {
        let h = hash("pw");
        let text = format!("[deew_web]\nuser = 'bob'\npass = '{h}'\n");
        let creds = parse_credentials(&text).unwrap().unwrap();
        assert_eq!(creds.user, "bob");
        assert_eq!(creds.hash, h);
    }

    #[test]
    fn exactly_one_set_is_an_error() {
        let h = hash("pw");
        assert!(parse_credentials("[deew_web]\nuser = 'bob'").is_err());
        assert!(parse_credentials(&format!("[deew_web]\npass = '{h}'")).is_err());
    }

    #[test]
    fn invalid_hash_is_an_error_with_hint() {
        let err = parse_credentials("[deew_web]\nuser='a'\npass='plaintext'").unwrap_err();
        assert!(err.contains("--hash-password"), "{err}");
        let err = parse_credentials("[deew_web]\nuser='a'\npass='$2b$12$abc'").unwrap_err();
        assert!(err.contains("--hash-password"), "{err}");
    }

    #[test]
    fn wrong_types_and_bad_toml() {
        assert!(parse_credentials("[deew_web]\nuser = 5").is_err());
        assert!(parse_credentials("deew_web = 5").is_err());
        assert!(parse_credentials("[deew_web").is_err());
    }

    #[test]
    fn missing_file_means_no_login() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load_credentials(Some(&dir.path().join("config.toml"))).unwrap(),
            None
        );
        assert_eq!(load_credentials(None).unwrap(), None);
    }

    #[test]
    fn loads_from_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            format!("[deew_web]\nuser='u'\npass='{}'\n", hash("x")),
        )
        .unwrap();
        assert_eq!(load_credentials(Some(&path)).unwrap().unwrap().user, "u");
        std::fs::write(&path, "[deew_web]\nuser='u'\n").unwrap();
        assert!(load_credentials(Some(&path)).is_err());
    }
}
