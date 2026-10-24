//! 통합 검색.

use super::*;

/// 통합 검색 결과 — 묶음별.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    pub messages: Vec<Message>,
    pub conversations: Vec<Summary>,
    pub peers: Vec<crate::peers::Peer>,
    pub agents: Vec<FoundAgent>,
}

/// 검색에 걸린 에이전트 — `owner` 가 `None` 이면 내 에이전트, 아니면 그 상대(대화에 나온 이름).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundAgent {
    pub name: String,
    pub owner: Option<i64>,
}

/// 통합 검색의 메시지 밖 묶음 상한.
pub const SEARCH_OTHERS: usize = 10;

/// 닫힌 대화(웹UI `convClosed` 와 같은 규칙)의 SQL 식 — 내가 나감 · 상대가 나감 · 끝난 공유 사본. `c` = `conversations` 표 별칭.
fn closed_sql(c: &str) -> String {
    format!("({c}.left_at IS NOT NULL OR {c}.peer_left_at IS NOT NULL OR ({c}.dir = 'shared' AND coalesce({c}.share_ended, 0) = 1))")
}

/// 통합 검색 — 메시지 본문(양쪽 글, 최신 `limit` 건) · 대화 제목 · 사람(이름 · 표시 이름 · 호스트) · 에이전트(내 것의 이름 ·
/// 폴더, 대화에 나온 상대 것의 이름). 글자 그대로 부분 일치(영문은 대소문자 무시), 메시지 밖은 묶음마다 [`SEARCH_OTHERS`] 건.
/// `closed` 가 거짓이면 닫힌 대화([`closed_sql`])의 메시지 · 대화를 뺀다(웹UI "나간 대화 포함" 스위치, 지운 대화는 DB 에 없다).
pub fn search(conn: &Connection, q: &str, limit: i64, closed: bool) -> rusqlite::Result<Found> {
    let ql = q.to_lowercase();
    let has = |s: &str| s.to_lowercase().contains(&ql);
    let messages = search_messages(conn, q, limit, closed)?;
    let shut: std::collections::HashSet<i64> = if closed {
        Default::default()
    } else {
        conn.prepare(&format!("SELECT c.id FROM conversations c WHERE {}", closed_sql("c")))?
            .query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let conversations = summaries(conn)?
        .into_iter()
        .filter(|s| has(&s.title) && !shut.contains(&s.conversation.id))
        .take(SEARCH_OTHERS)
        .collect();
    let peers = crate::peers::list(conn)?
        .into_iter()
        .filter(|p| has(&p.name) || has(&p.host) || p.alias.as_deref().is_some_and(has))
        .take(SEARCH_OTHERS)
        .collect();
    let mut agents: Vec<FoundAgent> = crate::agents::list(conn)?
        .into_iter()
        .filter(|a| has(&a.name) || has(&a.folder))
        .map(|a| FoundAgent {
            name: a.name,
            owner: None,
        })
        .collect();
    let mut stmt = conn.prepare(
        "SELECT DISTINCT peer_id, agent FROM conversations
         WHERE dir = 'out' AND peer_id IN (SELECT id FROM peers) ORDER BY peer_id, agent",
    )?;
    for row in stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (peer, name) = row?;
        if has(&name) {
            agents.push(FoundAgent {
                name,
                owner: Some(peer),
            });
        }
    }
    agents.truncate(SEARCH_OTHERS);
    Ok(Found {
        messages,
        conversations,
        peers,
        agents,
    })
}

/// 검색의 메시지 묶음 — 본문 부분 일치(글자 그대로) · 최신 `limit` 건, `context` 글 제외. `closed` 가 거짓이면 닫힌 대화의 글을 뺀다.
fn search_messages(conn: &Connection, q: &str, limit: i64, closed: bool) -> rusqlite::Result<Vec<Message>> {
    conn.prepare(&format!(
        "SELECT {} FROM messages m JOIN conversations c ON c.id = m.conversation_id
         WHERE m.author <> 'context' AND m.text LIKE ?1 ESCAPE '\\' AND (?3 OR NOT {})
         ORDER BY m.created_at DESC, m.id DESC LIMIT ?2",
        prefixed(MSG_COLS, "m"),
        closed_sql("c")
    ))?
    .query_map(params![like_pattern(q), limit, closed], msg_row)?
    .collect()
}
