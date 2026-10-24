//! 내 에이전트 API — 목록 · 등록 · 고치기 · 지우기 · 상대 에이전트.

use super::*;

/// 도구는 받지 않는다(모든 실행이 탐색 도구 전부, 모르는 필드는 422).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct NewAgent {
    name: String,
    folder: String,
    #[serde(default)]
    prompt: String,
    /// 상위 한 단계 읽기 허용 — 없으면 끔.
    #[serde(default)]
    parent_read: bool,
    /// 셸 명령 실행 허용 — 없으면 끔.
    #[serde(default)]
    shell: bool,
    /// 실행 도구 — `agents::ENGINES` 중 하나, 필수(없으면 422) · 등록 뒤 바꾸지 않는다.
    engine: String,
    /// Codex `--skip-git-repo-check` — 없으면 켬.
    #[serde(default)]
    skip_git: Option<bool>,
}

/// `PATCH /api/agents/{id}` — 준 항목만 바꾼다. 이름은 바꾸지 않는다(대화 · 상대 PC 가 이름으로 부른다 — 주면 400).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AgentPatch {
    #[serde(default)]
    name: Option<Value>,
    /// 실행 도구는 바꾸지 않는다(주면 400).
    #[serde(default)]
    engine: Option<Value>,
    /// Codex `--skip-git-repo-check` — 세대 · 세션 그대로.
    #[serde(default)]
    skip_git: Option<bool>,
    #[serde(default)]
    folder: Option<String>,
    #[serde(default)]
    prompt: Option<String>,
    /// 빈 문자열 = 그룹 없음.
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    parent_read: Option<bool>,
    #[serde(default)]
    shell: Option<bool>,
    /// 아이콘 색 — `agents::ICON_COLORS` 밖이면 400.
    #[serde(default)]
    color: Option<String>,
}

/// `uid` = 등록 세대(같은 이름 · 같은 id 로 다시 등록해도 다름 — 웹UI 저장 전 값의 주인).
/// `parent_dir` = 상위 읽기로 실제 여는 폴더(끔 · 막힌 곳이면 `null` — 웹UI 가 생략 사유를 보인다).
pub(super) fn agent_json(a: Agent) -> Value {
    let up = a
        .parent_read
        .then(|| an_core::paths::parent_read_dir(std::path::Path::new(&a.folder)))
        .flatten()
        .map(|p| p.display().to_string());
    json!({ "id": a.id, "name": a.name, "folder": a.folder, "prompt": a.prompt, "group": a.group, "parent_read": a.parent_read, "parent_dir": up, "shell": a.shell, "color": a.color, "uid": a.uid, "engine": a.engine, "skip_git": a.skip_git })
}

/// 폴더는 있는 절대 경로만 — 상대 경로는 앱의 현재 폴더 기준이 되어 의도와 달라진다. 데이터 루트를 품거나 네트워크 경로면 거부
/// ([`an_core::paths::agent_folder_problem`]).
pub(super) fn checked_folder(folder: &str) -> Result<String, String> {
    let folder = std::path::Path::new(folder.trim());
    if !folder.is_absolute() || !folder.is_dir() {
        return Err("폴더는 이 PC 에 있는 절대 경로여야 한다".into());
    }
    if let Some(why) = an_core::paths::agent_folder_problem(folder) {
        return Err(why.into());
    }
    Ok(folder.display().to_string())
}

pub(super) async fn list_agents(State(chat): State<Arc<Chat>>) -> Response {
    match agents::list(&chat.db()) {
        Ok(list) => Json(list.into_iter().map(agent_json).collect::<Vec<_>>()).into_response(),
        Err(e) => db_fail(e),
    }
}

