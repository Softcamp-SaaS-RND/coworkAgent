//! 피어 프레임 — 4바이트 big-endian 길이 + JSON. 연결 1개 = 요청 1개 · 응답 1개. 원격 입력은 신뢰하지 않는다.
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

/// 실행 중 단계 요약(#1c) — 받는 쪽이 `claude -p` 실시간 이벤트에서 센다. 받은 쪽은 이름을 다시 거른다([`StepDetail::cleaned`]).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepDetail {
    pub turns: u32,
    pub tools: std::collections::BTreeMap<String, u32>,
    pub secs: u32,
}

/// 받은 단계 요약의 도구 이름 수 · 이름 글자 수 상한.
pub const MAX_STEP_TOOLS: usize = 16;
pub const MAX_STEP_NAME: usize = 24;

impl StepDetail {
    /// 원격 값 정리 — 이름은 [`clean_name`] 뒤 24자 · 빈 이름 버림 · 16개까지.
    pub fn cleaned(self) -> Self {
        let tools = self
            .tools
            .into_iter()
            .map(|(k, v)| {
                (
                    clean_name(&k)
                        .chars()
                        .take(MAX_STEP_NAME)
                        .collect::<String>(),
                    v,
                )
            })
            .filter(|(k, _)| !k.is_empty())
            .take(MAX_STEP_TOOLS)
            .collect();
        StepDetail { tools, ..self }
    }
}

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
    /// 연결 확인 — 보낸 쪽 피어 수신 포트(`port`)를 싣는다. 받는 쪽은 페어링된 키면 (연결 출처 IP, `port`) 로 저장 주소를 맞춘다
    /// (임의 포트 · 바뀐 IP 를 승인 초기화 없이 따라감, 1b). 0 = 주소를 바꾸지 않음.
    Ping {
        #[serde(default)]
        port: u16,
    },
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
        /// 그 답을 낸 세션의 창 사용량(토큰, #1m) — 이전 버전 상대는 싣지 않는다(크기 모름).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ctx: Option<i64>,
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
        /// 에이전트 답의 창 사용량 — `Reply` 의 `ctx` 와 같다.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ctx: Option<i64>,
    },
    /// 실행 진행 — 받는 쪽이 상대가 보낸 실행 요청 글(`msg`)의 차례를 알린다(1b). `state` = `queued`(실행 문에서 기다림, `ahead` = 앞 사람 수) |
    /// `running`(실행 시작). 페어링된 상대만.
    Progress {
        conv: String,
        msg: String,
        state: String,
        #[serde(default)]
        ahead: u32,
        /// 실행 중 단계 요약(#1c) — 턴 · 도구 이름별 횟수 · 경과 초만(입력 · 경로 · 모델 글 없음).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        detail: Option<StepDetail>,
    },
    /// 상태 질의(#1c) — 보낸 쪽이 자기 실행 요청 글(`msg`)의 지금 상태를 묻는다. 답 `Response::Status`. 페어링된 상대만.
    Status {
        conv: String,
        msg: String,
    },
    /// 요청 상태(#1d) — 요청 글을 받은 쪽 사람이 바꾸고 쓴 쪽에 보낸다(쓴 쪽은 자기 요청 글에만 받는다). `state` = `wait` · `accepted` ·
    /// `rejected` · `done`, `note` = 한 줄 메모. 페어링된 상대만.
    ReqState {
        conv: String,
        msg: String,
        state: String,
        note: Option<String>,
    },
    /// 이력 정리(#1f) — 쓴 쪽 사람이 자기 사람 글(`msg`)의 본문을 고쳤다. 받는 쪽은 그 상대가 쓴 사람 글에만 적용한다(실행 · 알림 없음).
    /// 페어링된 상대만.
    Edit {
        conv: String,
        msg: String,
        text: String,
    },
    /// 이력 정리(#1f) — 쓴 쪽 사람이 자기 사람 글(`msg`)을 지웠다. 받는 쪽은 그 상대가 쓴 사람 글만 지운다(이미 없으면 `Ok`).
    /// 페어링된 상대만.
    Delete {
        conv: String,
        msg: String,
    },
    /// 컨텍스트(#1m) — 대화의 Claude 세션 압축 · 비우기. `op` = `compact` · `auto_compact` · `clear`(해 달라 — 받은 쪽은 자기 세션을 처리하고 다시
    /// 넘기지 않는다) · `done`(처리했다 — `text` = 요약, 비우기는 빈 값) · `failed`(`text` = 사유 코드). `msg` = 양쪽 같은 구분선 글 uuid(멱등).
    /// 이 종류를 모르는 상대(0.4.0)는 연결을 끊으므로 보내기 전에 `Pong.ctx` 를 본다. 페어링된 상대만.
    Context {
        conv: String,
        msg: String,
        op: String,
        #[serde(default)]
        text: String,
    },
    /// 공유 대화(#1o) 시작 · 다시 초대 — 대화를 가진 PC(A)가 관람자(C)에게. C 는 같은 uuid 의 읽기 전용 사본을 만들고(있으면 글을 비우고) 이어
    /// `SharePost` 로 이력을 받는다. `owner_dir` = A 쪽 대화 방향(`out` · `in`), `other` = A 대화의 원 상대 이름. 이 종류를 모르는 상대(0.5.0)는
    /// 연결을 끊으므로 보내기 전에 `Pong.share` 를 본다. 페어링된 상대만.
    Share {
        conv: String,
        title: String,
        agent: String,
        owner_dir: String,
        other: String,
    },
    /// 공유 대화 글 하나(#1o — 글 하나 = 프레임 하나) — 같은 uuid 면 고친 글로 덮어쓴다.
    SharePost {
        conv: String,
        msg: ShareMsg,
    },
    /// 공유 대화 글 지움(#1o).
    ShareDelete {
        conv: String,
        msg: String,
    },
    /// 공유 끝(#1o) — A → C 빼기 · C → A 나가기. C 의 사본은 읽기 전용으로 남는다.
    ShareEnd {
        conv: String,
    },
    /// 원 상대(B)에게 그 대화의 관람자 수(#1o) — 이름은 싣지 않는다.
    ShareNotice {
        conv: String,
        n: u32,
    },
    /// 대화 나가기(#1w) — 보낸 쪽이 그 대화를 끝냈다. 받은 쪽은 그 대화를 읽기 전용("상대가 나감")으로. 이 종류를 모르는 상대(0.8.0)는 연결을
    /// 끊으므로 보내기 전에 `Pong.leave` 를 본다(모르면 보내지 않는다 — 그쪽은 보낼 때 `left` 를 받는다). 페어링된 상대만.
    Leave {
        conv: String,
    },
    /// 대화 기록 모두 지우기(#1x — 기록 삭제 › 모든 사용자 글 삭제) — 보낸 쪽이 그 대화 글을 비웠다. 받은 쪽도 그 대화 글을 비운다(대화는 남는다).
    /// 그 대화의 참여 상대가 보낸 것만 받는다(나간 대화 = `left`, 실행 중 = `retry`). 이 종류를 모르는 상대(1.0.1)는 연결을 끊으므로 보내기 전에
    /// `Pong.clear` 를 본다. 페어링된 상대만. `id` = 이 지우기의 uuid — 받는 쪽은 적용한 `id` 를 남겨 다시 온 같은 지우기를 두 번 적용하지 않는다.
    Clear {
        conv: String,
        id: String,
    },
}

