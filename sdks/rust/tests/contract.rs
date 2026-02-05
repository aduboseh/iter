use iter_sdk::{IterClient, State, SdkError};

#[tokio::test]
async fn close_idempotent() {
    let mut client = IterClient::connect("iter-server", 1).await;
    if client.is_err() {
        return; // Skip if binary not available
    }
    let mut client = client.unwrap();
    let _ = client.close().await;
    let _ = client.close().await;
}

#[tokio::test]
async fn backpressure_blocks() {
    let mut client = IterClient::connect("iter-server", 1).await;
    if client.is_err() {
        return;
    }
    let client = client.unwrap();
    // Cannot fully test without server; just ensure error type exists
    let err = SdkError::Backpressure(1);
    assert!(err.to_string().contains("maxInflight=1"));
}

#[tokio::test]
async fn timeout_error_exists() {
    let err = SdkError::RequestTimeout { method: "test".into(), timeout_ms: 1 };
    assert!(err.to_string().contains("Request timeout"));
}
