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
}

const COLS: &str = "id, name, folder, tools, prompt";

fn row(r: &Row) -> rusqlite::Result<Agent> {
    Ok(Agent {
        id: r.get(0)?,
        name: r.get(1)?,
        folder: r.get(2)?,
        tools: r.get(3)?,
        prompt: r.get(4)?,
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
        "INSERT INTO agents (name, folder, tools, prompt, created_at) VALUES (?1, ?2, ?3, ?4, unixepoch())",
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

pub fn remove(conn: &Connection, id: i64) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM agents WHERE id = ?1", [id])? == 1)
}