/// 공유 대화 글(#1o) — `Post` 필드 관례. `owner` = 대화를 가진 PC(A) 쪽 글, `ok` 거짓 = 실패 글(없음 = 끝난 글), `at` = A 에 남은 시각(유닉스 초).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareMsg {
    pub uuid: String,
    pub author: String,
    pub text: String,
    #[serde(default)]
    pub lc: Option<i64>,
    #[serde(default)]
    pub at: i64,
    #[serde(default)]
    pub owner: bool,
    #[serde(default)]
    pub via: Option<String>,
    #[serde(default)]
    pub agent: Option<String>,
    #[serde(default)]
    pub ok: Option<bool>,
    #[serde(default)]
    pub edited: bool,
}

/// `Request::Context` 의 `op` 값(#1m).
pub const CTX_OPS: [&str; 5] = ["compact", "auto_compact", "clear", "done", "failed"];

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
        /// `Request::Context` 를 아는가(#1m) — 이전 버전은 싣지 않는다(거짓).
        #[serde(default)]
        ctx: bool,
        /// 공유 대화(`Request::Share*`, #1o)를 아는가 — 이전 버전은 싣지 않는다(거짓).
        #[serde(default)]
        share: bool,
        /// 대화 나가기(`Request::Leave`, #1w)를 아는가 — 이전 버전은 싣지 않는다(거짓).
        #[serde(default)]
        leave: bool,
        /// 대화 기록 모두 지우기(`Request::Clear`, #1x)를 아는가 — 이전 버전은 싣지 않는다(거짓).
        #[serde(default)]
        clear: bool,
    },
    Agents {
        names: Vec<String>,
    },
    /// `Status` 의 답 — `queued`(`ahead` = 앞 사람 수) · `running` · `done`(답을 보냈거나 보내는 중) · `unknown`.
    Status {
        state: String,
        #[serde(default)]
        ahead: u32,
    },
    /// `not_paired` · `unknown` · `stale`(이전 세대 승인) · `busy` · `closed`(페어링 받기 창이 닫혔거나 초대 코드 불일치) ·
    /// `bad_request` · `unknown_agent` · `unsupported`(받는 쪽에 처리기가 없음) · `focus`(받는 쪽이 집중 중 — 새 실행 요청 거부) ·
    /// `invalid`(대화 요청의 id 가 [`is_uuid`] 꼴이 아님) · `left`(받는 쪽이 그 대화를 나갔거나 지웠다, #1w)
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

