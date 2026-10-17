//! 트레이 상주(S2) — 메뉴 "열기" · "종료", "새 요청" 표시(S9), 시작 실패 시 오류 모드(D30).
//! 두 OS 공통 tray-icon 만 쓴다. macOS 는 Dock 아이콘 없이 메뉴 막대에만(T4).
use crate::browser;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// 이벤트 루프로 들어오는 것.
pub enum TrayEvent {
    Menu(MenuEvent),
    /// 아직 보지 않은 알림 수(0 = 지움).
    Attention(usize),
    /// 로컬 API 종료 요청(`POST /api/quit`, #1n) — 메뉴 "종료" 와 같은 길.
    Quit,
}

/// 트레이 만들기를 다시 시도할 간격 — `n` 번째 실패 뒤(0부터). 합 60초, 그 뒤는 `None`(#1h — 로그인 직후 알림 영역이 아직 준비되지 않은 경우).
pub fn retry_after(n: u32) -> Option<Duration> {
    [1, 2, 4, 8, 15, 30]
        .get(n as usize)
        .map(|s| Duration::from_secs(*s))
}

/// 메인 스레드에서 만든 이벤트 루프. `run` · `run_error` 중 하나로 돌린다.
pub struct Tray {
    event_loop: EventLoop<TrayEvent>,
}

/// 다른 스레드에서 트레이 "새 요청" 표시를 바꾼다 — 프록시로 보내기만 해서 막히지 않는다. 트레이 없는 모드는 아무것도 안 한다.
#[derive(Clone)]
pub struct Attention(Option<EventLoopProxy<TrayEvent>>);

impl Attention {
    /// 트레이 없는 모드(#1n) — 표시할 곳이 없다(브라우저 알림은 그대로).
    pub fn none() -> Self {
        Self(None)
    }

    pub fn set(&self, count: usize) {
        if let Some(proxy) = &self.0 {
            let _ = proxy.send_event(TrayEvent::Attention(count));
        }
    }
}

/// 시작 실패 사유 파일 — 사용자 임시 폴더(데이터 경로 자체가 원인일 수 있다). 디버그 빌드의 시험용 데이터 루트(`COWORKAGENT_DATA_DIR`)가 있으면
/// 그 안 — 시험이 실제 인스턴스의 사유 파일을 덮어쓰지 않게(#1n).
pub fn start_error_file() -> PathBuf {
    const NAME: &str = "coworkAgent-start-error.txt";
    if cfg!(debug_assertions)
        && let Some(dir) = std::env::var_os(an_core::paths::DATA_DIR_ENV).filter(|v| !v.is_empty())
    {
        return PathBuf::from(dir).join(NAME);
    }
    std::env::temp_dir().join(NAME)
}

/// 사유 파일을 프로세스 `pid` 가 썼나 — 감시가 자식의 코드 3 이 그 자식의 시작 실패인지(사유를 남김) 가린다.
pub fn start_error_by(pid: u32) -> bool {
    let line = format!("PID: {pid}");
    std::fs::read_to_string(start_error_file()).is_ok_and(|text| text.lines().any(|l| l == line))
}

/// 사유 파일의 "사유:" 줄 — `start` · 감시가 앱이 남긴 사유를 읽는다(#1n).
pub fn read_start_error() -> Option<String> {
    let text = std::fs::read_to_string(start_error_file()).ok()?;
    text.lines()
        .find_map(|l| l.strip_prefix("사유: "))
        .map(str::to_string)
}

/// 사유 파일을 쓴다. 매번 덮어쓴다.
pub fn write_start_error(message: &str) -> io::Result<PathBuf> {
    let path = start_error_file();
    let log_hint = an_core::paths::Paths::default_root()
        .map(|r| {
            an_core::paths::Paths::under(r)
                .log_file()
                .display()
                .to_string()
        })
        .unwrap_or_else(|| "?".into());
    let at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    std::fs::write(
        &path,
        format!(
            "coworkAgent 시작 실패\n\n사유: {message}\n\n시각(유닉스 초): {at}\n로그: {log_hint}\nPID: {}\n",
            std::process::id()
        ),
    )?;
    Ok(path)
}

