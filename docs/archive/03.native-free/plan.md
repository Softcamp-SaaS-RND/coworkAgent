# plan — 03.native-free

> 근거: [intent](intent.md)(승인 2026-10-11, 사용자 지시 · 확인 답). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

두 OS 같은 UI/UX(D30) — 인바운드 알림 = 브라우저 알림 + 트레이 표시, 시작 실패 = 트레이 오류 모드. OS 전용 UI API 제거.

### 성공 기준(intent 사본)

- A1: 알림이 켜져 있으면 인바운드 이벤트가 웹UI 이벤트 스트림으로 전달되고 트레이 "새 요청" 건수가 오른다. 꺼져 있으면 둘 다 없다.
- A2: 웹UI 가 이벤트 스트림에 붙거나 트레이에서 웹UI 를 열면 "새 요청" 표시가 지워진다. 알림을 끄면 표시도 지워진다.
- A3: 이벤트를 읽지 않는(느린) 웹UI 가 있어도 피어 수신 처리가 막히지 않는다.
- A4: 시작이 실패하면 사유를 담은 오류 파일이 생기고 트레이 오류 모드로 뜬다(파일 내용은 자동, 트레이는 수동 — D28).
- A5: 앱 코드에 OS 전용 UI API(WinRT · 메시지 상자 · AppleScript · 레지스트리)가 없다 — 의존 목록과 코드 검색으로 단정한다.
- A6: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 · 의존 경계 행이 통과한다.

### intent 결정 · 제외(사본)

- X1 SSE `GET /api/events` · X2 설정을 켤 때 브라우저 알림 권한 요청 · X3 트레이 주황 점 + 툴팁 건수(프록시로 갱신) · X4 오류 파일 =
  임시 폴더 `agentNetwork-start-error.txt`, "오류 보기" 는 OS 기본 앱 · X5 `tokio::sync::broadcast`.
- 제외: 원격 메시지 알림(04) · 탭 없는 브라우저 알림 · 수동 확인(D28).

## 설계

- `an_app::notify`:
  - `Notice { kind, title, body }`(Serialize) · `notice_for(&PeerEvent)`(`pair_request` · `approved`).
  - `Notifier::new(conn, attention: impl Fn(usize) + Send + Sync)` — `enabled()` · `set_enabled(on)`(끄면 `clear()`) ·
    `on_event(&PeerEvent)`(켜져 있으면 건수 +1 → `attention(n)` · broadcast 전송, 막히지 않음) · `subscribe()` · `clear()`(건수 0 →
    `attention(0)`).
  - `routes(n)`: `GET/PUT /api/settings`(02 그대로) · `GET /api/events` — **상태를 바꾸지 않고** `data: {Notice JSON}` 를 흘린다
    (뒤처진 구독자는 놓친 것을 건너뛴다) · `POST /api/seen` — `clear()`(웹UI 가 스트림 `onopen` 뒤 보낸다). 쿠키 인증의 GET/HEAD 밖
    요청은 guard 가 Origin 을 요구한다(개정 1).
  - 켜짐 · 건수는 한 잠금 — `on_event` · `clear` · `set_enabled` 가 표시 갱신 · 전송까지 한 순서로(개정 2).
- `an_app::tray`:
  - `Tray::new() -> (Tray, Attention)` — 메인 스레드에서 이벤트 루프를 만든다. `Attention::set(n)` 은 `EventLoopProxy` 로 보낸다.
  - `Tray::run(url, on_seen, on_quit) -> !` — 메뉴 "열기" · "내 정보 보기" · "종료". 열기 계열은 브라우저 열기에 성공하면 `on_seen()`.
    트레이를 못 만들면 사유 파일을 쓰고 OS 기본 앱으로 연 뒤 정리하고 끝낸다(개정 2). 아이콘 · 툴팁 갱신 실패는 로그.
  - `Tray::run_error(message, file) -> !` — 붉은 아이콘 · 툴팁 "agentNetwork 시작 실패: …" · 메뉴 "오류 보기"(`browser::open(file)`) · "종료".
  - `write_start_error(message) -> io::Result<PathBuf>` — 임시 폴더 파일에 시각 · 사유 · 로그 경로 안내를 쓴다.
