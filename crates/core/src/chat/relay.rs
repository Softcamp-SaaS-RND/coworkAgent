//! 자동 왕복 표시.

use super::*;

/// 자동 왕복 기록 종류 — 같은 표에 상대 반영 실패도 남는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayMark {
    Start,
    End,
    /// 편집 · 삭제를 상대 창에 반영하지 못했다(상대 거절 · 24시간 포기). 편집이면 그 글 id — 글을 지우면 기록(본문 미리보기)도 함께 지운다.
    SyncFailed(Option<i64>),
}

/// 자동 왕복 시작 · 끝을 남긴다(내 활동) — 알림 설정과 무관하게. `detail` = 끝 사유 · 시작 글 미리보기. 같은 표의 `failed` 행은 내 글이 실패로
/// 바뀐 순간([`set_state`] · [`set_flags`] 가 남긴다 — 목록에는 그 글로 보인다).
pub fn add_relay_event(
    conn: &Connection,
    conversation_id: i64,
    mark: RelayMark,
    detail: &str,
) -> rusqlite::Result<()> {
    let (kind, message) = match mark {
        RelayMark::Start => ("start", None),
        RelayMark::End => ("end", None),
        RelayMark::SyncFailed(message) => ("sync_failed", message),
    };
    conn.execute(
        "INSERT INTO relay_events (conversation_id, kind, detail, created_at, message_id) VALUES (?1, ?2, ?3, unixepoch(), ?4)",
        params![conversation_id, kind, detail, message],
    )
    .map(drop)
}

/// 자동 왕복을 켠다(연 쪽, 새 대화) — 카운트 0.
pub fn set_relay(conn: &Connection, id: i64, agent: &str, cap: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_agent = ?2, relay_cap = ?3, relay_count = 0, relay_on = 1 WHERE id = ?1",
        params![id, agent, cap],
    )
    .map(drop)
}

/// 받은 쪽: 자동 왕복 대화로 기록 — 상대(연 쪽) 에이전트 이름만(상한 · 켜짐은 연 쪽 것).
pub fn mark_relayed(conn: &Connection, id: i64, agent: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_agent = ?2 WHERE id = ?1 AND dir = 'in' AND relay_agent IS NULL",
        params![id, agent],
    )
    .map(drop)
}

/// 자동 왕복 켜기 · 끄기(연 쪽). `reset` 이면 카운트 0(재개).
pub fn relay_on(conn: &Connection, id: i64, on: bool, reset: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_on = ?2, relay_count = CASE WHEN ?3 THEN 0 ELSE relay_count END
         WHERE id = ?1 AND relay_agent IS NOT NULL",
        params![id, on, reset],
    )
    .map(drop)
}

/// 왕복 하나를 센다 — 돌려주는 값: 센 뒤 카운트.
pub fn relay_step(conn: &Connection, id: i64) -> rusqlite::Result<i64> {
    conn.execute(
        "UPDATE conversations SET relay_count = coalesce(relay_count, 0) + 1 WHERE id = ?1",
        [id],
    )?;
    conn.query_row(
        "SELECT relay_count FROM conversations WHERE id = ?1",
        [id],
        |r| r.get(0),
    )
}

/// 받은 쪽: 에이전트발 실행 요청 거절 켜기 · 끄기.
pub fn set_blocked(conn: &Connection, id: i64, blocked: bool) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET relay_blocked = ?2 WHERE id = ?1",
        params![id, blocked],
    )
    .map(drop)
}
