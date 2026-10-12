//! 인바운드 알림(S9) — 기본 OFF(D29). 두 OS 같은 UI/UX(D30): 웹UI 탭이 열려 있으면 브라우저 알림(이벤트 스트림 →
//! `Notification`, OS 알림 정책은 브라우저 · OS 가 따른다 — D27), 탭과 무관하게 트레이 "새 요청" 표시.
use an_agent::chat::{MAX_RELAY_CAP, RELAY_CAP_KEY, default_relay_cap};
use an_core::db::Connection;
use an_core::run_opts;
use an_core::settings;
use an_net::node::{MessageKind, PeerEvent};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use std::io;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

/// 설정 키 — 값 `1` 이면 켜짐.
pub const NOTIFY_KEY: &str = "notify_inbound";
/// 이벤트 스트림 버퍼 — 넘치면 뒤처진 구독자가 오래된 것을 놓친다(보내는 쪽은 막히지 않는다).
pub const BACKLOG: usize = 64;

/// 웹UI 로 가는 알림 한 건. 원격 이름이 들어 있다 — 웹UI 는 텍스트로만 쓴다.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Notice {
    /// `pair_request` · `approved`
    pub kind: &'static str,
    pub title: String,
    pub body: String,
}

type Attention = Box<dyn Fn(usize) + Send + Sync>;

/// 자동 왕복 알림 — 시작 · 끝만(기본) / 글마다 / 없음.
pub const RELAY_MODES: [&str; 3] = ["start_end", "every", "off"];

/// 알림 세부 설정 — 전체 켜짐(`notify_inbound`) 아래에서 거른다. 설정 키 = 필드 이름(값 `1`/`0` · 모드 문자열).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Prefs {
    /// 페어링 요청 · 승인.
    pub notify_pair: bool,
    /// 받은 요청(사람 글).
    pub notify_request: bool,
    /// 받은 답.
    pub notify_reply: bool,
    /// `RELAY_MODES` 중 하나.
    pub notify_relay: String,
    /// 끄면 알림 본문에 글 앞부분을 넣지 않는다.
    pub notify_preview: bool,
    /// 알림 일시 중지(기본 꺼짐) — 켜 있는 동안 브라우저 알림 · 트레이 표시 · 이벤트를 내지 않는다. `notify_inbound` 와 별개라 풀면 그 설정대로.
    pub notify_paused: bool,
    /// 집중(기본 꺼짐, 설정 키 `focus`) — 상대의 새 실행 요청 거부와 함께 알림도 멈춘다. 일시 중지 값은 바꾸지 않는다.
    pub focus: bool,
}

impl Prefs {
    fn load(db: &Connection) -> io::Result<Self> {
        let flag = |key: &str| -> io::Result<bool> {
            Ok(settings::get(db, key).map_err(db_err)?.as_deref() != Some("0"))
        };
        let relay = settings::get(db, "notify_relay")
            .map_err(db_err)?
            .filter(|v| RELAY_MODES.contains(&v.as_str()))
            .unwrap_or_else(|| RELAY_MODES[0].to_string());
        Ok(Self {
            notify_pair: flag("notify_pair")?,
            notify_request: flag("notify_request")?,
            notify_reply: flag("notify_reply")?,
            notify_relay: relay,
            notify_preview: flag("notify_preview")?,
            notify_paused: settings::get(db, "notify_paused")
                .map_err(db_err)?
                .as_deref()
                == Some("1"),
            focus: settings::focus(db).map_err(db_err)?,
        })
    }

    /// 이 이벤트를 알리나.
    fn allows(&self, e: &PeerEvent) -> bool {
        if self.notify_paused || self.focus {
            return false;
        }
        match e {
            PeerEvent::PairRequest { .. }
            | PeerEvent::Approved { .. }
            | PeerEvent::AutoPaired { .. } => self.notify_pair,
            PeerEvent::Message { kind, .. } => match kind {
                MessageKind::Request => self.notify_request,
                MessageKind::Reply => self.notify_reply,
                MessageKind::RelayStart | MessageKind::RelayEnd => self.notify_relay != "off",
                MessageKind::RelayStep => self.notify_relay == "every",
            },
        }
    }
}

