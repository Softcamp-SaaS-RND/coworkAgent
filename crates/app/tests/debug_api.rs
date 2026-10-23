//! 디버깅 API(#1z-8) — 토큰 필요 · 파일 정보(경로 · 크기, 신원 키 · 토큰 · 지문 칸 없음) · 로그 끝 200줄 · 64 KiB 상한.
mod common;
use an_app::server;
use common::{host, send};
use serde_json::Value;

/// `data_dir` 를 시험 폴더로 둔 서버.
async fn start(tag: &str) -> (u16, String, std::path::PathBuf) {
    let root = std::env::temp_dir().join(format!("an-app-debug-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("logs")).unwrap();
    let listener = server::bind().await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let token = server::new_token();
    let status = server::Status {
        version: "9.9.9".into(),
        data_dir: root.display().to_string(),
        db: "ok".into(),
    };
    let app = server::router(port, token.clone(), status, || Ok(()));
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (port, token, root)
}

async fn get(port: u16, token: &str, path: &str) -> (u16, Value) {
    let (code, raw) = send(
        port,
        "GET",
        path,
        &[host(port), ("Authorization", format!("Bearer {token}"))],
    )
    .await;
    let body = raw.split("\r\n\r\n").nth(1).unwrap_or("");
    (code, serde_json::from_str(body).unwrap_or(Value::Null))
}

#[tokio::test]
async fn debug_needs_token() {
    let (port, _, _) = start("tok").await;
    for path in ["/api/debug", "/api/debug/log"] {
        let (code, _) = send(port, "GET", path, &[host(port)]).await;
        assert_eq!(code, 401, "{path}");
    }
}

#[tokio::test]
async fn debug_info_paths_sizes_no_secrets() {
    let (port, token, root) = start("info").await;
    std::fs::write(root.join("coworkAgent.db"), vec![0u8; 1234]).unwrap();
    std::fs::write(root.join("identity.key"), "SECRET-KEY").unwrap();
    let (code, r) = get(port, &token, "/api/debug").await;
    assert_eq!(code, 200, "{r}");
    let mut keys: Vec<&str> = r.as_object().unwrap().keys().map(String::as_str).collect();
    keys.sort();
    assert_eq!(keys, ["arch", "data_dir", "files", "os", "version"], "{r}");
    assert_eq!(r["version"], "9.9.9");
    assert_eq!(r["files"][0]["key"], "db");
    assert_eq!(r["files"][0]["size"], 1234);
    assert!(r["files"][0]["path"].as_str().unwrap().ends_with("coworkAgent.db"));
    assert_eq!(r["files"][1]["key"], "log");
    assert_eq!(r["files"][1]["size"], Value::Null, "없는 로그 = null");
    let text = r.to_string().to_lowercase();
    for bad in ["secret", "identity", "instance", "token", "fingerprint", "지문"] {
        assert!(!text.contains(bad), "{bad} — {r}");
    }
}

#[tokio::test]
async fn debug_log_last_200_lines() {
    let (port, token, root) = start("tail").await;
    let body: String = (1..=300).map(|i| format!("{i} INFO 줄 {i}\n")).collect();
    std::fs::write(root.join("logs").join("coworkAgent.log"), body).unwrap();
    let (code, r) = get(port, &token, "/api/debug/log").await;
    assert_eq!(code, 200, "{r}");
    let lines: Vec<&str> = r["lines"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
    assert_eq!(lines.len(), server::LOG_TAIL_LINES);
    assert_eq!(lines[0], "101 INFO 줄 101");
    assert_eq!(lines[199], "300 INFO 줄 300");
}

#[tokio::test]
async fn debug_log_reads_at_most_64k() {
    let (port, token, root) = start("cap").await;
    // 1000 바이트 줄 300개(약 300 KB) — 끝 64 KiB 안의 온전한 줄만, 잘린 첫 줄 없음.
    let body: String = (0..300).map(|i| format!("L{i:04}{}\n", "x".repeat(994))).collect();
    std::fs::write(root.join("logs").join("coworkAgent.log"), body).unwrap();
    let (_, r) = get(port, &token, "/api/debug/log").await;
    let lines: Vec<&str> = r["lines"].as_array().unwrap().iter().map(|l| l.as_str().unwrap()).collect();
    let bytes: usize = lines.iter().map(|l| l.len() + 1).sum();
    assert!(bytes as u64 <= server::LOG_TAIL_BYTES, "{bytes}");
    assert!(lines.len() >= 60 && lines.len() <= 66, "{}", lines.len());
    assert!(lines.iter().all(|l| l.starts_with('L') && l.len() == 999), "잘린 줄 없음");
    assert_eq!(*lines.last().unwrap(), format!("L0299{}", "x".repeat(994)));
}

#[tokio::test]
async fn debug_log_missing_is_empty() {
    let (port, token, _) = start("none").await;
    let (code, r) = get(port, &token, "/api/debug/log").await;
    assert_eq!(code, 200, "{r}");
    assert_eq!(r["lines"], serde_json::json!([]));
}
