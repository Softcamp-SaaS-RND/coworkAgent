//! 인바운드 알림 — 기본 OFF. 두 OS 같은 UI/UX(D30): 웹UI 탭이 열려 있으면 브라우저 알림(이벤트 스트림 →
//! `Notification`, OS 알림 정책은 브라우저 · OS 가 따른다), 탭과 무관하게 트레이 "새 요청" 표시.
use an_agent::chat::{MAX_RELAY_CAP, RELAY_CAP_KEY, default_relay_cap};
use an_core::db::{Connection, db_err};
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
    /// `request` · `pair_request` · `auto_paired` · `approved` · `message` · `relay`(`change` 는 알림이 아니라 보내지 않는다)
    pub kind: &'static str,
    pub title: String,
    pub body: String,
}

/// 웹UI 변경 신호 한 건(SSE `event: change`) — 알림과 별개, 알림 설정 · 집중과 무관하게 항상 간다.
/// `res` 가 비면 전체 다시 읽기(뒤처져 놓쳤을 때).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Change {
    pub res: Vec<Res>,
}

/// 바뀐 자원 — `conv`(이 PC 대화 id) · `peer`(연결).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Res {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

impl Change {
    pub fn full() -> Self {
        Change { res: Vec::new() }
    }
}

/// 이벤트 → 바뀐 자원.
pub fn change_of(e: &PeerEvent) -> Change {
    let r = match e {
        PeerEvent::Message { conv, .. } => Res {
            kind: "conv",
            id: Some(*conv),
        },
        PeerEvent::Changed { conv } => Res {
            kind: "conv",
            id: *conv,
        },
        PeerEvent::PairRequest { .. }
        | PeerEvent::Approved { .. }
        | PeerEvent::AutoPaired { .. } => Res {
            kind: "peer",
            id: None,
        },
    };
    Change { res: vec![r] }
}