/// 켜짐 · 세부 설정과 건수는 한 잠금 — 켜짐 확인 · 건수 변경 · 표시 갱신 · 알림 전송 · 끄기가 한 순서로 직렬화된다.
struct Status {
    enabled: bool,
    prefs: Prefs,
    /// 사용자가 아직 보지 않은 알림 수 — 트레이 표시.
    pending: usize,
}

pub struct Notifier {
    status: Mutex<Status>,
    db: Mutex<Connection>,
    tx: broadcast::Sender<Notice>,
    /// 트레이 표시 갱신(건수, 0 = 지움). 막히면 안 된다 — 운영은 이벤트 루프 프록시로 보낸다.
    attention: Attention,
}

fn db_err(e: an_core::db::Error) -> io::Error {
    io::Error::other(e)
}

impl Notifier {
    /// 켜짐 여부는 DB 설정에서 읽는다(없으면 OFF).
    pub fn new(
        db: Connection,
        attention: impl Fn(usize) + Send + Sync + 'static,
    ) -> io::Result<Arc<Self>> {
        let on = settings::get(&db, NOTIFY_KEY).map_err(db_err)?.as_deref() == Some("1");
        let prefs = Prefs::load(&db)?;
        Ok(Arc::new(Self {
            status: Mutex::new(Status {
                enabled: on,
                prefs,
                pending: 0,
            }),
            db: Mutex::new(db),
            tx: broadcast::channel(BACKLOG).0,
            attention: Box::new(attention),
        }))
    }