- `main`: `Tray::new()` → `start(attention)` → 성공이면 `tray.run(…)`, 핸드오프면 종료, 실패면 `write_start_error` → `tray.run_error`.
  `report_fatal`(메시지 상자) 삭제.
- 웹UI: `EventSource('/api/events')` → 피어 목록 갱신 · 상태 줄 · (설정 켬 ∧ 권한 허용) `new Notification(title, {body})`. 설정을 켤 때
  `Notification.requestPermission()`.
- 의존: `windows` 제거, `windows-sys` 는 `Win32_UI_Shell` · `Win32_UI_WindowsAndMessaging`(브라우저 · 파일 열기 `ShellExecuteW` ·
  `SW_SHOWNORMAL`)만. `futures-util`(이미 트리에 있음)을 SSE 스트림용으로 직접 의존.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | Notifier 재작성 · SSE · 트레이 표시 · 오류 모드 · 네이티브 제거 | 테스트(notifier · events_api · start_error) | E1~E7 |

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3 — Notifier 6건(disabled_sends_nothing · enabled_sends_notice_and_marks_tray · clear_and_off_reset_tray · slow_subscriber_does_not_block · concurrent_events_and_clear_keep_tray_in_sync · turning_off_during_events_leaves_nothing_pending)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A1·A2 — 이벤트 스트림 4건(events_stream_delivers_without_clearing · events_need_token · seen_post_clears · cookie_post_without_origin_403)
  CHECK: cargo test -p agentnetwork-app --test events_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A4 — 시작 실패 파일 1건(start_error_file_has_reason)
  CHECK: cargo test -p agentnetwork-app --test start_error
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A5 — OS 전용 UI API 없음(의존 · 코드 검색)
  CHECK: python -c "import pathlib,re;src=' '.join(p.read_text(encoding='utf-8') for p in pathlib.Path('crates').rglob('*.rs'))+pathlib.Path('crates/app/Cargo.toml').read_text(encoding='utf-8');bad=[w for w in ['ToastNotification','MessageBoxW','RegSetKeyValueW','osascript','windows = ','Win32_System_Registry'] if w in src];print('native=' + (','.join(bad) or 'none'))"
  EXPECT: native=none
  EVIDENCE: pending
- [ ] E7: A6 — 의존 경계 3 · 3 · 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: pending
- [ ] E8: A1·A2·A4 — 실동작(수동, Windows · macOS 같은 절차): 알림 켬 → 권한 허용 → 페어링 요청 시 브라우저 알림 · 트레이 주황 점 · 툴팁 건수 → 웹UI 열면 지워짐 · 데이터 폴더를 읽기 전용으로 만든 시작 실패 → 트레이 오류 모드 · "오류 보기"
  EVIDENCE: pending

## 종결 시 할 일

- `PRODUCT.md` 의존 · 도메인 규칙(알림 · 시작 실패) · 매뉴얼 `an_app` 갱신. masterPlan `#03`.

## 개정 기록

2. 2026-10-11 · 설계 "notify" · "tray" · 평가 항목 E3(5 → 6) — 켜짐 · 건수를 한 잠금으로(끄기와 이벤트 경쟁 제거), 정상 트레이 생성
   실패도 사유 파일을 열고 끝냄, 아이콘 · 툴팁 갱신 실패 로그, 설계 본문을 개정 1 계약(GET 무부작용 · `POST /api/seen`)으로 정정.
   사유: 적대적 평가 라운드 2. 승인: 자동 진행 위임.
1. 2026-10-11 · 설계 "notify" · "tray" · "웹UI" · guard · 평가 항목 E3(4 → 5) · E4(2 → 4, 이름 변경) — 건수 변경과 트레이 표시를 한
   잠금으로 · SSE GET 은 상태를 바꾸지 않고 `POST /api/seen` 이 지움(웹UI `onopen`) · guard: 쿠키 인증의 GET/HEAD 밖 요청은 Origin
   필수(Bearer 예외) · 트레이 "열기" 는 브라우저 열기 성공 때만 지움 · 오류 트레이를 못 띄우면 사유 파일을 바로 연다 · 알림 권한
   거부 · 미지원이면 "트레이 표시만" 안내. 사유: 적대적 평가 라운드 1. 승인: 자동 진행 위임.
