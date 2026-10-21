//! The embedded frontend (`gui/dist`).

use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../dist"]
struct Dist;

const MISSING: &str = "frontend not built: run npm run build in gui/ and rebuild deew-web\n";

/// Serves `path` (without leading slash) from the embedded frontend, falling
/// back to `index.html` for unknown paths (client-side routing).
pub fn serve(path: &str) -> Response {
    let path = path.trim_start_matches('/');
    let (file, name) = match Dist::get(path).filter(|_| !path.is_empty()) {
        Some(file) => (file, path),
        None => match Dist::get("index.html") {
            Some(file) => (file, "index.html"),
            None => {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
                    MISSING,
                )
                    .into_response();
            }
        },
    };
    let mime = mime_guess::from_path(name).first_or_octet_stream();
    let mut response = file.data.into_owned().into_response();
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(mime.as_ref()) {
        headers.insert(header::CONTENT_TYPE, value);
    }
    // Hashed bundles never change; everything else must be revalidated.
    let cache = if name.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_is_embedded_and_is_the_fallback() {
        let r = serve("");
        assert_eq!(r.status(), StatusCode::OK);
        assert_eq!(r.headers()[header::CONTENT_TYPE], "text/html");
        let r = serve("some/spa/route");
        assert_eq!(r.headers()[header::CONTENT_TYPE], "text/html");
    }
}