    fn status(&self) -> std::sync::MutexGuard<'_, Status> {
        self.status.lock().expect("알림 상태 잠금이 오염됐다")
    }

    pub fn enabled(&self) -> bool {
        self.status().enabled
    }

    /// DB 에 저장한 뒤 반영한다. 끄면 트레이 표시도 지운다 — 상태 잠금 안에서라 그 사이 이벤트가 끼어들지 못한다.
    pub fn set_enabled(&self, on: bool) -> io::Result<()> {
        let mut st = self.status();
        {
            let db = self.db.lock().expect("설정 DB 잠금이 오염됐다");
            settings::set(&db, NOTIFY_KEY, if on { "1" } else { "0" }).map_err(db_err)?;
        }
        st.enabled = on;
        if !on {
            st.pending = 0;
            (self.attention)(0);
        }
        Ok(())
    }

    pub fn prefs(&self) -> Prefs {
        self.status().prefs.clone()
    }

    /// 세부 설정 · 자동 왕복 상한 기본값을 바꾼다 — 하나라도 틀리면 아무것도 바꾸지 않고 `InvalidInput`, 저장은 한 트랜잭션.
    pub fn update(&self, patch: &SettingsPatch) -> io::Result<()> {
        let invalid = |m: &str| io::Error::new(io::ErrorKind::InvalidInput, m.to_string());
        if let Some(m) = &patch.notify_relay
            && !RELAY_MODES.contains(&m.as_str())
        {
            return Err(invalid("notify_relay 는 start_end · every · off"));
        }
        if let Some(c) = patch.relay_cap_default
            && !(1..=MAX_RELAY_CAP).contains(&c)
        {
            return Err(invalid(&format!(
                "relay_cap_default 는 1 ~ {MAX_RELAY_CAP}"
            )));
        }
        if patch
            .claude_model
            .as_deref()
            .is_some_and(|m| !run_opts::valid_model(m))
        {
            return Err(invalid(
                "claude_model 은 1 ~ 64자 영문 · 숫자 · ._:[]- (- 로 시작하지 않음)",
            ));
        }
        if patch
            .claude_effort
            .as_deref()
            .is_some_and(|e| !run_opts::valid_effort(e))
        {
            return Err(invalid(&format!(
                "claude_effort 는 {}",
                run_opts::EFFORTS.join(" · ")
            )));
        }
        if patch
            .ui_theme
            .as_deref()
            .is_some_and(|t| !run_opts::THEMES.contains(&t))
        {
            return Err(invalid("ui_theme 은 light · dark"));
        }
        if let Some(w) = patch.ui_side_width
            && !run_opts::SIDE_WIDTHS.contains(&w)
        {
            return Err(invalid(&format!(
                "ui_side_width 는 {} ~ {}",
                run_opts::SIDE_WIDTHS.start(),
                run_opts::SIDE_WIDTHS.end()
            )));
        }
        let me_alias = patch
            .me_alias
            .as_deref()
            .map(run_opts::clean_alias)
            .transpose()
            .map_err(|m| invalid(&m))?;
        let me_icon = patch
            .me_icon
            .as_deref()
            .map(run_opts::clean_icon)
            .transpose()
            .map_err(|m| invalid(&m))?;
        let flag = |b: bool| if b { "1" } else { "0" }.to_string();
        let mut writes: Vec<(&str, String)> = Vec::new();
        // 지움 = 빈 값(읽을 때 없음으로 본다).
        for (key, v) in [
            (run_opts::ME_ALIAS_KEY, me_alias),
            (run_opts::ME_ICON_KEY, me_icon),
        ] {
            if let Some(v) = v {
                writes.push((key, v.unwrap_or_default()));
            }
        }
        if let Some(w) = patch.ui_side_width {
            writes.push((run_opts::SIDE_WIDTH_KEY, w.to_string()));
        }
        for (key, v) in [
            (run_opts::MODEL_KEY, &patch.claude_model),
            (run_opts::EFFORT_KEY, &patch.claude_effort),
            (run_opts::THEME_KEY, &patch.ui_theme),
        ] {
            if let Some(v) = v {
                writes.push((key, v.clone()));
            }
        }
        for (key, v) in [
            ("notify_pair", patch.notify_pair),
            ("notify_request", patch.notify_request),
            ("notify_reply", patch.notify_reply),
            ("notify_preview", patch.notify_preview),
            ("notify_paused", patch.notify_paused),
            (settings::FOCUS_KEY, patch.focus),
        ] {
            if let Some(v) = v {
                writes.push((key, flag(v)));
            }
        }
        if let Some(m) = &patch.notify_relay {
            writes.push(("notify_relay", m.clone()));
        }
        if let Some(c) = patch.relay_cap_default {
            writes.push((RELAY_CAP_KEY, c.to_string()));
        }
        if let Some(on) = patch.notify_inbound {
            writes.push((NOTIFY_KEY, flag(on)));
        }
        // 전체 켜짐까지 한 트랜잭션 · 한 상태 잠금 — 부분 저장도, 그 사이에 끼는 이벤트도 없다.
        let mut st = self.status();
        let prefs = {
            let db = self.db.lock().expect("설정 DB 잠금이 오염됐다");
            let tx = db.unchecked_transaction().map_err(db_err)?;
            for (key, v) in &writes {
                settings::set(&tx, key, v).map_err(db_err)?;
            }
            let prefs = Prefs::load(&tx)?;
            tx.commit().map_err(db_err)?;
            prefs
        };
        st.prefs = prefs;
        if let Some(on) = patch.notify_inbound {
            st.enabled = on;
            if !on {
                st.pending = 0;
                (self.attention)(0);
            }
        }
        Ok(())
    }

    /// 실행 전역 기본값(model, effort) · 화면 테마.
    pub fn run_defaults(&self) -> io::Result<(String, String, String)> {
        let db = self.db.lock().expect("설정 DB 잠금이 오염됐다");
        let (model, effort) = run_opts::defaults(&db).map_err(db_err)?;
        Ok((model, effort, run_opts::theme(&db).map_err(db_err)?))
    }

    /// 화면 — 사이드바 폭 · '나' 타일(표시 이름, 아이콘).
    pub fn screen(&self) -> io::Result<(i64, Option<String>, Option<String>)> {
        let db = self.db.lock().expect("설정 DB 잠금이 오염됐다");
        let (alias, icon) = run_opts::me_tile(&db).map_err(db_err)?;
        Ok((run_opts::side_width(&db).map_err(db_err)?, alias, icon))
    }

    /// 자동 왕복 상한 기본값(설정).
    pub fn relay_cap_default(&self) -> io::Result<i64> {
        default_relay_cap(&self.db.lock().expect("설정 DB 잠금이 오염됐다")).map_err(db_err)
    }

    /// 막히지 않는다 — 켜져 있고 세부 설정이 허락하면 건수 · 트레이 표시를 올리고 구독자에게 뿌린다(구독자가 없으면 그냥
    /// 버려진다). 걸러진 이벤트는 건수에도 들지 않는다.
    pub fn on_event(&self, e: &PeerEvent) {
        let mut st = self.status();
        if !st.enabled || !st.prefs.allows(e) {
            return;
        }
        st.pending += 1;
        (self.attention)(st.pending);
        let _ = self.tx.send(notice_with(e, st.prefs.notify_preview));
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Notice> {
        self.tx.subscribe()
    }

    /// 사용자가 봤다 — 트레이 "새 요청" 을 지운다.
    pub fn clear(&self) {
        let mut st = self.status();
        st.pending = 0;
        (self.attention)(0);
    }

    /// 아직 보지 않은 알림 수.
    pub fn pending(&self) -> usize {
        self.status().pending
    }
}

