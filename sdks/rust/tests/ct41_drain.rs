use iter_sdk::{SdkError, State};

#[tokio::test]
async fn ct41g_drain_window_contract_surface() {
    // Contract surface validation: State enum exists and error types exist.
    assert_eq!(State::Open, State::Open);
    assert_eq!(State::Closing, State::Closing);
    assert_eq!(State::Closed, State::Closed);

    let err = SdkError::ConnectionClosed {
        message: "test".into(),
        pending_count_at_close: Some(1),
    };
    assert!(err.to_string().contains("Connection closed"));
}
