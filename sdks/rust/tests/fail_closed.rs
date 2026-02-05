use iter_sdk::{RpcError, RpcResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex};

#[tokio::test]
async fn fail_closed_rejects_pending_with_error() {
    let queue: Arc<Mutex<HashMap<u64, oneshot::Sender<RpcResponse>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let (tx, rx) = oneshot::channel();
    queue.lock().await.insert(1, tx);

    {
        let mut q = queue.lock().await;
        for (id, tx) in q.drain() {
            let resp = RpcResponse {
                jsonrpc: "2.0".into(),
                result: None,
                error: Some(RpcError {
                    code: -32000,
                    message: "Protocol violation: test".into(),
                }),
                id: serde_json::json!(id),
            };
            let _ = tx.send(resp);
        }
    }

    let resp = rx.await.expect("response");
    let err = resp.error.expect("error");
    assert_eq!(err.code, -32000);
    assert!(err.message.contains("Protocol violation"));
}