pub fn notice_for(e: &PeerEvent) -> Notice {
    notice_with(e, true)
}

/// `preview` 가 거짓이면 원격 글 앞부분을 넣지 않는다(이름 · PC 만).
pub fn notice_with(e: &PeerEvent, preview_on: bool) -> Notice {
    match e {
        PeerEvent::PairRequest {
            name,
            host,
            fingerprint,
        } => Notice {
            kind: "pair_request",
            title: "페어링 요청".into(),
            body: format!("{name} · {host}\n지문 {fingerprint} — 웹UI 에서 확인 후 승인"),
        },
        PeerEvent::AutoPaired {
            name,
            host,
            fingerprint,
        } => Notice {
            kind: "auto_paired",
            title: "새 연결 등록".into(),
            body: format!(
                "{name} · {host}\n지문 {fingerprint} — 모르는 연결이면 연결 목록에서 삭제"
            ),
        },
        PeerEvent::Approved { name, host } => Notice {
            kind: "approved",
            title: "페어링 승인".into(),
            body: format!("{name} · {host} 이(가) 승인했다"),
        },
        PeerEvent::Message {
            name,
            host,
            agent,
            preview,
            kind,
        } => {
            let (code, title) = match kind {
                MessageKind::RelayStart => ("relay", format!("자동 왕복 시작 — {agent}")),
                MessageKind::RelayEnd => ("relay", format!("자동 왕복 끝 — {agent}")),
                MessageKind::RelayStep => ("relay", format!("자동 왕복 — {agent}")),
                MessageKind::Request | MessageKind::Reply => {
                    ("message", format!("메시지 — {agent}"))
                }
            };
            // 끝 알림의 본문은 이 PC 가 만든 멈춘 사유라 미리보기 설정과 무관하게 보인다.
            let body = if preview_on || *kind == MessageKind::RelayEnd {
                format!("{name} · {host}\n{preview}")
            } else {
                format!("{name} · {host}")
            };
            Notice {
                kind: code,
                title,
                body,
            }
        }
    }
}

