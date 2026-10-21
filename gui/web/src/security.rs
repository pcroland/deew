//! Pure helpers for the Host / Origin checks and the security headers.

pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; font-src 'self' data:; connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

/// The hostname part of a `Host` header value (port and IPv6 brackets stripped).
pub fn hostname(host: &str) -> &str {
    let host = host.trim();
    if let Some(rest) = host.strip_prefix('[') {
        return rest.split(']').next().unwrap_or("");
    }
    match host.rsplit_once(':') {
        Some((name, port)) if port.chars().all(|c| c.is_ascii_digit()) => name,
        _ => host,
    }
}

/// DNS-rebinding protection: when bound to loopback only localhost names are accepted.
pub fn host_allowed(host: Option<&str>, loopback_bind: bool) -> bool {
    if !loopback_bind {
        return true;
    }
    let Some(host) = host else {
        return false;
    };
    let name = hostname(host).to_ascii_lowercase();
    matches!(name.as_str(), "localhost" | "127.0.0.1" | "::1")
}

/// CSRF protection: an Origin header, if present, must be `http(s)://<Host>`.
pub fn origin_allowed(origin: Option<&str>, host: Option<&str>) -> bool {
    let Some(origin) = origin else {
        return true;
    };
    let Some(host) = host else {
        return false;
    };
    let host = host.trim().to_ascii_lowercase();
    let origin = origin.trim().to_ascii_lowercase();
    origin == format!("http://{host}") || origin == format!("https://{host}")
}

/// Headers added to every response.
pub fn security_headers() -> [(&'static str, &'static str); 4] {
    [
        ("content-security-policy", CSP),
        ("x-content-type-options", "nosniff"),
        ("referrer-policy", "no-referrer"),
        ("x-frame-options", "DENY"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostnames() {
        assert_eq!(hostname("localhost:8765"), "localhost");
        assert_eq!(hostname("127.0.0.1"), "127.0.0.1");
        assert_eq!(hostname("[::1]:8765"), "::1");
        assert_eq!(hostname("[::1]"), "::1");
        assert_eq!(hostname("evil.com:80"), "evil.com");
    }

    #[test]
    fn host_check() {
        for ok in ["localhost", "LocalHost:1", "127.0.0.1:8765", "[::1]:8765"] {
            assert!(host_allowed(Some(ok), true), "{ok}");
        }
        for bad in [
            "evil.com",
            "evil.com:8765",
            "127.0.0.1.evil.com",
            "10.0.0.2:1",
        ] {
            assert!(!host_allowed(Some(bad), true), "{bad}");
        }
        assert!(!host_allowed(None, true));
        assert!(host_allowed(Some("evil.com"), false));
        assert!(host_allowed(None, false));
    }

    #[test]
    fn origin_check() {
        assert!(origin_allowed(None, Some("localhost:1")));
        assert!(origin_allowed(None, None));
        assert!(origin_allowed(
            Some("http://localhost:1"),
            Some("localhost:1")
        ));
        assert!(origin_allowed(
            Some("https://localhost:1"),
            Some("localhost:1")
        ));
        assert!(origin_allowed(
            Some("http://LOCALHOST:1"),
            Some("localhost:1")
        ));
        assert!(!origin_allowed(
            Some("http://evil.com"),
            Some("localhost:1")
        ));
        assert!(!origin_allowed(
            Some("http://localhost:2"),
            Some("localhost:1")
        ));
        assert!(!origin_allowed(Some("null"), Some("localhost:1")));
        assert!(!origin_allowed(Some("http://localhost:1"), None));
    }
}