impl Tray {
    pub fn new() -> (Self, Attention) {
        #[allow(unused_mut)]
        let mut event_loop = EventLoopBuilder::<TrayEvent>::with_user_event().build();
        #[cfg(target_os = "macos")]
        {
            use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
            event_loop.set_activation_policy(ActivationPolicy::Accessory);
            event_loop.set_dock_visibility(false);
        }
        let proxy = event_loop.create_proxy();
        let menu_proxy = proxy.clone();
        MenuEvent::set_event_handler(Some(move |event| {
            let _ = menu_proxy.send_event(TrayEvent::Menu(event));
        }));
        (Self { event_loop }, Attention(Some(proxy)))
    }

    /// 로컬 API 종료 요청을 이벤트 루프로 보내는 함수(#1n).
    pub fn quitter(&self) -> crate::server::Quit {
        let proxy = std::sync::Mutex::new(self.event_loop.create_proxy());
        Box::new(move || {
            if let Ok(p) = proxy.lock() {
                let _ = p.send_event(TrayEvent::Quit);
            }
        })
    }

    /// 정상 모드 — "열기" 는 웹UI 를 열고 `on_seen` 을 부른다(새 요청 표시 지움). "종료" 는 `on_quit` 후 끝. `note` 는 툴팁 둘째 줄(피어 포트 등).
    pub fn run(
        self,
        url: String,
        note: Option<String>,
        on_seen: impl Fn() + 'static,
        on_quit: impl FnOnce() + 'static,
    ) -> ! {
        let open = MenuItem::new("열기", true, None);
        let quit = MenuItem::new("종료", true, None);
        let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
        let mut menu = Some((open, quit));
        let mut tray: Option<TrayIcon> = None;
        let mut on_quit = Some(on_quit);
        // 트레이 만들기 실패 횟수 · 다음 시도 시각(#1h) — 기다리는 동안에도 다른 이벤트(새 요청 표시 등)는 돈다.
        let mut failures = 0;
        let mut retry_at: Option<Instant> = None;
        // 마지막 "새 요청" 수 — 트레이가 아직 없을 때(다시 시도 중) 온 값도 남겨 만들 때 그대로 그린다(라운드 1).
        let mut attention = 0;

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = retry_at.map_or(ControlFlow::Wait, ControlFlow::WaitUntil);
            match event {
                // macOS 는 이벤트 루프가 돈 뒤에 트레이를 만들어야 한다. 실패하면 간격을 늘려 다시(`retry_after`).
                Event::NewEvents(StartCause::Init | StartCause::ResumeTimeReached { .. }) => {
                    if let Some((open, quit)) = &menu {
                        retry_at = None;
                        match build(&[open, quit], &tip(attention, note.as_deref()), icon(mark_of(attention))) {
                            Ok(t) => {
                                tray = Some(t);
                                menu = None;
                                *control_flow = ControlFlow::Wait;
                            }
                            Err(e) => match retry_after(failures) {
                                Some(wait) => {
                                    failures += 1;
                                    log::warn!("트레이 아이콘을 만들 수 없다({failures}번째) — {}초 뒤 다시: {e}", wait.as_secs());
                                    let at = Instant::now() + wait;
                                    retry_at = Some(at);
                                    *control_flow = ControlFlow::WaitUntil(at);
                                }
                                None => {
                                    // 트레이 없이는 쓸 수 없다 — 사유를 보이고 정리 뒤 끝낸다(조용히 사라지지 않는다). 종료 코드 0 — 감시가 다시 띄우면
                                    // 같은 다시 시도를 되풀이하며 사유 파일을 매번 연다(1h plan 해석 7).
                                    show_failure(&format!("트레이 아이콘을 만들 수 없다: {e}"));
                                    if let Some(f) = on_quit.take() {
                                        f();
                                    }
                                    *control_flow = ControlFlow::Exit;
                                }
                            },
                        }
                    }
                }
                Event::UserEvent(TrayEvent::Attention(n)) => {
                    attention = n;
                    if let Some(t) = &tray {
                        let tip = tip(n, note.as_deref());
                        if let Err(e) = t.set_icon(Some(icon(mark_of(n)))) {
                            log::warn!("트레이 아이콘 갱신 실패(새 요청 {n}건): {e}");
                        }
                        if let Err(e) = t.set_tooltip(Some(tip)) {
                            log::warn!("트레이 툴팁 갱신 실패(새 요청 {n}건): {e}");
                        }
                    }
                }
                Event::UserEvent(TrayEvent::Menu(e)) if e.id == open_id => {
                    // 열린 경우에만 "봤다" — 실패하면 새 요청 표시를 남긴다.
                    match browser::open(&url) {
                        Ok(()) => on_seen(),
                        Err(e) => log::error!("브라우저 열기 실패: {e}"),
                    }
                }
                Event::UserEvent(TrayEvent::Quit) => {
                    tray.take();
                    if let Some(f) = on_quit.take() {
                        f();
                    }
                    *control_flow = ControlFlow::Exit;
                }
                Event::UserEvent(TrayEvent::Menu(e)) if e.id == quit_id => {
                    tray.take();
                    if let Some(f) = on_quit.take() {
                        f();
                    }
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        })
    }

