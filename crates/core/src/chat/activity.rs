//! 내 활동 — 받은 요청 · 받은 답 · 실패한 내 글.

use super::*;

/// `MSG_COLS` 열 수 — 그 뒤에 붙는 열의 위치.
const MSG_N: usize = 28;

/// 내 활동 한 줄의 종류 — 상대 쪽 글(받은 요청 · 받은 답) · 내 쪽 실패 · 내 요청 · 상대 반영 실패.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityKind {
    Message,
    Failed,
    MyRequest,
    SyncFailed,
}

impl ActivityKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ActivityKind::Message => "message",
            ActivityKind::Failed => "failed",
            ActivityKind::MyRequest => "my_request",
            ActivityKind::SyncFailed => "sync_failed",
        }
    }
}

/// 내 활동 거르기 — 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivityFilter {
    All,
    Request,
    MyRequest,
    Answer,
    Failed,
}

/// 내 활동 한 줄.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub kind: ActivityKind,
    /// 남은 시각(유닉스 초) — 글이면 저장 시각, 내 요청이면 마지막으로 상태가 바뀐 시각.
    pub at: i64,
    pub conversation_id: i64,
    pub peer_id: i64,
    /// 글(받은 요청 · 받은 답 · 내 요청 · 실패한 내 글) — 상대 반영 실패 기록이면 `None`.
    pub message: Option<Message>,
    /// 상대 반영 실패 기록의 사유 · 미리보기.
    pub detail: Option<String>,
    /// 그 상대의 내 활동을 본 마지막 글 · 기록(id) 뒤의 것(지운 상대의 것은 본 것으로 — 본 표시를 둘 곳이 없다).
    pub unseen: bool,
}

/// 받은 요청 — 상대 사람의 요청(공유 사본 글은 빼고). `messages m` · `conversations c` 문맥의 SQL 식.
const RECV_REQUEST: &str =
    "(m.mine = 0 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND c.dir <> 'shared')";

/// 받은 답 — 내 사람 글에 단 상대 에이전트 답(자동 왕복에서 내 에이전트 글에 단 답은 아니다).
const RECV_ANSWER: &str = "(m.mine = 0 AND m.author = 'agent' AND c.dir <> 'shared'
   AND EXISTS (SELECT 1 FROM messages q WHERE q.id = m.reply_to AND q.mine = 1 AND q.author = 'human'))";

/// 실패한 내 글.
const MINE_FAILED: &str = "(m.mine = 1 AND m.state = 'error' AND m.author <> 'context')";

/// 내 요청 — 실패한 것은 '실패' 로만.
const MY_REQUEST: &str =
    "(m.mine = 1 AND m.author = 'human' AND coalesce(m.request, 0) = 1 AND m.state <> 'error')";

/// 내 활동에 드는 글 중 글 id 로 안 본 것을 가르는 것 — 받은 요청 · 받은 답 · 실패한 내 글(상대 사람 일반 메시지는 빠진다).
fn activity_messages() -> String {
    format!("({RECV_REQUEST} OR {RECV_ANSWER} OR {MINE_FAILED})")
}

