//! Password hashing, credential checks and in-memory login sessions.

use crate::config::Credentials;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;

pub const COOKIE_NAME: &str = "deew_session";
pub const SESSION_LIFETIME: Duration = Duration::from_secs(7 * 24 * 3600);

/// Hashes a password with argon2id (default parameters, random salt) into a PHC string.
pub fn hash_password(password: &str) -> Result<String, String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|hash| hash.to_string())
        .map_err(|e| format!("hashing failed: {e}"))
}

/// Verifies `password` against a PHC string.
pub fn verify_password(password: &str, hash: &str) -> bool {
    match PasswordHash::new(hash) {
        Ok(parsed) => Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok(),
        Err(_) => false,
    }
}

/// Checks user (constant time) and password. The password is verified even
/// when the user is wrong so both cases take the same time.
pub fn check_credentials(creds: &Credentials, user: &str, pass: &str) -> bool {
    let user_ok: bool = creds.user.as_bytes().ct_eq(user.as_bytes()).into();
    let pass_ok = verify_password(pass, &creds.hash);
    user_ok & pass_ok
}

/// A random 32-byte token, hex encoded.
pub fn new_token() -> Result<String, String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| format!("no random numbers available: {e}"))?;
    Ok(hex::encode(bytes))
}

/// Extracts the session token from a `Cookie` header value.
pub fn token_from_cookie_header(header: &str) -> Option<&str> {
    header.split(';').find_map(|part| {
        let (name, value) = part.trim().split_once('=')?;
        (name == COOKIE_NAME && !value.is_empty()).then_some(value)
    })
}

#[derive(Default)]
pub struct Sessions {
    map: Mutex<HashMap<String, Instant>>,
}

impl Sessions {
    fn map(&self) -> std::sync::MutexGuard<'_, HashMap<String, Instant>> {
        self.map.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn create(&self) -> Result<String, String> {
        self.create_with_lifetime(SESSION_LIFETIME)
    }

    pub fn create_with_lifetime(&self, lifetime: Duration) -> Result<String, String> {
        let token = new_token()?;
        let now = Instant::now();
        let mut map = self.map();
        map.retain(|_, expires| *expires > now);
        map.insert(token.clone(), now + lifetime);
        Ok(token)
    }

    pub fn is_valid(&self, token: &str) -> bool {
        let mut map = self.map();
        match map.get(token) {
            Some(expires) if *expires > Instant::now() => true,
            Some(_) => {
                map.remove(token);
                false
            }
            None => false,
        }
    }

    pub fn remove(&self, token: &str) {
        self.map().remove(token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let hash = hash_password("correct horse").unwrap();
        assert!(hash.starts_with("$argon2id$"), "{hash}");
        assert!(verify_password("correct horse", &hash));
        assert!(!verify_password("wrong", &hash));
        assert!(!verify_password("correct horse", "garbage"));
        assert_ne!(hash, hash_password("correct horse").unwrap(), "random salt");
    }

    #[test]
    fn credentials_need_both() {
        let creds = Credentials {
            user: "bob".into(),
            hash: hash_password("pw").unwrap(),
        };
        assert!(check_credentials(&creds, "bob", "pw"));
        assert!(!check_credentials(&creds, "bob", "nope"));
        assert!(!check_credentials(&creds, "bo", "pw"));
        assert!(!check_credentials(&creds, "bobb", "pw"));
    }

    #[test]
    fn tokens_are_hex_64() {
        let a = new_token().unwrap();
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, new_token().unwrap());
    }

    #[test]
    fn cookie_parsing() {
        assert_eq!(
            token_from_cookie_header("a=b; deew_session=abc; c=d"),
            Some("abc")
        );
        assert_eq!(token_from_cookie_header("deew_session=abc"), Some("abc"));
        assert_eq!(token_from_cookie_header("xdeew_session=abc"), None);
        assert_eq!(token_from_cookie_header("deew_session="), None);
        assert_eq!(token_from_cookie_header(""), None);
    }

    #[test]
    fn sessions_expire_and_logout() {
        let s = Sessions::default();
        let t = s.create().unwrap();
        assert!(s.is_valid(&t));
        assert!(!s.is_valid("other"));
        s.remove(&t);
        assert!(!s.is_valid(&t));
        let t = s.create_with_lifetime(Duration::ZERO).unwrap();
        assert!(!s.is_valid(&t));
    }
}