/// 제어 문자(Cc)와 서식 문자(Cf — 방향 바꾸기 · 폭 없는 글자 등, 이름 위장)를 빼고 앞뒤 공백을 지운 뒤 `MAX_NAME_CHARS` 글자로
/// 자른다. 비면 `?`.
pub fn clean_name(s: &str) -> String {
    let kept: String = s
        .chars()
        .filter(|&c| !c.is_control() && !is_format(c))
        .collect();
    let cut: String = kept.trim().chars().take(MAX_NAME_CHARS).collect();
    if cut.is_empty() { "?".into() } else { cut }
}

/// 유니코드 일반 범주 Cf(서식 문자) — 표 출처 Unicode 15 `DerivedGeneralCategory`.
pub fn is_format(c: char) -> bool {
    matches!(c as u32,
        0x00AD | 0x0600..=0x0605 | 0x061C | 0x06DD | 0x070F | 0x0890..=0x0891 | 0x08E2 | 0x180E
        | 0x200B..=0x200F | 0x202A..=0x202E | 0x2060..=0x2064 | 0x2066..=0x206F | 0xFEFF
        | 0xFFF9..=0xFFFB | 0x110BD | 0x110CD | 0x13430..=0x1343F | 0x1BCA0..=0x1BCA3
        | 0x1D173..=0x1D17A | 0xE0001 | 0xE0020..=0xE007F)
}

/// 보낸 쪽이 만든 대화 · 글 id 꼴 — 소문자 hex 8-4-4-4-12(`an_agent::chat::new_uuid`).
pub fn is_uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| match i {
            8 | 13 | 18 | 23 => b == b'-',
            _ => b.is_ascii_digit() || (b'a'..=b'f').contains(&b),
        })
}

impl Request {
    /// 대화 요청의 id(`conv` · `msg` · `reply_to`)가 모두 [`is_uuid`] 꼴인가 — 처리기에 넘기기 전에 본다(원격 입력 총량 · 형식, 1a A8).
    /// 요청이 닿는 대화 uuid — 대화 없는 요청은 `None`.
    pub fn conv(&self) -> Option<&str> {
        match self {
            Request::Message { conv, .. }
            | Request::ReqState { conv, .. }
            | Request::Progress { conv, .. }
            | Request::Status { conv, .. }
            | Request::Edit { conv, .. }
            | Request::Delete { conv, .. }
            | Request::Context { conv, .. }
            | Request::Reply { conv, .. }
            | Request::Post { conv, .. }
            | Request::Relay { conv, .. }
            | Request::Share { conv, .. }
            | Request::ShareEnd { conv }
            | Request::ShareNotice { conv, .. }
            | Request::ShareDelete { conv, .. }
            | Request::SharePost { conv, .. }
            | Request::Leave { conv }
            | Request::Clear { conv, .. } => Some(conv),
            _ => None,
        }
    }

    pub fn ids_ok(&self) -> bool {
        match self {
            Request::Message { conv, msg, .. }
            | Request::ReqState { conv, msg, .. }
            | Request::Progress { conv, msg, .. }
            | Request::Status { conv, msg }
            | Request::Edit { conv, msg, .. }
            | Request::Delete { conv, msg }
            | Request::Context { conv, msg, .. } => is_uuid(conv) && is_uuid(msg),
            Request::Reply {
                conv,
                msg,
                reply_to,
                ..
            } => is_uuid(conv) && is_uuid(msg) && is_uuid(reply_to),
            Request::Post {
                conv,
                msg,
                reply_to,
                ..
            } => is_uuid(conv) && is_uuid(msg) && reply_to.as_deref().is_none_or(is_uuid),
            Request::Relay { conv, .. }
            | Request::Share { conv, .. }
            | Request::ShareEnd { conv }
            | Request::ShareNotice { conv, .. }
            | Request::Leave { conv } => is_uuid(conv),
            Request::Clear { conv, id } => is_uuid(conv) && is_uuid(id),
            Request::ShareDelete { conv, msg } => is_uuid(conv) && is_uuid(msg),
            Request::SharePost { conv, msg } => is_uuid(conv) && is_uuid(&msg.uuid),
            _ => true,
        }
    }
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
