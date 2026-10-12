use an_app::notify::{Notifier, SettingsPatch};
use an_net::node::PeerEvent;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::broadcast::error::TryRecvError;

/// 임시 DB · 트레이 표시 대역(받은 건수를 모은다).
fn notifier(tag: &str) -> (Arc<Notifier>, Arc<Mutex<Vec<usize>>>) {
    let path = std::env::temp_dir().join(format!("an-app-notify-{tag}-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let marks = Arc::new(Mutex::new(Vec::new()));
    let sink = marks.clone();
    let n = Notifier::new(an_core::db::open(&path).unwrap(), move |count| {
        sink.lock().unwrap().push(count)
    })
    .unwrap();
    (n, marks)
}

fn request() -> PeerEvent {
    PeerEvent::PairRequest {
        name: "이도윤".into(),
        host: "DYOON-PC".into(),
        fingerprint: "7F3A 19C2 0000 0000 0000 0000 0000 0000".into(),
    }
}

#[test]
fn disabled_sends_nothing() {
    let (n, marks) = notifier("off");
    assert!(!n.enabled(), "기본 OFF");
    let mut rx = n.subscribe();
    n.on_event(&request());
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    assert!(marks.lock().unwrap().is_empty(), "트레이도 그대로");
}

#[test]
fn enabled_sends_notice_and_marks_tray() {
    let (n, marks) = notifier("on");
    n.set_enabled(true).unwrap();
    let mut rx = n.subscribe();
    n.on_event(&request());
    n.on_event(&PeerEvent::Approved {
        name: "박서연".into(),
        host: "SYPARK-WIN".into(),
    });
    let first = rx.try_recv().unwrap();
    assert_eq!(first.kind, "pair_request");
    assert_eq!(first.title, "페어링 요청");
    assert!(first.body.contains("이도윤 · DYOON-PC"), "{}", first.body);
    assert!(first.body.contains("7F3A 19C2"), "지문을 보여 준다");
    let second = rx.try_recv().unwrap();
    assert_eq!(
        (second.kind, second.title.as_str()),
        ("approved", "페어링 승인")
    );
    assert!(second.body.contains("박서연 · SYPARK-WIN"));
    assert_eq!(*marks.lock().unwrap(), vec![1, 2], "트레이 새 요청 건수");
}

#[test]
fn clear_and_off_reset_tray() {
    let (n, marks) = notifier("clear");
    n.set_enabled(true).unwrap();
    n.on_event(&request());
    n.clear();
    n.on_event(&request());
    n.set_enabled(false).unwrap();
    n.on_event(&request());
    assert_eq!(*marks.lock().unwrap(), vec![1, 0, 1, 0]);
}

#[test]
fn slow_subscriber_does_not_block() {
    let (n, marks) = notifier("slow");
    n.set_enabled(true).unwrap();
    let mut rx = n.subscribe(); // 읽지 않는 구독자
    let started = Instant::now();
    for _ in 0..10_000 {
        n.on_event(&request());
    }
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "보내는 쪽이 막혔다"
    );
    assert_eq!(marks.lock().unwrap().last(), Some(&10_000));
    // 뒤처진 구독자는 놓친 것을 알고 최근 것부터 이어 읽는다.
    assert!(matches!(rx.try_recv(), Err(TryRecvError::Lagged(_))));
    assert!(rx.try_recv().is_ok());
}

#[test]
fn concurrent_events_and_clear_keep_tray_in_sync() {
    // 트레이 갱신(1)이 느린 사이에 clear 가 끼어드는 순서를 강제한다 — 건수 변경과 표시가 한 순서로 묶여 있으면
    // clear 는 갱신이 끝날 때까지 기다리고, 마지막 표시는 실제 건수(0)와 같다.
    let path = std::env::temp_dir().join(format!("an-app-notify-race-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let marks = Arc::new(Mutex::new(Vec::new()));
    let (entered_tx, entered_rx) = std::sync::mpsc::channel::<()>();
    let entered_tx = Mutex::new(entered_tx);
    let sink = marks.clone();
    let n = Notifier::new(an_core::db::open(&path).unwrap(), move |c| {
        if c == 1 {
            let _ = entered_tx.lock().unwrap().send(());
            std::thread::sleep(Duration::from_millis(200));
        }
        sink.lock().unwrap().push(c);
    })
    .unwrap();
    n.set_enabled(true).unwrap();
    let worker = {
        let n = n.clone();
        std::thread::spawn(move || n.on_event(&request()))
    };
    entered_rx.recv().unwrap();
    n.clear();
    worker.join().unwrap();
    assert_eq!(n.pending(), 0);
    assert_eq!(
        *marks.lock().unwrap(),
        vec![1, 0],
        "표시 순서가 실제 상태와 어긋났다"
    );
}

#[test]
fn turning_off_during_events_leaves_nothing_pending() {
    // 알림이 쏟아지는 중에 끈다 — 끈 뒤에는 건수 0 · 마지막 표시 0 이어야 한다(켜짐 확인과 건수 변경 사이 틈이 없어야).
    for round in 0..30 {
        let (n, marks) = notifier(&format!("offrace{round}"));
        n.set_enabled(true).unwrap();
        let go = Arc::new(std::sync::Barrier::new(9));
        let workers: Vec<_> = (0..8)
            .map(|_| {
                let (n, go) = (n.clone(), go.clone());
                std::thread::spawn(move || {
                    go.wait();
                    for _ in 0..3000 {
                        n.on_event(&request());
                    }
                })
            })
            .collect();
        go.wait();
        std::thread::sleep(Duration::from_millis(1));
        n.set_enabled(false).unwrap();
        for w in workers {
            w.join().unwrap();
        }
        assert_eq!(n.pending(), 0, "round {round}: 끈 뒤 건수가 남았다");
        assert_eq!(
            marks.lock().unwrap().last(),
            Some(&0),
            "round {round}: 끈 뒤 표시가 남았다"
        );
    }
}

fn message(kind: an_net::node::MessageKind) -> PeerEvent {
    PeerEvent::Message {
        name: "이도윤".into(),
        host: "DYOON-PC".into(),
        agent: "review".into(),
        preview: "비밀 본문".into(),
        kind,
    }
}

#[test]
fn kinds_filtered() {
    use an_app::notify::SettingsPatch;
    use an_net::node::MessageKind::*;
    let (n, marks) = notifier("kinds");
    n.set_enabled(true).unwrap();
    let mut rx = n.subscribe();
    n.update(&SettingsPatch {
        notify_pair: Some(false),
        notify_reply: Some(false),
        ..Default::default()
    })
    .unwrap();
    n.on_event(&request());
    n.on_event(&message(Reply));
    n.on_event(&message(Request));
    // 꺼진 종류는 알림도 건수도 없다.
    assert_eq!(rx.try_recv().unwrap().kind, "message");
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    assert_eq!(*marks.lock().unwrap(), vec![1]);
}

#[test]
fn relay_modes() {
    use an_app::notify::SettingsPatch;
    use an_net::node::MessageKind::*;
    let (n, _) = notifier("relay");
    n.set_enabled(true).unwrap();
    let mut rx = n.subscribe();
    let titles = |rx: &mut tokio::sync::broadcast::Receiver<an_app::notify::Notice>| {
        std::iter::from_fn(|| rx.try_recv().ok())
            .map(|x| x.title)
            .collect::<Vec<_>>()
    };
    let run = |n: &Notifier| {
        for k in [RelayStart, RelayStep, RelayStep, RelayEnd] {
            n.on_event(&message(k));
        }
    };
    // 기본 = 시작 · 끝만.
    assert_eq!(n.prefs().notify_relay, "start_end");
    run(&n);
    assert_eq!(
        titles(&mut rx),
        ["자동 왕복 시작 — review", "자동 왕복 끝 — review"]
    );
    n.update(&SettingsPatch {
        notify_relay: Some("every".into()),
        ..Default::default()
    })
    .unwrap();
    run(&n);
    assert_eq!(titles(&mut rx).len(), 4);
    n.update(&SettingsPatch {
        notify_relay: Some("off".into()),
        ..Default::default()
    })
    .unwrap();
    run(&n);
    assert!(titles(&mut rx).is_empty());
    // 틀린 모드는 거부 · 그대로.
    assert!(
        n.update(&SettingsPatch {
            notify_relay: Some("sometimes".into()),
            ..Default::default()
        })
        .is_err()
    );
    assert_eq!(n.prefs().notify_relay, "off");
}

#[test]
fn preview_off() {
    use an_app::notify::SettingsPatch;
    use an_net::node::MessageKind::*;
    let (n, _) = notifier("preview");
    n.set_enabled(true).unwrap();
    let mut rx = n.subscribe();
    n.on_event(&message(Request));
    assert!(rx.try_recv().unwrap().body.contains("비밀 본문"));
    n.update(&SettingsPatch {
        notify_preview: Some(false),
        ..Default::default()
    })
    .unwrap();
    n.on_event(&message(Request));
    let hidden = rx.try_recv().unwrap();
    assert_eq!(hidden.body, "이도윤 · DYOON-PC");
    // 끝 알림의 사유는 이 PC 문구라 보인다.
    n.on_event(&message(RelayEnd));
    assert!(rx.try_recv().unwrap().body.contains("비밀 본문"));
}

fn pause(n: &Notifier, on: bool) {
    n.update(&SettingsPatch {
        notify_paused: Some(on),
        ..Default::default()
    })
    .unwrap();
}

/// 알림 일시 중지 — 켜져 있어도 알림 · 트레이 표시가 나가지 않는다(전체 켜짐은 그대로).
#[test]
fn paused_drops_notices_and_tray() {
    let (n, marks) = notifier("paused");
    n.set_enabled(true).unwrap();
    pause(&n, true);
    assert!(n.prefs().notify_paused);
    let mut rx = n.subscribe();
    n.on_event(&request());
    n.on_event(&message(an_net::node::MessageKind::Request));
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    assert!(marks.lock().unwrap().is_empty(), "트레이도 그대로");
    assert_eq!(n.pending(), 0);
    assert!(n.enabled(), "전체 켜짐은 따로");
}

/// 일시 중지를 풀면 다시 알린다 — 멈춘 동안의 이벤트는 쌓이지 않는다.
#[test]
fn resume_after_pause_notifies_again() {
    let (n, marks) = notifier("resume");
    n.set_enabled(true).unwrap();
    pause(&n, true);
    n.on_event(&request());
    pause(&n, false);
    let mut rx = n.subscribe();
    n.on_event(&request());
    assert_eq!(rx.try_recv().unwrap().kind, "pair_request");
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    assert_eq!(*marks.lock().unwrap(), vec![1]);
}

/// 집중 — 요청 거부와 함께 알림도 멈춘다(알림 일시 중지 설정 값은 그대로). 집중을 풀면 그 설정대로 다시 알린다.
#[test]
fn focus_pauses_notices() {
    let (n, marks) = notifier("focus");
    n.set_enabled(true).unwrap();
    let focus = |on: bool| {
        n.update(&SettingsPatch {
            focus: Some(on),
            ..Default::default()
        })
        .unwrap()
    };
    focus(true);
    assert!(n.prefs().focus && !n.prefs().notify_paused);
    let mut rx = n.subscribe();
    n.on_event(&request());
    n.on_event(&message(an_net::node::MessageKind::Reply));
    assert_eq!(rx.try_recv().unwrap_err(), TryRecvError::Empty);
    assert!(marks.lock().unwrap().is_empty(), "트레이도 그대로");
    focus(false);
    n.on_event(&request());
    assert_eq!(rx.try_recv().unwrap().kind, "pair_request");
    assert_eq!(*marks.lock().unwrap(), vec![1]);
}