/// `PUT /api/settings` 본문 — 준 항목만 바꾼다.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SettingsPatch {
    pub notify_inbound: Option<bool>,
    pub notify_pair: Option<bool>,
    pub notify_request: Option<bool>,
    pub notify_reply: Option<bool>,
    pub notify_relay: Option<String>,
    pub notify_preview: Option<bool>,
    /// 알림 일시 중지(이 PC 에만).
    pub notify_paused: Option<bool>,
    /// 집중(이 PC 에만) — 상대의 새 실행 요청 거부 + 알림 멈춤.
    pub focus: Option<bool>,
    pub relay_cap_default: Option<i64>,
    /// 실행 전역 기본값(상대별 덮어쓰기가 없을 때).
    pub claude_model: Option<String>,
    pub claude_effort: Option<String>,
    /// 화면 테마 — `light` · `dark`.
    pub ui_theme: Option<String>,
    /// 사이드바(대화 목록) 폭(px, [`run_opts::SIDE_WIDTHS`]).
    pub ui_side_width: Option<i64>,
    /// '나' 타일 — 표시 이름 · 아이콘(PNG data URL). 빈 문자열 = 지움.
    pub me_alias: Option<String>,
    pub me_icon: Option<String>,
}

/// `GET/PUT /api/settings` · `GET /api/events`(SSE) — `server::router_with` 의 guard 뒤에 붙는다.
pub fn routes(n: Arc<Notifier>) -> Router {
    Router::new()
        .route("/api/settings", get(get_settings).put(put_settings))
        .route("/api/events", get(events))
        .route("/api/seen", post(seen))
        .with_state(n)
}

fn settings_json(n: &Notifier) -> io::Result<serde_json::Value> {
    let mut v = serde_json::to_value(n.prefs()).unwrap_or_else(|_| json!({}));
    v["notify_inbound"] = json!(n.enabled());
    v["relay_cap_default"] = json!(n.relay_cap_default()?);
    let (model, effort, theme) = n.run_defaults()?;
    v["claude_model"] = json!(model);
    v["claude_effort"] = json!(effort);
    v["ui_theme"] = json!(theme);
    let (side, alias, icon) = n.screen()?;
    v["ui_side_width"] = json!(side);
    v["me_alias"] = json!(alias);
    v["me_icon"] = json!(icon);
    Ok(v)
}

async fn get_settings(State(n): State<Arc<Notifier>>) -> Response {
    settings_reply(settings_json(&n))
}

async fn put_settings(State(n): State<Arc<Notifier>>, Json(s): Json<SettingsPatch>) -> Response {
    settings_reply(n.update(&s).and_then(|()| settings_json(&n)))
}

/// 설정 응답 — 틀린 값은 400, DB 오류는 500(기본값을 성공처럼 내지 않는다).
fn settings_reply(v: io::Result<serde_json::Value>) -> Response {
    match v {
        Ok(v) => Json(v).into_response(),
        Err(e) => {
            let code = if e.kind() == io::ErrorKind::InvalidInput {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            (code, Json(json!({ "error": e.to_string() }))).into_response()
        }
    }
}

/// "봤다" — 웹UI 가 스트림에 붙은 뒤(onopen) 보낸다. 상태를 바꾸므로 POST(guard 가 Origin 을 요구)로만.
async fn seen(State(n): State<Arc<Notifier>>) -> StatusCode {
    n.clear();
    StatusCode::NO_CONTENT
}

/// 이후 알림을 흘린다 — GET 이라 상태를 바꾸지 않는다. 뒤처지면 놓친 것을 건너뛴다.
async fn events(
    State(n): State<Arc<Notifier>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = n.subscribe();
    let s = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(notice) => {
                    let ev = Event::default()
                        .json_data(&notice)
                        .unwrap_or_else(|_| Event::default().comment("직렬화 실패"));
                    return Some((Ok(ev), rx));
                }
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(s).keep_alive(KeepAlive::default())
}
