//! 피어 프레임 — 4바이트 big-endian 길이 + JSON. 연결 1개 = 요청 1개 · 응답 1개. 원격 입력은 신뢰하지 않는다.
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// 이보다 긴 프레임은 본문을 읽지 않고 끊는다.
pub const MAX_FRAME: usize = 64 * 1024;
/// 원격 메시지 본문 상한(바이트, R4).
pub const MAX_TEXT: usize = 16 * 1024;
/// 원격에서 온 이름 · 호스트의 최대 글자 수.
pub const MAX_NAME_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    /// 페어링 시작 — 보낸 쪽 이름 · 호스트 · 피어 수신 포트 · 이번 페어링 세대(`round`, 보낸 쪽이 만든 난수) ·
    /// 받는 쪽이 보여 준 초대 코드(처음 보는 사이에만 필요).
    Hello {
        name: String,
        host: String,
        port: u16,
        round: String,
        #[serde(default)]
        code: Option<String>,
    },
    /// 내 승인 — 같은 `round` 의 기록에만 적용된다(늦게 온 이전 세대 승인은 `stale`).
    Approve {
        round: String,
    },
    Remove,
    Ping,
    /// 상대 PC 의 에이전트 이름 목록(페어링된 상대만).
    Agents,
    /// 사람 → 원격 에이전트 메시지(페어링된 상대만). `conv` · `msg` 는 보낸 쪽이 만든 UUID.
    Message {
        conv: String,
        msg: String,
        agent: String,
        text: String,
        /// 대화 안 양쪽 공통 순서(램포트 시계) — 이전 버전 상대는 싣지 않는다.
        #[serde(default)]
        lc: Option<i64>,
        /// 자동 왕복(D24) — 보낸 쪽 에이전트 이름. 있으면 사람이 아니라 그 에이전트의 실행 요청.
        #[serde(default)]
        by_agent: Option<String>,
        /// 자동 왕복을 여는 사람 요청 — 연 쪽 내 에이전트 이름. 받은 쪽은 그 대화를 자동 왕복으로 기록하고 알림을 "자동 왕복 시작"
        /// 하나로(알림 묶기).
        #[serde(default)]
        relay: Option<String>,
    },
    /// 에이전트의 답을 보낸 쪽으로 되돌린다(페어링된 상대만). `ok = false` 면 `text` 는 실패 사유.
    /// `turns` · `duration_ms` 는 실행 정보 — 이전 버전 상대는 싣지 않는다(없으면 `None`).
    Reply {
        conv: String,
        msg: String,
        reply_to: String,
        text: String,
        ok: bool,
        #[serde(default)]
        turns: Option<i64>,
        #[serde(default)]
        duration_ms: Option<i64>,
        #[serde(default)]
        lc: Option<i64>,
    },
    /// 실행 없이 상대 이력에 남기는 글(D18 · D23) — `author = "human"` 사람 글(`exec` = 보낸 쪽 에이전트에게 한 실행 요청인가),
    /// `author = "agent"` 보낸 쪽 에이전트의 답(`reply_to` = 그 실행 요청 사람 글 uuid). 페어링된 상대만.
    /// 자동 왕복 켜기 · 끄기 알림(D24) — 받은 쪽 → 연 쪽 `on = false`(멈춤), 연 쪽 → 받은 쪽 `on`(재개 = 거절 해제 · 멈춤).
    Relay {
        conv: String,
        on: bool,
    },
    Post {
        conv: String,
        msg: String,
        author: String,
        text: String,
        #[serde(default)]
        exec: bool,
        #[serde(default)]
        reply_to: Option<String>,
        #[serde(default)]
        ok: Option<bool>,
        #[serde(default)]
        turns: Option<i64>,
        #[serde(default)]
        duration_ms: Option<i64>,
        #[serde(default)]
        lc: Option<i64>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Hello {
        name: String,
        host: String,
    },
    Ok,
    Pong,
    Agents {
        names: Vec<String>,
    },
    /// `not_paired` · `unknown` · `stale`(이전 세대 승인) · `busy` · `closed`(페어링 받기 창이 닫혔거나 초대 코드 불일치) ·
    /// `bad_request` · `unknown_agent` · `unsupported`(받는 쪽에 처리기가 없음)
    Error {
        code: String,
    },
}

impl Response {
    pub fn error(code: &str) -> Self {
        Response::Error { code: code.into() }
    }
}

pub async fn write_frame<W: AsyncWrite + Unpin, T: Serialize>(
    w: &mut W,
    msg: &T,
) -> io::Result<()> {
    let body = serde_json::to_vec(msg)?;
    if body.len() > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "프레임이 너무 크다",
        ));
    }
    w.write_all(&(body.len() as u32).to_be_bytes()).await?;
    w.write_all(&body).await?;
    w.flush().await
}

pub async fn read_frame<R: AsyncRead + Unpin, T: DeserializeOwned>(r: &mut R) -> io::Result<T> {
    let mut len = [0u8; 4];
    r.read_exact(&mut len).await?;
    let len = u32::from_be_bytes(len) as usize;
    if len > MAX_FRAME {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("프레임 {len} 바이트 > {MAX_FRAME}"),
        ));
    }
    let mut body = vec![0u8; len];
    r.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body)?)
}

/// 제어 문자를 빼고 앞뒤 공백을 지운 뒤 `MAX_NAME_CHARS` 글자로 자른다. 비면 `?`.
pub fn clean_name(s: &str) -> String {
    let kept: String = s.chars().filter(|c| !c.is_control()).collect();
    let cut: String = kept.trim().chars().take(MAX_NAME_CHARS).collect();
    if cut.is_empty() { "?".into() } else { cut }
}