    /// 오류 모드 — 붉은 아이콘 · 툴팁에 사유, 메뉴 "오류 보기"(사유 파일을 OS 기본 앱으로) · "종료". 감시 잠금 안에서만 띄운다(#1n — 하나).
    /// "오류 보기" 는 열기 전에 사유 파일을 다시 쓴다 — 다른 실행이 덮어쓴 사유를 보여 주지 않게.
    pub fn run_error(self, message: &str, file: &Path) -> ! {
        let show = MenuItem::new("오류 보기", true, None);
        let quit = MenuItem::new("종료", true, None);
        let (show_id, quit_id) = (show.id().clone(), quit.id().clone());
        let mut menu = Some((show, quit));
        let mut tray: Option<TrayIcon> = None;
        // 툴팁은 짧게(OS 마다 길이 제한이 있다).
        let tip: String = format!("coworkAgent 시작 실패: {message}")
            .chars()
            .take(120)
            .collect();
        let file = file.display().to_string();
        let message = message.to_string();

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::NewEvents(StartCause::Init) => {
                    if let Some((show, quit)) = menu.take() {
                        match build(&[&show, &quit], &tip, icon(Mark::Error)) {
                            Ok(t) => tray = Some(t),
                            Err(e) => {
                                // 트레이도 못 띄우면 사유 파일을 바로 연다 — 조용히 사라지지 않는다.
                                log::error!("오류 트레이 생성 실패: {e}");
                                if let Err(e) = browser::open(&file) {
                                    eprintln!("coworkAgent: 오류 파일 열기 실패: {e}");
                                }
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }
                }
                Event::UserEvent(TrayEvent::Menu(e)) if e.id == show_id => {
                    let file = write_start_error(&message)
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|_| file.clone());
                    if let Err(e) = browser::open(&file) {
                        log::error!("오류 파일 열기 실패: {e}");
                    }
                }
                Event::UserEvent(TrayEvent::Menu(e)) if e.id == quit_id => {
                    tray.take();
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        })
    }
}