type Attention = Box<dyn Fn(usize) + Send + Sync>;
/// 자동 시작 켜기(등록) · 끄기(지움) — 앱이 설치한다(디버그 빌드 · 시험은 없음).
type AutostartHook = Box<dyn Fn(bool) -> io::Result<()> + Send + Sync>;

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
    /// 받은 요청 알림 효과음(기본 켜짐) — 웹UI 가 낸다(집중 중에는 알림 자체가 없다).
    pub notify_sound: bool,
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
            notify_sound: flag("notify_sound")?,
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
            PeerEvent::Changed { .. } => false,
            PeerEvent::Message { kind, .. } => match kind {
                MessageKind::Request
                | MessageKind::Ask
                | MessageKind::AskState
                | MessageKind::Share
                | MessageKind::Clear => self.notify_request,
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
    /// 변경 신호 — 알림(`tx`)과 따로, 거르지 않는다.
    changes: broadcast::Sender<Change>,
    /// 트레이 표시 갱신(건수, 0 = 지움). 막히면 안 된다 — 운영은 이벤트 루프 프록시로 보낸다.
    attention: Attention,
    autostart: Mutex<Option<AutostartHook>>,
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
            changes: broadcast::channel(BACKLOG).0,
            attention: Box::new(attention),
            autostart: Mutex::new(None),
        }))
    }

    fn status(&self) -> std::sync::MutexGuard<'_, Status> {
        an_core::sync::lock(&self.status)
    }

    pub fn enabled(&self) -> bool {
        self.status().enabled
    }

    /// DB 에 저장한 뒤 반영한다. 끄면 트레이 표시도 지운다 — 상태 잠금 안에서라 그 사이 이벤트가 끼어들지 못한다.
    pub fn set_enabled(&self, on: bool) -> io::Result<()> {
        let mut st = self.status();
        {
            let db = an_core::sync::lock(&self.db);
            settings::set(&db, NOTIFY_KEY, if on { "1" } else { "0" }).map_err(db_err)?;
        }
        st.enabled = on;
        if !on {
            st.pending = 0;
            (self.attention)(0);
        }
        Ok(())
    }

    /// `PUT /api/settings {autostart}` 가 저장 전에 부른다 — 실패하면 저장하지 않는다.
    pub fn set_autostart_hook(
        &self,
        hook: impl Fn(bool) -> io::Result<()> + Send + Sync + 'static,
    ) {
        *an_core::sync::lock(&self.autostart) = Some(Box::new(hook));
    }

    /// 로그인 때 자동 시작 설정.
    pub fn autostart(&self) -> io::Result<bool> {
        settings::autostart(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    pub fn auto_compact(&self) -> io::Result<bool> {
        settings::auto_compact(&an_core::sync::lock(&self.db)).map_err(db_err)
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
            return Err(invalid(&format!(
                "claude_model 은 {} 중 하나",
                run_opts::MODELS.join(" · ")
            )));
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
        if patch
            .ui_rail_order
            .as_deref()
            .is_some_and(|t| !run_opts::RAIL_ORDERS.contains(&t))
        {
            return Err(invalid("ui_rail_order 는 peers_first · groups_first"));
        }
        if patch
            .ui_tile_size
            .as_deref()
            .is_some_and(|t| !run_opts::TILE_SIZES.contains(&t))
        {
            return Err(invalid("ui_tile_size 는 normal · small"));
        }
        if patch
            .ui_date_format
            .as_deref()
            .is_some_and(|t| !run_opts::DATE_FORMATS.contains(&t))
        {
            return Err(invalid("ui_date_format 는 md · relative · ymd · time"));
        }
        if patch
            .ui_home_view
            .as_deref()
            .is_some_and(|t| !run_opts::HOME_VIEWS.contains(&t))
        {
            return Err(invalid("ui_home_view 는 preview · list"));
        }
        if patch
            .ui_home_zoom
            .is_some_and(|z| !run_opts::HOME_ZOOMS.contains(&z))
        {
            return Err(invalid("ui_home_zoom 는 100 · 125 · 150 · 175 · 200"));
        }
        if patch
            .ui_peer_weeks
            .is_some_and(|w| !(1..=run_opts::PEER_WEEKS_MAX).contains(&w))
        {
            return Err(invalid("ui_peer_weeks 는 1 ~ 12"));
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
        // 빈 값 = 흑연(기본)으로 되돌림.
        let me_color = patch
            .me_color
            .as_deref()
            .map(|c| {
                if c.is_empty() {
                    Ok(None)
                } else {
                    run_opts::clean_tile_color(c).map(Some)
                }
            })
            .transpose()
            .map_err(|m| invalid(&m))?;
        let folded = patch
            .ui_folded
            .as_deref()
            .map(run_opts::clean_folded)
            .transpose()
            .map_err(|m| invalid(&m))?;
        let flag = |b: bool| if b { "1" } else { "0" }.to_string();
        let mut writes: Vec<(&str, String)> = Vec::new();
        // 지움 = 빈 값(읽을 때 없음으로 본다).
        for (key, v) in [
            (run_opts::ME_ALIAS_KEY, me_alias),
            (run_opts::ME_ICON_KEY, me_icon),
            (run_opts::ME_COLOR_KEY, me_color),
        ] {
            if let Some(v) = v {
                writes.push((key, v.unwrap_or_default()));
            }
        }
        if let Some(w) = patch.ui_side_width {
            writes.push((run_opts::SIDE_WIDTH_KEY, w.to_string()));
        }
        if let Some(w) = patch.ui_peer_weeks {
            writes.push((run_opts::PEER_WEEKS_KEY, w.to_string()));
        }
        if let Some(z) = patch.ui_home_zoom {
            writes.push((run_opts::HOME_ZOOM_KEY, z.to_string()));
        }
        if let Some(f) = folded {
            writes.push((run_opts::FOLDED_KEY, f));
        }
        for (key, v) in [
            (run_opts::MODEL_KEY, &patch.claude_model),
            (run_opts::EFFORT_KEY, &patch.claude_effort),
            (run_opts::THEME_KEY, &patch.ui_theme),
            (run_opts::RAIL_ORDER_KEY, &patch.ui_rail_order),
            (run_opts::TILE_SIZE_KEY, &patch.ui_tile_size),
            (run_opts::DATE_FORMAT_KEY, &patch.ui_date_format),
            (run_opts::HOME_VIEW_KEY, &patch.ui_home_view),
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
            ("notify_sound", patch.notify_sound),
            ("notify_paused", patch.notify_paused),
            (settings::FOCUS_KEY, patch.focus),
            (settings::AUTO_COMPACT_KEY, patch.auto_compact),
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
        // 자동 시작 — 켜기는 OS 등록이 저장보다 먼저, 끄기는 저장 뒤에 OS 지우기. 어느 순간 중단돼도(강제 종료 · 전원) 남는 어긋남은 "OS 켜짐 ·
        // 저장 꺼짐" 뿐이다 — 다음 로그인에 실행되고 시작 맞추기가 저장 값으로 고친다. "저장 켜짐 · OS 꺼짐"(실행 기회 자체가 없음)은 생기지 않는다.
        // 훅 잠금을 끝까지 쥐어 동시 요청의 등록 순서 = 저장 순서. 반환된 실패는 되돌린다(되돌리기마저 실패하면 다음 시작이 맞춘다).
        let hook = an_core::sync::lock(&self.autostart);
        let auto = patch
            .autostart
            .and_then(|on| hook.as_ref().map(|h| (h, on)));
        let before = match auto {
            Some(_) => self.autostart()?,
            None => true,
        };
        if let Some((h, true)) = auto {
            h(true)?;
        }
        if let Some(on) = patch.autostart {
            writes.push((settings::AUTOSTART_KEY, flag(on)));
        }
        // 전체 켜짐까지 한 트랜잭션 · 한 상태 잠금 — 부분 저장도, 그 사이에 끼는 이벤트도 없다.
        let mut st = self.status();
        let saved = (|| {
            let db = an_core::sync::lock(&self.db);
            let tx = an_core::db::write_tx(&db).map_err(db_err)?;
            for (key, v) in &writes {
                settings::set(&tx, key, v).map_err(db_err)?;
            }
            let prefs = Prefs::load(&tx)?;
            tx.commit().map_err(db_err)?;
            Ok::<_, io::Error>(prefs)
        })();
        let prefs = match saved {
            Ok(prefs) => prefs,
            Err(e) => {
                if let Some((h, true)) = auto
                    && let Err(u) = h(before)
                {
                    log::warn!("자동 시작 되돌리기 실패(다음 시작 때 맞춘다): {u}");
                }
                return Err(e);
            }
        };
        st.prefs = prefs;
        if let Some((h, false)) = auto
            && let Err(e) = h(false)
        {
            // 지우지 못했다 — 저장을 되돌린다(이 사이 중단돼도 "OS 켜짐 · 저장 꺼짐" 이라 다음 시작이 다시 지운다).
            let db = an_core::sync::lock(&self.db);
            if let Err(u) = settings::set(&db, settings::AUTOSTART_KEY, flag(before).as_str()) {
                log::warn!("자동 시작 설정 되돌리기 실패(다음 시작 때 맞춘다): {u}");
            }
            return Err(e);
        }
        drop(hook);
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
        let db = an_core::sync::lock(&self.db);
        let (model, effort) = run_opts::defaults(&db).map_err(db_err)?;
        Ok((model, effort, run_opts::theme(&db).map_err(db_err)?))
    }

    /// 레일 타일 크기 — `normal` · `small`.
    pub fn tile_size(&self) -> io::Result<String> {
        run_opts::tile_size(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 날짜 표시방식 — `md` · `relative` · `ymd` · `time`.
    pub fn date_format(&self) -> io::Result<String> {
        run_opts::date_format(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 개요 보기 — `preview` · `list`.
    pub fn home_view(&self) -> io::Result<String> {
        run_opts::home_view(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 개요 미리보기 배율(%) — 100 · 125 · 150 · 175 · 200.
    pub fn home_zoom(&self) -> io::Result<i64> {
        run_opts::home_zoom(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 주고받은 메시지 보기 기간(주) — 1 ~ 12.
    pub fn peer_weeks(&self) -> io::Result<i64> {
        run_opts::peer_weeks(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 받은 요청 알림 본문 재료 — 최근 대기 요청(상대 표시 이름 · 본문) 최대 [`REQUEST_LINES`] 건과 대기 전체 수. 못 읽으면 `None`(보통 알림으로).
    fn waiting(&self) -> Option<(Vec<(String, String)>, i64)> {
        let db = an_core::sync::lock(&self.db);
        let (rows, total) = an_core::chat::waiting_requests(&db, REQUEST_LINES as i64)
            .map_err(|e| log::warn!("받은 요청 읽기 실패: {e}"))
            .ok()?;
        let named = rows
            .into_iter()
            .map(|(peer, text)| {
                let name = an_core::peers::by_id(&db, peer)
                    .ok()
                    .flatten()
                    .map(|p| p.alias.unwrap_or(p.name))
                    .unwrap_or_default();
                (name, text)
            })
            .collect();
        Some((named, total))
    }

    pub fn rail_order(&self) -> io::Result<String> {
        run_opts::rail_order(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 화면 — 사이드바 폭 · '나' 타일(표시 이름, 아이콘, 색).
    pub fn screen(&self) -> io::Result<(i64, run_opts::MeTile)> {
        let db = an_core::sync::lock(&self.db);
        let me = run_opts::me_tile(&db).map_err(db_err)?;
        Ok((run_opts::side_width(&db).map_err(db_err)?, me))
    }

    /// 접은 절 열쇠 목록.
    pub fn folded(&self) -> io::Result<Vec<String>> {
        run_opts::folded(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 자동 왕복 상한 기본값(설정).
    pub fn relay_cap_default(&self) -> io::Result<i64> {
        default_relay_cap(&an_core::sync::lock(&self.db)).map_err(db_err)
    }

    /// 막히지 않는다 — 켜져 있고 세부 설정이 허락하면 건수 · 트레이 표시를 올리고 구독자에게 뿌린다(구독자가 없으면 그냥
    /// 버려진다). 걸러진 이벤트는 건수에도 들지 않는다.
    pub fn on_event(&self, e: &PeerEvent) {
        // 변경 신호는 켜짐 · 세부 설정 앞 — 알림을 꺼도 화면은 맞춰진다.
        let _ = self.changes.send(change_of(e));
        if matches!(e, PeerEvent::Changed { .. }) {
            return;
        }
        let mut st = self.status();
        if !st.enabled || !st.prefs.allows(e) {
            return;
        }
        let preview_on = st.prefs.notify_preview;
        st.pending += 1;
        (self.attention)(st.pending);
        drop(st);
        // 받은 요청 — 대기 요청 묶음 한 장. 상태 잠금을 놓고 읽는다(DB 잠금과 겹치지 않게).
        let ask = matches!(e, PeerEvent::Message { kind: MessageKind::Ask, .. })
            .then(|| self.waiting())
            .flatten()
            .filter(|(_, total)| *total > 0);
        let notice = match ask {
            Some((rows, total)) => request_notice(&rows, total, preview_on),
            None => notice_with(e, preview_on),
        };
        let _ = self.tx.send(notice);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Notice> {
        self.tx.subscribe()
    }

    pub fn subscribe_changes(&self) -> broadcast::Receiver<Change> {
        self.changes.subscribe()
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

/// 받은 요청 알림 본문 줄 수.
pub const REQUEST_LINES: usize = 3;

/// 받은 요청 알림 — 종류 `request`, 제목 "받은 요청 {대기 수}", 본문 = 최근 대기 요청 줄(최대 [`REQUEST_LINES`] — 미리보기 켜짐
/// `{이름} · {첫 줄 앞부분}`, 꺼짐 `{이름}`) + 넘으면 "외 {n}건". 웹UI 는 같은 tag 로 앞 알림을 바꾼다.
pub fn request_notice(rows: &[(String, String)], total: i64, preview_on: bool) -> Notice {
    let mut lines: Vec<String> = rows
        .iter()
        .take(REQUEST_LINES)
        .map(|(name, text)| {
            let first: String = text.lines().next().unwrap_or("").chars().take(40).collect();
            if preview_on && !first.is_empty() {
                format!("{name} · {first}")
            } else {
                name.clone()
            }
        })
        .collect();
    let rest = total - lines.len() as i64;
    if rest > 0 {
        lines.push(format!("외 {rest}건"));
    }
    Notice {
        kind: "request",
        title: format!("받은 요청 {total}"),
        body: lines.join("\n"),
    }
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
                "{name} · {host}\n지문 {fingerprint} — 모르는 연결이면 그룹설정에서 삭제"
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
            ..
        } => {
            let (code, title) = match kind {
                MessageKind::RelayStart => ("relay", format!("자동 왕복 시작 — {agent}")),
                MessageKind::RelayEnd => ("relay", format!("자동 왕복 끝 — {agent}")),
                MessageKind::RelayStep => ("relay", format!("자동 왕복 — {agent}")),
                MessageKind::Request | MessageKind::Reply => {
                    ("message", format!("메시지 — {agent}"))
                }
                MessageKind::Ask => ("message", format!("요청 받음 — {agent}")),
                MessageKind::AskState => ("message", format!("요청 상태 — {agent}")),
                // 상대가 내 대화를 공유했다 — 본문 = "관람자 n"(이름 없음).
                MessageKind::Share => ("message", format!("관람자 — {agent}")),
                // 상대가 대화 기록을 모두 지웠다 — 본문 = 사유(미리보기와 무관).
                MessageKind::Clear => {
                    return Notice {
                        kind: "message",
                        title: format!("대화 기록 삭제 — {agent}"),
                        body: format!("{name} · {host}\n{name} 이(가) 대화 기록을 모두 지웠습니다"),
                    };
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
        // 알림이 아니다 — `on_event` 가 먼저 돌려보낸다.
        PeerEvent::Changed { .. } => Notice {
            kind: "change",
            title: String::new(),
            body: String::new(),
        },
    }
}

/// 다음 변경 신호 — 뒤처져 놓쳤으면 빈 `res`(전체 다시 읽기) 하나, 닫히면 `None`.
pub async fn next_change(rx: &mut broadcast::Receiver<Change>) -> Option<Change> {
    match rx.recv().await {
        Ok(c) => Some(c),
        Err(RecvError::Lagged(_)) => Some(Change::full()),
        Err(RecvError::Closed) => None,
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
    /// 받은 요청 알림 효과음.
    pub notify_sound: Option<bool>,
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
    /// 레일 타일 순서 — `peers_first` · `groups_first`.
    pub ui_rail_order: Option<String>,
    /// 레일 타일 크기 — `normal` · `small`.
    pub ui_tile_size: Option<String>,
    /// 날짜 표시방식 — `md` · `relative` · `ymd` · `time`.
    pub ui_date_format: Option<String>,
    /// 개요 보기 — `preview` · `list`.
    pub ui_home_view: Option<String>,
    /// 개요 미리보기 배율(%) — [`run_opts::HOME_ZOOMS`].
    pub ui_home_zoom: Option<i64>,
    /// 주고받은 메시지 보기 기간(주) — 1 ~ 12.
    pub ui_peer_weeks: Option<i64>,
    /// 사이드바(대화 목록) 폭(px, [`run_opts::SIDE_WIDTHS`]).
    pub ui_side_width: Option<i64>,
    /// 접은 절 열쇠 목록([`run_opts::clean_folded`]) — 통째로 바꾼다.
    pub ui_folded: Option<Vec<String>>,
    /// '나' 타일 — 표시 이름 · 아이콘(PNG data URL). 빈 문자열 = 지움.
    pub me_alias: Option<String>,
    pub me_icon: Option<String>,
    /// '나' 타일 색 — `an_core::agents::ICON_COLORS` 중 하나, 빈 문자열 = 흑연(기본).
    pub me_color: Option<String>,
    /// 로그인 때 자동 시작(이 PC).
    pub autostart: Option<bool>,
    /// 자동 압축(이 PC) — 이 PC 에서 돈 세션이 컨텍스트 80% 를 넘으면 요약 후 새 세션.
    pub auto_compact: Option<bool>,
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
    v["ui_rail_order"] = json!(n.rail_order()?);
    v["ui_tile_size"] = json!(n.tile_size()?);
    v["ui_date_format"] = json!(n.date_format()?);
    v["ui_home_view"] = json!(n.home_view()?);
    v["ui_home_zoom"] = json!(n.home_zoom()?);
    v["ui_peer_weeks"] = json!(n.peer_weeks()?);
    let (side, (alias, icon, color)) = n.screen()?;
    v["ui_side_width"] = json!(side);
    v["me_alias"] = json!(alias);
    v["me_icon"] = json!(icon);
    v["me_color"] = json!(color);
    v["ui_folded"] = json!(n.folded()?);
    v["autostart"] = json!(n.autostart()?);
    v["auto_compact"] = json!(n.auto_compact()?);
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

/// 이후 알림 · 변경 신호를 흘린다 — GET 이라 상태를 바꾸지 않는다. 알림은 뒤처지면 놓친 것을 건너뛰고, 변경 신호는
/// 전체 다시 읽기 하나로 바꾼다.
async fn events(
    State(n): State<Arc<Notifier>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let changes = stream::unfold(n.subscribe_changes(), |mut rx| async move {
        let c = next_change(&mut rx).await?;
        let ev = Event::default()
            .event("change")
            .json_data(&c)
            .unwrap_or_else(|_| Event::default().comment("직렬화 실패"));
        Some((Ok(ev), rx))
    });
    let rx = n.subscribe();
    let notices = stream::unfold(rx, |mut rx| async move {
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
    Sse::new(stream::select(notices, changes)).keep_alive(KeepAlive::default())
}
