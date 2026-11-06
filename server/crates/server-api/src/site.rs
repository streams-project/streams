//! The website, served from a folder. Addresses under `/v1` go to the API and everything else to
//! the site, where an unknown path gets the page itself, because the core owns the addresses.

use std::path::Path;

use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::{Next, from_fn};
use axum::response::IntoResponse;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::compression::CompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

/// Scripts come only from this site. Video and images may come from anywhere a finder or the data
/// set names, and hls.js builds its worker from a blob. The TV page runs under this same policy, so
/// a set below Chrome 97, which does not know 'wasm-unsafe-eval', shows that page's notice.
const POLICY: &str = "default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; \
    style-src 'self' 'unsafe-inline'; img-src * data:; media-src * blob:; connect-src *; \
    worker-src blob:; frame-ancestors 'none'; base-uri 'none'";

fn fixed(name: &'static str, value: &'static str) -> SetResponseHeaderLayer<HeaderValue> {
    SetResponseHeaderLayer::overriding(
        HeaderName::from_static(name),
        HeaderValue::from_static(value),
    )
}

fn for_the_api(path: &str) -> bool {
    path == "/v1" || path.starts_with("/v1/") || path.starts_with("/internal/")
}

/// The API with the website around it. This looks at the path alone: routing it through the
/// router again would add a captured value that every `Path` in the API would then read.
pub fn with_site(api: Router, root: &Path) -> Router {
    let files = ServeDir::new(root).fallback(ServeFile::new(root.join("index.html")));
    let site = ServiceBuilder::new()
        .layer(fixed("content-security-policy", POLICY))
        .layer(fixed("x-content-type-options", "nosniff"))
        .layer(fixed("referrer-policy", "no-referrer"))
        // The file names stay the same across updates, so a browser must ask before reusing one,
        // or an old script meets a new wasm file and the page does not start.
        .layer(fixed("cache-control", "no-cache"))
        .layer(CompressionLayer::new())
        .service(files);
    api.layer(from_fn(move |request: Request, next: Next| {
        let site = site.clone();
        async move {
            if for_the_api(request.uri().path()) {
                next.run(request).await
            } else {
                site.oneshot(request).await.into_response()
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::extract::Path as AxumPath;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use tower::ServiceExt;

    use super::*;

    fn folder() -> std::path::PathBuf {
        let name = format!(
            "site_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        );
        let dir = std::env::temp_dir().join(name.replace(['(', ')'], ""));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("index.html"), "the page").unwrap();
        std::fs::write(dir.join("app.js"), "the script").unwrap();
        dir
    }

    fn app(root: &Path) -> Router {
        let api = Router::new()
            .route("/v1/hello", get(|| async { "the api" }))
            .route(
                "/v1/things/{id}",
                get(|AxumPath(id): AxumPath<String>| async move { id }),
            );
        with_site(api, root)
    }

    async fn read(app: Router, path: &str) -> (StatusCode, String, bool) {
        let request = Request::get(path).body(Body::empty()).unwrap();
        let answer = app.oneshot(request).await.unwrap();
        let status = answer.status();
        let policy = answer.headers().contains_key("content-security-policy");
        let body = to_bytes(answer.into_body(), 4096).await.unwrap();
        (status, String::from_utf8_lossy(&body).into_owned(), policy)
    }

    #[tokio::test]
    async fn api_addresses_reach_the_api_and_carry_no_page_policy() {
        let dir = folder();
        let (status, body, policy) = read(app(&dir), "/v1/hello").await;
        assert_eq!(
            (status, body.as_str(), policy),
            (StatusCode::OK, "the api", false)
        );
    }

    #[tokio::test]
    async fn an_api_address_with_a_value_in_it_keeps_exactly_that_value() {
        let dir = folder();
        let (status, body, _) = read(app(&dir), "/v1/things/abc").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "abc"));
    }

    #[tokio::test]
    async fn a_file_is_served_with_the_policy() {
        let dir = folder();
        let (status, body, policy) = read(app(&dir), "/app.js").await;
        assert_eq!(
            (status, body.as_str(), policy),
            (StatusCode::OK, "the script", true)
        );
    }

    #[tokio::test]
    async fn any_other_address_gets_the_page_so_a_deep_link_opens() {
        let dir = folder();
        for path in ["/", "/home", "/title/movie/7", "/watch/tv/1/2/3"] {
            let (status, body, _) = read(app(&dir), path).await;
            assert_eq!(
                (status, body.as_str()),
                (StatusCode::OK, "the page"),
                "{path}"
            );
        }
    }

    #[tokio::test]
    async fn no_path_climbs_out_of_the_folder() {
        let dir = folder();
        let outside = dir.parent().unwrap().join("outside.txt");
        std::fs::write(&outside, "secret").unwrap();
        let (_, body, _) = read(app(&dir), "/../outside.txt").await;
        assert_ne!(body, "secret");
    }

    #[tokio::test]
    async fn an_unknown_api_address_is_not_answered_with_the_page() {
        let dir = folder();
        let (status, body, _) = read(app(&dir), "/v1/nothing").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_ne!(body, "the page");
    }
}