/// 툴팁 — "coworkAgent"(새 요청이 있으면 " — 새 요청 n건") + 덧붙임 줄. Windows 툴팁은 UTF-16 128 칸(끝 NUL 포함)이라 넘으면 "…" 로 자른다.
pub fn tip(n: usize, note: Option<&str>) -> String {
    let head = if n > 0 {
        format!("coworkAgent — 새 요청 {n}건")
    } else {
        "coworkAgent".to_string()
    };
    let full = match note {
        Some(note) => format!("{head}\n{note}"),
        None => head,
    };
    if full.encode_utf16().count() <= 127 {
        return full;
    }
    let mut cut = String::new();
    for c in full.chars() {
        if cut.encode_utf16().count() + c.len_utf16() > 126 {
            break;
        }
        cut.push(c);
    }
    cut + "…"
}

fn build(items: &[&MenuItem], tooltip: &str, icon: Icon) -> Result<TrayIcon, String> {
    let menu = Menu::new();
    for item in items {
        menu.append(*item).map_err(|e| format!("메뉴 구성: {e}"))?;
    }
    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .map_err(|e| e.to_string())
}

/// 트레이를 못 띄운 실패 — 사유 파일을 쓰고 OS 기본 앱으로 연다(못 하면 stderr).
fn show_failure(message: &str) {
    log::error!("{message}");
    eprintln!("coworkAgent: {message}");
    match write_start_error(message) {
        Ok(file) => {
            if let Err(e) = browser::open(&file.display().to_string()) {
                eprintln!("coworkAgent: 오류 파일 열기 실패: {e}");
            }
        }
        Err(e) => eprintln!("coworkAgent: 오류 파일 쓰기 실패: {e}"),
    }
}

/// 새 요청 수의 표시 — 있으면 주황 점.
fn mark_of(n: usize) -> Mark {
    if n > 0 { Mark::Attention } else { Mark::None }
}

#[derive(Clone, Copy)]
enum Mark {
    None,
    /// 새 요청 — 오른쪽 위 주황 점.
    Attention,
    /// 시작 실패 — 붉은 바탕.
    Error,
}

/// 32×32 RGBA — 실행 파일 아이콘과 같은 그림. `tools/gen_icon.py` 가 만든다.
const TRAY: &[u8] = include_bytes!("../assets/tray.rgba");
const TRAY_ERROR: &[u8] = include_bytes!("../assets/tray-error.rgba");

fn icon(mark: Mark) -> Icon {
    const N: u32 = 32;
    let mut rgba = match mark {
        Mark::Error => TRAY_ERROR,
        _ => TRAY,
    }
    .to_vec();
    if matches!(mark, Mark::Attention) {
        for y in 0..N {
            for x in 0..N {
                let (ax, ay) = (x as f32 - 25.0, y as f32 - 6.0);
                if ax * ax + ay * ay <= 36.0 {
                    let i = ((y * N + x) * 4) as usize;
                    rgba[i..i + 4].copy_from_slice(&[0xf0, 0x8c, 0x00, 0xff]);
                }
            }
        }
    }
    Icon::from_rgba(rgba, N, N).expect("고정 크기 아이콘")
}

#[cfg(test)]
mod tests {
    #[test]
    fn tip_lines_and_windows_limit() {
        assert_eq!(super::tip(0, None), "coworkAgent");
        assert_eq!(
            super::tip(2, Some("둘째")),
            "coworkAgent — 새 요청 2건\n둘째"
        );
        let long = super::tip(1, Some(&"가".repeat(200)));
        assert_eq!(long.encode_utf16().count(), 127);
        assert!(long.ends_with('…'));
    }

    /// 다시 시도 간격(#1h) — 1 · 2 · 4 · 8 · 15 · 30초(합 60초), 그 뒤 없음.
    #[test]
    fn retry_after_schedule() {
        let secs: Vec<u64> = (0..10)
            .map_while(super::retry_after)
            .map(|d| d.as_secs())
            .collect();
        assert_eq!(secs, [1, 2, 4, 8, 15, 30]);
        assert_eq!(secs.iter().sum::<u64>(), 60);
        assert_eq!(super::retry_after(6), None);
        assert_eq!(super::retry_after(u32::MAX), None);
    }
}