/// 내 활동 — 최신 것부터 `limit` 건. `peer` 로 그 상대 대화만, `q` 는 글 본문 · 상대 반영 실패 사유의 부분 일치.
pub fn activity(
    conn: &Connection,
    peer: Option<i64>,
    filter: ActivityFilter,
    q: Option<&str>,
    limit: i64,
) -> rusqlite::Result<Vec<Activity>> {
    let like = q.map(like_pattern);
    let mut out = Vec::new();
    let cols = prefixed(MSG_COLS, "m");
    let msg_where = match filter {
        ActivityFilter::All => Some(activity_messages()),
        ActivityFilter::Request => Some(RECV_REQUEST.to_string()),
        ActivityFilter::Answer => Some(RECV_ANSWER.to_string()),
        ActivityFilter::Failed => Some(MINE_FAILED.to_string()),
        ActivityFilter::MyRequest => None,
    };
    if let Some(w) = msg_where {
        let mut stmt = conn.prepare(&format!(
            "SELECT {cols}, c.peer_id, p.id IS NOT NULL AND (m.id > coalesce(p.activity_seen_msg, 0)
               OR EXISTS (SELECT 1 FROM relay_events f WHERE f.message_id = m.id AND f.kind = 'failed'
                 AND f.id > coalesce(p.activity_seen_event, 0)))
             FROM messages m JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE {w} AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR m.text LIKE ?2 ESCAPE '\\')
             ORDER BY m.created_at DESC, m.id DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            let m = msg_row(r)?;
            Ok(Activity {
                kind: if m.mine {
                    ActivityKind::Failed
                } else {
                    ActivityKind::Message
                },
                at: m.created_at,
                conversation_id: m.conversation_id,
                peer_id: r.get(MSG_N)?,
                unseen: r.get(MSG_N + 1)?,
                message: Some(m),
                detail: None,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    // 내 요청 — 요청 하나 = 한 줄, 시각 = 마지막으로 상태가 바뀐 때, 안 본 것 = 본 뒤 상태가 바뀐 기록이 있을 때.
    if matches!(filter, ActivityFilter::All | ActivityFilter::MyRequest) {
        let mut stmt = conn.prepare(&format!(
            "SELECT {cols}, c.peer_id, p.id IS NOT NULL AND EXISTS (SELECT 1 FROM relay_events f WHERE f.message_id = m.id
               AND f.kind = 'req_state' AND f.id > coalesce(p.activity_seen_event, 0)), coalesce(m.req_at, m.created_at) AS at
             FROM messages m JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE {MY_REQUEST} AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR m.text LIKE ?2 ESCAPE '\\')
             ORDER BY at DESC, m.id DESC LIMIT ?3"
        ))?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            let m = msg_row(r)?;
            Ok(Activity {
                kind: ActivityKind::MyRequest,
                at: r.get(MSG_N + 2)?,
                conversation_id: m.conversation_id,
                peer_id: r.get(MSG_N)?,
                unseen: r.get(MSG_N + 1)?,
                message: Some(m),
                detail: None,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    // 상대 반영 실패 기록(실패 거르기에도).
    if matches!(filter, ActivityFilter::All | ActivityFilter::Failed) {
        let mut stmt = conn.prepare(
            "SELECT e.created_at, e.conversation_id, c.peer_id, e.detail,
               p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0)
             FROM relay_events e JOIN conversations c ON c.id = e.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
             WHERE e.kind = 'sync_failed' AND (?1 IS NULL OR c.peer_id = ?1) AND (?2 IS NULL OR e.detail LIKE ?2 ESCAPE '\\')
             ORDER BY e.created_at DESC, e.id DESC LIMIT ?3",
        )?;
        let rows = stmt.query_map(params![peer, like, limit], |r| {
            Ok(Activity {
                kind: ActivityKind::SyncFailed,
                at: r.get(0)?,
                conversation_id: r.get(1)?,
                peer_id: r.get(2)?,
                message: None,
                detail: Some(r.get(3)?),
                unseen: r.get(4)?,
            })
        })?;
        for a in rows {
            out.push(a?);
        }
    }
    // 같은 초면 글이 먼저(안정 정렬) — 목록 각각은 이미 최신순.
    out.sort_by_key(|a| std::cmp::Reverse(a.at));
    out.truncate(usize::try_from(limit).unwrap_or(0));
    Ok(out)
}

/// 내 활동 안 본 수 — `peer` 가 있으면 그 상대만(지운 상대의 것은 세지 않는다). 본 뒤 온 받은 요청 · 받은 답 · 실패 글 + 본 뒤 실패로 바뀐 내 글
/// (이미 본 id) + 본 뒤 상대 반영 실패 기록 + 본 뒤 상태가 바뀐 내 요청(요청마다 하나). 가장 앞선 본 표시 뒤의 id 만 훑는다(5초 새로 고침).
pub fn activity_unseen(conn: &Connection, peer: Option<i64>) -> rusqlite::Result<i64> {
    let messages = activity_messages();
    conn.query_row(
        &format!(
            "SELECT
               (SELECT count(*) FROM messages m JOIN conversations c ON c.id = m.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE m.id > coalesce((SELECT min(coalesce(activity_seen_msg, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND {messages} AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND m.id > coalesce(p.activity_seen_msg, 0))
             + (SELECT count(DISTINCT m.id) FROM relay_events f JOIN messages m ON m.id = f.message_id
                  JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
                WHERE f.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND f.kind = 'failed' AND m.state = 'error' AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND f.id > coalesce(p.activity_seen_event, 0) AND m.id <= coalesce(p.activity_seen_msg, 0))
             + (SELECT count(*) FROM relay_events e JOIN conversations c ON c.id = e.conversation_id
                  LEFT JOIN peers p ON p.id = c.peer_id
                WHERE e.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND e.kind = 'sync_failed' AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND e.id > coalesce(p.activity_seen_event, 0))
             + (SELECT count(DISTINCT m.id) FROM relay_events f JOIN messages m ON m.id = f.message_id
                  JOIN conversations c ON c.id = m.conversation_id LEFT JOIN peers p ON p.id = c.peer_id
                WHERE f.id > coalesce((SELECT min(coalesce(activity_seen_event, 0)) FROM peers WHERE ?1 IS NULL OR id = ?1), 0)
                  AND f.kind = 'req_state' AND {MY_REQUEST} AND (?1 IS NULL OR c.peer_id = ?1)
                  AND p.id IS NOT NULL AND f.id > coalesce(p.activity_seen_event, 0))"
        ),
        params![peer],
        |r| r.get(0),
    )
}

/// 지금 마지막 글 · 자동 왕복 기록 id — 내 활동 화면이 받은 범위(본 표시는 이 범위까지만 보낸다).
pub fn activity_upto(conn: &Connection) -> rusqlite::Result<(i64, i64)> {
    conn.query_row(
        "SELECT (SELECT coalesce(max(id), 0) FROM messages), (SELECT coalesce(max(id), 0) FROM relay_events)",
        [],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
}

/// 내 활동을 `(message, event)` 까지 봤다(이 PC, [`activity_upto`] 로 받은 범위) — `peer` 가 있으면 그 상대만, 없으면 전부. 본 표시는
/// 뒤로 가지 않고 지금 마지막 항목을 넘지 않는다(앞으로 올 것을 미리 본 것으로 하지 않게). 돌려주는 값: 바꾼 상대 수.
pub fn mark_activity_seen(
    conn: &Connection,
    peer: Option<i64>,
    (message, event): (i64, i64),
) -> rusqlite::Result<usize> {
    conn.execute(
        "UPDATE peers SET
           activity_seen_msg = max(coalesce(activity_seen_msg, 0), min(?2, (SELECT coalesce(max(id), 0) FROM messages))),
           activity_seen_event = max(coalesce(activity_seen_event, 0), min(?3, (SELECT coalesce(max(id), 0) FROM relay_events)))
         WHERE ?1 IS NULL OR id = ?1",
        params![peer, message, event],
    )
}
