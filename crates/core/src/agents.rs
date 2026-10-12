//! 에이전트 프로필(D21) — 이 PC 사용자가 등록한 것만 원격이 부를 수 있다.
use rusqlite::{Connection, OptionalExtension, Row, params};

/// 새 프로필의 허용 도구 — 읽기 전용(D22 보강).
pub const DEFAULT_TOOLS: &str = "Read,Grep,Glob";

/// 프로필이 고를 수 있는 도구 — 읽기 전용만(D31: 원격 요청은 문의 응답, 수신 PC 작업에 개입 · 편집 · 삭제 금지).
pub const READ_ONLY_TOOLS: [&str; 5] = ["Read", "Grep", "Glob", "LS", "NotebookRead"];

fn tool_names(tools: &str) -> impl Iterator<Item = &str> {
    tools.split(',').map(str::trim).filter(|t| !t.is_empty())
}

/// 읽기 전용 목록 밖 도구(등록 거부용).
pub fn unknown_tools(tools: &str) -> Vec<String> {
    tool_names(tools)
        .filter(|t| !READ_ONLY_TOOLS.contains(t))
        .map(str::to_string)
        .collect()
}

/// 실행에 넘길 도구 — 목록 안 것만(이전에 저장된 쓰기 도구도 빠진다), 남는 것이 없으면 [`DEFAULT_TOOLS`].
pub fn read_only(tools: &str) -> String {
    let kept: Vec<&str> = tool_names(tools)
        .filter(|t| READ_ONLY_TOOLS.contains(t))
        .collect();
    if kept.is_empty() {
        DEFAULT_TOOLS.to_string()
    } else {
        kept.join(",")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Agent {
    pub id: i64,
    pub name: String,
    /// `claude -p` 의 실행 폴더 — 원격으로 나갈 수 있는 범위.
    pub folder: String,
    /// 쉼표로 이은 도구 이름(`--tools` · `--allowedTools`).
    pub tools: String,
    /// 추가 지시문(`--append-system-prompt`), 비면 없음.
    pub prompt: String,
    /// 사용자가 붙인 그룹 이름(없으면 `None`).
    pub group: Option<String>,
    /// 세대 — 등록 · 폴더 변경 때마다 새 값(다시 쓰지 않는다). 끝난 실행은 지금 세대가 실행 때와 같을 때만 세션을 남긴다.
    pub uid: String,
}

const COLS: &str = "id, name, folder, tools, prompt, grp, coalesce(uid, '')";

fn row(r: &Row) -> rusqlite::Result<Agent> {
    Ok(Agent {
        id: r.get(0)?,
        name: r.get(1)?,
        folder: r.get(2)?,
        tools: r.get(3)?,
        prompt: r.get(4)?,
        group: r.get(5)?,
        uid: r.get(6)?,
    })
}

/// `tools` 가 `None` 이면 [`DEFAULT_TOOLS`]. 이름이 겹치면 오류.
pub fn add(
    conn: &Connection,
    name: &str,
    folder: &str,
    tools: Option<&str>,
    prompt: &str,
) -> rusqlite::Result<Agent> {
    conn.execute(
        "INSERT INTO agents (name, folder, tools, prompt, created_at, uid)
         VALUES (?1, ?2, ?3, ?4, unixepoch(), lower(hex(randomblob(16))))",
        params![name, folder, tools.unwrap_or(DEFAULT_TOOLS), prompt],
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

/// 폴더 · 도구 · 추가 지시 · 그룹을 바꾼다 — 이름은 바꾸지 않는다(대화 · 상대 PC 가 이름으로 부른다). 없는 id 면 `None`.
/// 폴더가 바뀌면 새 세대(`uid`)로 하고 이 에이전트가 이어 가던 세션(받은 대화 · 내가 연 자동 왕복 · 멘션)을 비운다 — Claude Code 세션은 폴더에
/// 묶여 있어 새 폴더에서 `--resume` 하면 찾지 못한다. 다음 실행은 새 세션.
pub fn update(
    conn: &Connection,
    id: i64,
    folder: &str,
    tools: &str,
    prompt: &str,
    group: Option<&str>,
) -> rusqlite::Result<Option<Agent>> {
    let tx = conn.unchecked_transaction()?;
    let Some(now) = by_id(&tx, id)? else {
        return Ok(None);
    };
    tx.execute(
        "UPDATE agents SET folder = ?2, tools = ?3, prompt = ?4, grp = ?5 WHERE id = ?1",
        params![id, folder, tools, prompt, group],
    )?;
    if now.folder != folder {
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
    let tx = conn.unchecked_transaction()?;
    let Some(gone) = by_id(&tx, id)? else {
        return Ok(false);
    };
    tx.execute("DELETE FROM agents WHERE id = ?1", [id])?;
    forget_sessions(&tx, &gone.name)?;
    tx.commit()?;
    Ok(true)
}