pub(super) async fn add_agent(State(chat): State<Arc<Chat>>, Json(req): Json<NewAgent>) -> Response {
    let name = req.name.trim();
    if name.is_empty()
        || name.chars().count() > MAX_AGENT_NAME
        || name.chars().any(char::is_control)
    {
        return bad("이름은 1 ~ 64자, 제어 문자 없이");
    }
    if !agents::ENGINES.contains(&req.engine.as_str()) {
        return bad(&format!("engine 은 {}", agents::ENGINES.join(" · ")));
    }
    if let Some(m) = codex_options(&req.engine, req.parent_read, req.shell) {
        return bad(m);
    }
    let folder = match checked_folder(&req.folder) {
        Ok(f) => f,
        Err(m) => return bad(&m),
    };
    let db = chat.db();
    if matches!(agents::by_name(&db, name), Ok(Some(_))) {
        return fail(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("같은 이름의 에이전트가 있다: {name}"),
        ));
    }
    let skip_git = req.skip_git.unwrap_or(true);
    match agents::add_with(&db, name, &folder, req.prompt.trim(), req.parent_read, req.shell, &req.engine, skip_git) {
        Ok(a) => (StatusCode::CREATED, Json(agent_json(a))).into_response(),
        Err(e) => db_fail(e),
    }
}

pub(super) async fn update_agent(
    State(chat): State<Arc<Chat>>,
    Path(id): Path<i64>,
    Json(req): Json<AgentPatch>,
) -> Response {
    if req.name.is_some() {
        return bad("에이전트 이름은 바꾸지 않는다 — 새 이름이면 새로 등록");
    }
    if req.engine.is_some() {
        return bad("실행 도구는 바꾸지 않는다 — 다른 도구면 새로 등록");
    }
    let db = chat.db();
    let now = match agents::by_id(&db, id) {
        Ok(Some(a)) => a,
        Ok(None) => return fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => return db_fail(e),
    };
    let folder = match req.folder.as_deref().map(checked_folder) {
        None => now.folder,
        Some(Ok(f)) => f,
        Some(Err(m)) => return bad(&m),
    };
    let prompt = req.prompt.map_or(now.prompt, |p| p.trim().to_string());
    let group = match req.group.as_deref().map(an_core::run_opts::clean_group) {
        None => now.group,
        Some(Ok(g)) => g,
        Some(Err(m)) => return bad(&m),
    };
    let parent_read = req.parent_read.unwrap_or(now.parent_read);
    let shell = req.shell.unwrap_or(now.shell);
    if let Some(m) = codex_options(&now.engine, parent_read, shell) {
        return bad(m);
    }
    if let Some(c) = req.color.as_deref()
        && !agents::ICON_COLORS.contains(&c)
    {
        return bad(&format!("color 는 {}", agents::ICON_COLORS.join(" · ")));
    }
    match agents::update(&db, id, &folder, &prompt, group.as_deref(), parent_read, shell) {
        Ok(Some(mut a)) => {
            if let Some(c) = req.color {
                if let Err(e) = agents::set_color(&db, id, &c) {
                    return db_fail(e);
                }
                a.color = c;
            }
            if let Some(on) = req.skip_git {
                if let Err(e) = agents::set_skip_git(&db, id, on) {
                    return db_fail(e);
                }
                a.skip_git = on;
            }
            Json(agent_json(a)).into_response()
        }
        Ok(None) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => db_fail(e),
    }
}

/// Codex 에이전트에 Claude 전용 옵션(상위 한단계 탐색 · 셸 명령 실행)을 켜면 그 사유(Codex 는 쓰지 않는다).
pub(super) fn codex_options(engine: &str, parent_read: bool, shell: bool) -> Option<&'static str> {
    (engine == agents::CODEX && (parent_read || shell))
        .then_some("Codex 에이전트는 상위 한단계 탐색 · 셸 명령 실행 허용을 쓰지 않는다")
}

pub(super) async fn remove_agent(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match agents::remove(&chat.db(), id) {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => fail(io::Error::new(io::ErrorKind::NotFound, "없는 에이전트")),
        Err(e) => db_fail(e),
    }
}

pub(super) async fn remote_agents(State(chat): State<Arc<Chat>>, Path(id): Path<i64>) -> Response {
    match chat.remote_agents(id).await {
        Ok(names) => Json(json!({ "names": names })).into_response(),
        Err(e) => fail(e),
    }
}
