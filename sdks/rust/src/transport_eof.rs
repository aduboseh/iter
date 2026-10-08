use super::*;
use std::future::Future;
use std::path::PathBuf;
use std::task::Poll;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "iter-sdk-eof-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir(&dir).unwrap();
        let fixture = Self(dir);
        let compiler = std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
        let output = std::process::Command::new(compiler)
            .arg("--edition=2021")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/transport_child.rs"
            ))
            .arg("-o")
            .arg(fixture.binary())
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        fixture
    }

    fn binary(&self) -> PathBuf {
        self.0
            .join(format!("transport-child{}", std::env::consts::EXE_SUFFIX))
    }

    async fn connect(&self, mode: &str, capacity: usize) -> IterClient {
        IterClient::connect_with_runtime_mode(self.binary().to_str().unwrap(), capacity, mode)
            .await
            .unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).expect("fixture process must be reaped before cleanup");
    }
}

async fn close(client: &mut IterClient) {
    timeout(Duration::from_secs(10), client.close())
        .await
        .unwrap()
        .unwrap();
    client.close().await.unwrap();
    assert_eq!(client.state().await, State::Closed);
    assert!(client.process.lock().await.try_wait().unwrap().is_some());
}

async fn assert_transport_failure(mode: &str) {
    let fixture = Fixture::new();
    let mut client = fixture.connect(mode, 4).await;
    let params =
        (mode == "blocked-write").then(|| serde_json::json!({"data": "x".repeat(2 * 1024 * 1024)}));
    let result = timeout(Duration::from_secs(5), client.send("test", params, 30000)).await;
    let state = client.state().await;
    let pending = client.response_queue.lock().await.len();
    let alive = client.process.lock().await.try_wait().unwrap().is_none();
    let later = timeout(Duration::from_secs(1), client.send("later", None, 30000)).await;
    close(&mut client).await;
    assert!(
        matches!(result, Ok(Err(SdkError::ConnectionFailed(ref reason))) if reason.contains("stdout")),
        "{mode}: {result:?}"
    );
    assert_eq!(state, State::Closing, "{mode}");
    assert_eq!(pending, 0, "{mode}");
    assert!(
        matches!(later, Ok(Err(SdkError::ConnectionClosed { .. }))),
        "{mode}: {later:?}"
    );
    if mode != "exit" {
        assert!(alive, "stdout EOF must not claim that the child exited");
    }
}

#[tokio::test]
async fn exit_without_response_fails_promptly() {
    assert_transport_failure("exit").await;
}

#[tokio::test]
async fn stdout_closed_child_alive_is_still_reaped() {
    assert_transport_failure("alive").await;
}

#[tokio::test]
async fn read_failure_is_terminal() {
    assert_transport_failure("read-error").await;
}

#[tokio::test]
async fn eof_interrupts_blocked_stdin_write() {
    assert_transport_failure("blocked-write").await;
}

#[tokio::test]
async fn buffered_response_is_delivered_before_eof() {
    let fixture = Fixture::new();
    let mut client = fixture.connect("response", 1).await;
    let result = timeout(Duration::from_secs(5), client.send("test", None, 30000)).await;
    let terminal = timeout(Duration::from_secs(5), async {
        while client.state().await == State::Open {
            tokio::task::yield_now().await;
        }
    })
    .await;
    let alive = client.process.lock().await.try_wait().unwrap().is_none();
    close(&mut client).await;
    assert_eq!(
        result.unwrap().unwrap().result,
        Some(serde_json::json!({"ok": true}))
    );
    assert!(terminal.is_ok());
    assert!(alive);
}

#[tokio::test]
async fn eof_drains_all_pending_and_preserves_stderr() {
    let fixture = Fixture::new();
    let mut client = fixture.connect("two-requests", 4).await;
    let outcomes = timeout(Duration::from_secs(5), async {
        tokio::join!(client.send("a", None, 30000), client.send("b", None, 30000))
    })
    .await;
    let diagnostic = timeout(Duration::from_secs(5), async {
        loop {
            if String::from_utf8_lossy(&client.stderr_ring.lock().await)
                .contains("transport fixture diagnostic")
            {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await;
    let pending = client.response_queue.lock().await.len();
    close(&mut client).await;
    let (first, second) = outcomes.unwrap();
    assert!(matches!(first, Err(SdkError::ConnectionFailed(_))));
    assert!(matches!(second, Err(SdkError::ConnectionFailed(_))));
    assert!(diagnostic.is_ok());
    assert_eq!(pending, 0);
}

#[tokio::test]
async fn timeout_releases_capacity_without_closing_transport() {
    let fixture = Fixture::new();
    let mut client = fixture.connect("silent", 1).await;
    let mut pending = Box::pin(client.send("first", None, 200));
    std::future::poll_fn(|cx| {
        assert!(pending.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    let overflow = client.send("overflow", None, 30000).await;
    let first = timeout(Duration::from_secs(5), pending).await;
    let remaining = client.response_queue.lock().await.len();
    let state = client.state().await;
    let next = timeout(Duration::from_secs(5), client.send("next", None, 20)).await;
    close(&mut client).await;
    assert!(matches!(overflow, Err(SdkError::Backpressure(1))));
    assert!(matches!(first, Ok(Err(SdkError::RequestTimeout { .. }))));
    assert_eq!(remaining, 0);
    assert_eq!(state, State::Open);
    assert!(matches!(next, Ok(Err(SdkError::RequestTimeout { .. }))));
}

#[tokio::test]
async fn admission_holds_state_until_queue_registration() {
    let fixture = Fixture::new();
    let mut client = fixture.connect("alive", 1).await;
    let queue = client.response_queue.lock().await;
    let mut send = Box::pin(client.send("test", None, 30000));
    std::future::poll_fn(|cx| {
        assert!(send.as_mut().poll(cx).is_pending());
        Poll::Ready(())
    })
    .await;
    let admission_locked = client.state.try_lock().is_err();
    drop(queue);
    let result = timeout(Duration::from_secs(5), send).await;
    close(&mut client).await;
    assert!(
        admission_locked,
        "EOF must not interleave state validation and registration"
    );
    assert!(matches!(result, Ok(Err(SdkError::ConnectionFailed(_)))));
}

#[tokio::test]
async fn eof_during_close_does_not_wait_for_drain_timeout() {
    let fixture = Fixture::new();
    let mut client = fixture.connect("alive", 1).await;
    let (tx, rx) = oneshot::channel();
    client.response_queue.lock().await.insert(1, tx);
    client
        .stdin
        .lock()
        .await
        .write_all(b"request\n")
        .await
        .unwrap();
    let result = timeout(Duration::from_secs(2), client.close()).await;
    close(&mut client).await;
    assert!(matches!(rx.await, Ok(Err(SdkError::ConnectionFailed(_)))));
    assert!(matches!(result, Ok(Ok(()))));
}
