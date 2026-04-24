/// Integration tests for the HTTP client transport (`http` feature).
///
/// Uses `wiremock` to stub the ingestion service so no real server is needed.
#[cfg(feature = "http")]
mod http_tests {
    use heeczer::http::Client;
    use serde_json::json;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn canonical_event() -> heeczer::Event {
        let raw = include_str!("../../../core/schema/fixtures/events/valid/01-prd-canonical.json");
        serde_json::from_str(raw).unwrap()
    }

    /// Load the golden ScoreResult fixture to use as a valid mock response body.
    fn golden_score() -> serde_json::Value {
        let raw =
            include_str!("../../../core/schema/fixtures/golden/01-prd-canonical.score_result.json");
        serde_json::from_str(raw).unwrap()
    }

    fn ok_envelope(event_id: &str) -> serde_json::Value {
        json!({
            "ok": true,
            "envelope_version": "1",
            "event_id": event_id,
            "score": golden_score()
        })
    }

    #[tokio::test]
    async fn score_event_returns_score_result_on_200() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/events"))
            .and(header("x-heeczer-api-key", "test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(ok_envelope("evt_abc")))
            .mount(&server)
            .await;

        let client = Client::new(server.uri(), "test-key");
        let result = client
            .score_event("ws_default", &canonical_event())
            .await
            .expect("score_event should succeed");

        assert!(!result.scoring_version.is_empty());
        assert!(!result.human_summary.is_empty());
    }

    #[tokio::test]
    async fn score_event_returns_err_on_api_error_envelope() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/v1/events"))
            .respond_with(ResponseTemplate::new(422).set_body_json(json!({
                "ok": false,
                "envelope_version": "1",
                "error": { "kind": "schema", "message": "missing field: model_id" }
            })))
            .mount(&server)
            .await;

        let client = Client::new(server.uri(), "");
        let err = client
            .score_event("ws_default", &canonical_event())
            .await
            .unwrap_err();

        let msg = err.to_string();
        assert!(
            msg.contains("schema"),
            "expected kind in error message, got: {msg}"
        );
    }
}
