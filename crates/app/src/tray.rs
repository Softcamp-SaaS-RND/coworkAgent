//! 트레이 상주(S2) — 메뉴 "열기" · "종료", "새 요청" 표시(S9), 시작 실패 시 오류 모드(D30).
//! 두 OS 공통 tray-icon 만 쓴다. macOS 는 Dock 아이콘 없이 메뉴 막대에만(T4).
use crate::browser;
use std::io;
use std::path::{Path, PathBuf};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

/// 이벤트 루프로 들어오는 것.
pub enum TrayEvent {
    Menu(MenuEvent),
    /// 아직 보지 않은 알림 수(0 = 지움).
    Attention(usize),
}

/// 메인 스레드에서 만든 이벤트 루프. `run` · `run_error` 중 하나로 돌린다.
pub struct Tray {
    event_loop: EventLoop<TrayEvent>,
}

/// 다른 스레드에서 트레이 "새 요청" 표시를 바꾼다 — 프록시로 보내기만 해서 막히지 않는다.
#[derive(Clone)]
pub struct Attention(EventLoopProxy<TrayEvent>);

impl Attention {
    pub fn set(&self, count: usize) {
        let _ = self.0.send_event(TrayEvent::Attention(count));
    }
}

/// 시작 실패 사유 파일 — 사용자 임시 폴더(데이터 경로 자체가 원인일 수 있다). 매번 덮어쓴다.
pub fn write_start_error(message: &str) -> io::Result<PathBuf> {
    let path = std::env::temp_dir().join("coworkAgent-start-error.txt");
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
            "coworkAgent 시작 실패\n\n사유: {message}\n\n시각(유닉스 초): {at}\n로그: {log_hint}\n"
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
        (Self { event_loop }, Attention(proxy))
    }

    /// 정상 모드 — "열기" 는 웹UI 를 열고 `on_seen` 을 부른다(새 요청 표시 지움). "종료" 는 `on_quit` 후 끝.
    pub fn run(
        self,
        url: String,
        on_seen: impl Fn() + 'static,
        on_quit: impl FnOnce() + 'static,
    ) -> ! {
        let open = MenuItem::new("열기", true, None);
        let quit = MenuItem::new("종료", true, None);
        let (open_id, quit_id) = (open.id().clone(), quit.id().clone());
        let mut menu = Some((open, quit));
        let mut tray: Option<TrayIcon> = None;
        let mut on_quit = Some(on_quit);

        self.event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                // macOS 는 이벤트 루프가 돈 뒤에 트레이를 만들어야 한다.
                Event::NewEvents(StartCause::Init) => {
                    if let Some((open, quit)) = menu.take() {
                        match build(&[&open, &quit], "coworkAgent", icon(Mark::None)) {
                            Ok(t) => tray = Some(t),
                            Err(e) => {
                                // 트레이 없이는 쓸 수 없다 — 사유를 보이고 정리 뒤 끝낸다(조용히 사라지지 않는다).
                                show_failure(&format!("트레이 아이콘을 만들 수 없다: {e}"));
                                if let Some(f) = on_quit.take() {
                                    f();
                                }
                                *control_flow = ControlFlow::Exit;
                            }
                        }
                    }
                }
                Event::UserEvent(TrayEvent::Attention(n)) => {
                    if let Some(t) = &tray {
                        let (mark, tip) = if n > 0 {
                            (Mark::Attention, format!("coworkAgent — 새 요청 {n}건"))
                        } else {
                            (Mark::None, "coworkAgent".to_string())
                        };
                        if let Err(e) = t.set_icon(Some(icon(mark))) {
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

    /// 오류 모드 — 붉은 아이콘 · 툴팁에 사유, 메뉴 "오류 보기"(사유 파일을 OS 기본 앱으로) · "종료".
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
