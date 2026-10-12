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
        /// 글의 출처 — `cli` = 보낸 쪽 사용자 AI 에이전트가 CLI 로 쓴 글. 이전 버전 상대는 싣지 않고 무시한다(사람 글로 보인다).
        #[serde(default)]
        via: Option<String>,
        /// 사람 글의 종류 — 참 = 요청(건의), 거짓 = 질의(기본). 받은 쪽 이력에도 남는다.
        #[serde(default)]
        request: bool,
        /// 멘션 — 그 대화에서 부른 받는 PC 의 다른 에이전트(`agent` 는 늘 대화 에이전트 — 받은 쪽에 대화가 아직 없어도 대화를 그 에이전트로 연다).
        #[serde(default)]
        mention: Option<String>,
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
        /// 답을 실행한 model · effort — 받은 쪽은 `run_opts` 글자 규칙에 맞는 것만 남긴다.
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        effort: Option<String>,
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
        /// 글의 출처 — `Message` 의 `via` 와 같다.
        #[serde(default)]
        via: Option<String>,
        /// 에이전트 답의 model · effort — `Reply` 와 같다.
        #[serde(default)]
        model: Option<String>,
        #[serde(default)]
        effort: Option<String>,
        /// 사람 글의 종류 — `Message` 의 `request` 와 같다.
        #[serde(default)]
        request: bool,
    },
    /// 요청 완료 표시 — 그 글을 받은 쪽이 바꾸고 쓴 쪽에 상태로 보낸다(쓴 쪽은 자기 요청 글에만 받는다). 페어링된 상대만.
    Done {
        conv: String,
        msg: String,
        done: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Response {
    Hello {
        name: String,
        host: String,
        /// 초대 생성 측이 이 hello 를 초대 코드로 자동 승인했다 — 받은 쪽은 같은 세대의 상대 승인으로 기록한다.
        #[serde(default)]
        approved: bool,
    },
    Ok,
    /// `Ping` 의 답 — 답한 쪽이 집중(설정 `focus`) 중인가(접속 상태 · 상대 집중 표시).
    Pong {
        #[serde(default)]
        focus: bool,
    },
    Agents {
        names: Vec<String>,
    },
    /// `not_paired` · `unknown` · `stale`(이전 세대 승인) · `busy` · `closed`(페어링 받기 창이 닫혔거나 초대 코드 불일치) ·
    /// `bad_request` · `unknown_agent` · `unsupported`(받는 쪽에 처리기가 없음) · `focus`(받는 쪽이 집중 중 — 새 실행 요청 거부)
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

/// 상대가 준 오류 코드 — 영소문자 · `_` 1 ~ 32 자만 그대로, 그 밖은 `unknown`(저장 · 표시 전에 — 페어링된 상대의 값도 그대로 믿지 않는다).
pub fn clean_code(code: &str) -> &str {
    let ok =
        (1..=32).contains(&code.len()) && code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_');
    if ok { code } else { "unknown" }
}

/// 제어 문자를 빼고 앞뒤 공백을 지운 뒤 `MAX_NAME_CHARS` 글자로 자른다. 비면 `?`.
pub fn clean_name(s: &str) -> String {
    let kept: String = s.chars().filter(|c| !c.is_control()).collect();
    let cut: String = kept.trim().chars().take(MAX_NAME_CHARS).collect();
    if cut.is_empty() { "?".into() } else { cut }
}

#[cfg(test)]
mod tests {
    use super::clean_code;

    #[test]
    fn clean_code_keeps_known_shape_only() {
        assert_eq!(clean_code("focus"), "focus");
        assert_eq!(clean_code("unknown_agent"), "unknown_agent");
        let long = "x".repeat(33);
        for bad in ["", "Focus", "focus!", "a\u{202e}b", long.as_str()] {
            assert_eq!(clean_code(bad), "unknown", "{bad:?}");
        }
    }
}
