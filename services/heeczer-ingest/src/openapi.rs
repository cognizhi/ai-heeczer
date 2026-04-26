//! OpenAPI contract serving for the ingestion service.

use axum::http::{header, HeaderValue};
use axum::response::{IntoResponse, Response};

/// Serve the checked-in OpenAPI YAML contract.
pub async fn openapi_yaml() -> Response {
    let mut response = include_str!("../openapi.yaml").into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/yaml"),
    );
    response
}
