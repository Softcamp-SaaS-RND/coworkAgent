//! 에이전트 프로필(D21) — 이 PC 사용자가 등록한 것만 원격이 부를 수 있다.
use rusqlite::{Connection, OptionalExtension, Row, params};

/// 모든 실행의 도구 — 탐색(읽기 전용) 전부, 프로필마다 고르지 않는다(D31 · #1d: 원격 요청은 문의 응답, 수신 PC 작업에 개입 · 편집 · 삭제 금지).
pub const READ_ONLY_TOOLS: [&str; 5] = ["Read", "Grep", "Glob", "LS", "NotebookRead"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: i64,
    pub name: String,
    /// `claude -p` 의 실행 폴더 — 원격으로 나갈 수 있는 범위.
    pub folder: String,
    /// 추가 지시문(`--append-system-prompt`), 비면 없음.
    pub prompt: String,
    /// 사용자가 붙인 그룹 이름(없으면 `None`).
    pub group: Option<String>,
    /// 상위 한 단계 읽기 허용 — 참이면 실행이 바로 위 폴더도 `--add-dir` 로 연다([`crate::paths::parent_read_dir`] 가 막는 곳이면 폴더만).
    pub parent_read: bool,
    /// 세대 — 등록 · 폴더 · 상위 읽기 변경 때마다 새 값(다시 쓰지 않는다). 끝난 실행은 지금 세대가 실행 때와 같을 때만 세션을 남긴다.
    pub uid: String,
}

const COLS: &str = "id, name, folder, prompt, grp, coalesce(uid, ''), parent_read";

fn row(r: &Row) -> rusqlite::Result<Agent> {
    Ok(Agent {
        id: r.get(0)?,
        name: r.get(1)?,
        folder: r.get(2)?,
        prompt: r.get(3)?,
        group: r.get(4)?,
        uid: r.get(5)?,
        parent_read: r.get(6)?,
    })
}

/// 이름이 겹치면 오류. `agents.tools` 열은 읽지 않는다(#1d — 도구는 [`READ_ONLY_TOOLS`] 고정). 상위 읽기 끔 — [`add_with`].
pub fn add(conn: &Connection, name: &str, folder: &str, prompt: &str) -> rusqlite::Result<Agent> {
    add_with(conn, name, folder, prompt, false)
}

/// [`add`] + 상위 한 단계 읽기 허용(`parent_read`).
pub fn add_with(conn: &Connection, name: &str, folder: &str, prompt: &str, parent_read: bool) -> rusqlite::Result<Agent> {
    conn.execute(
        "INSERT INTO agents (name, folder, tools, prompt, created_at, uid, parent_read)
         VALUES (?1, ?2, ?3, ?4, unixepoch(), lower(hex(randomblob(16))), ?5)",
        params![name, folder, READ_ONLY_TOOLS.join(","), prompt, parent_read],
    )?;
    by_name(conn, name).map(|a| a.expect("방금 쓴 프로필"))
}

pub fn by_name(conn: &Connection, name: &str) -> rusqlite::Result<Option<Agent>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM agents WHERE name = ?1"),
        [name],
        row,
    )
    .optional()
}

pub fn list(conn: &Connection) -> rusqlite::Result<Vec<Agent>> {
    conn.prepare(&format!("SELECT {COLS} FROM agents ORDER BY id"))?
        .query_map([], row)?
        .collect()
}

/// 폴더 · 추가 지시 · 그룹 · 상위 읽기를 바꾼다 — 이름은 바꾸지 않는다(대화 · 상대 PC 가 이름으로 부른다). 없는 id 면 `None`.
/// 폴더나 상위 읽기가 바뀌면 새 세대(`uid`)로 하고 이 에이전트가 이어 가던 세션(받은 대화 · 내가 연 자동 왕복 · 멘션)을 비운다 — Claude Code 세션은 폴더에
/// 묶여 있어 새 폴더에서 `--resume` 하면 찾지 못한다 · 상위 읽기를 끈 뒤 위 폴더 내용이 든 세션 · 요약을 잇지 않게. 다음 실행은 새 세션.
pub fn update(
    conn: &Connection,
    id: i64,
    folder: &str,
    prompt: &str,
    group: Option<&str>,
    parent_read: bool,
) -> rusqlite::Result<Option<Agent>> {
    let tx = crate::db::write_tx(conn)?;
    let Some(now) = by_id(&tx, id)? else {
        return Ok(None);
    };
    tx.execute(
        "UPDATE agents SET folder = ?2, prompt = ?3, grp = ?4, parent_read = ?5 WHERE id = ?1",
        params![id, folder, prompt, group, parent_read],
    )?;
    if now.folder != folder || now.parent_read != parent_read {
        tx.execute(
            "UPDATE agents SET uid = lower(hex(randomblob(16))) WHERE id = ?1",
            [id],
        )?;
        forget_sessions(&tx, &now.name)?;
    }
    tx.commit()?;
    by_id(conn, id)
}

/// 이 이름의 에이전트가 이어 가던 Claude 세션을 비운다 — 받은 대화 · 내가 연 자동 왕복 · 멘션(`agent_sessions`). 다음 실행은 새 세션.
pub fn forget_sessions(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE conversations SET session = NULL
         WHERE (dir = 'in' AND agent = ?1) OR (dir = 'out' AND relay_agent = ?1)",
        [name],
    )?;
    conn.execute("DELETE FROM agent_sessions WHERE agent = ?1", [name])?;
    // 압축 요약(#1m)도 — 옛 폴더의 사실을 새 세대 첫 실행에 넣지 않게(라운드 1).
    conn.execute(
        "DELETE FROM ctx_seeds WHERE agent = ?1 OR (agent = '' AND conversation_id IN
           (SELECT id FROM conversations WHERE (dir = 'in' AND agent = ?1) OR (dir = 'out' AND relay_agent = ?1)))",
        [name],
    )?;
    Ok(())
}

pub fn by_id(conn: &Connection, id: i64) -> rusqlite::Result<Option<Agent>> {
    conn.query_row(
        &format!("SELECT {COLS} FROM agents WHERE id = ?1"),
        [id],
        row,
    )
    .optional()
}

/// 지운다 — 그 에이전트의 세션도 비운다(같은 이름을 다른 폴더로 다시 등록해도 옛 세션을 `--resume` 하지 않게).
pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    let tx = crate::db::write_tx(conn)?;
    let Some(gone) = by_id(&tx, id)? else {
        return Ok(false);
    };
    tx.execute("DELETE FROM agents WHERE id = ?1", [id])?;
    forget_sessions(&tx, &gone.name)?;
    tx.commit()?;
    Ok(true)
}
