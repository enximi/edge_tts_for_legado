use axum::http::HeaderMap;

pub fn request_scheme(headers: &HeaderMap) -> &str {
    header_value(headers, "x-forwarded-proto").unwrap_or("http")
}

pub fn request_host(headers: &HeaderMap) -> &str {
    header_value(headers, "host").unwrap_or("localhost:8000")
}

pub fn request_origin(headers: &HeaderMap) -> String {
    format!("{}://{}", request_scheme(headers), request_host(headers))
}

fn header_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::*;

    #[test]
    fn prefers_forwarded_proto_for_scheme() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-proto", HeaderValue::from_static("https"));

        assert_eq!(request_scheme(&headers), "https");
    }

    #[test]
    fn falls_back_to_http_and_localhost() {
        let headers = HeaderMap::new();

        assert_eq!(request_scheme(&headers), "http");
        assert_eq!(request_host(&headers), "localhost:8000");
        assert_eq!(request_origin(&headers), "http://localhost:8000");
    }

    #[test]
    fn reads_host_header() {
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("example.com"));

        assert_eq!(request_host(&headers), "example.com");
    }
}
