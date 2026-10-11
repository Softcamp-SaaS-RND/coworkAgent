# plan — 02.inbound-toast

> 근거: [intent](intent.md)(승인 2026-10-11, 자동 진행 위임 + 사용자 확인 답). **구현자는 이 문서만 보고 일한다.**
> 프로젝트 결정 번호(D·S)는 [masterPlan](../../masterPlan.md).

## 목표

인바운드 이벤트(페어링 요청 · 상대 승인)를 OS 알림(Windows 토스트)으로 알리는 옵션 — 기본 OFF, OS 알림 정책 준수(D27).

### 성공 기준(intent 사본)

- A1: 처음 보는 PC 의 페어링 요청이 받아들여지거나 아는 PC 가 다시 추가하면 "페어링 요청" 이벤트가, 상대 승인이 반영되면 "승인"
  이벤트가 한 번 난다. 거절된 요청(closed · busy · stale · unknown)은 이벤트가 없다.
- A2: 알림은 기본 OFF 이고, 웹UI 에서 켜고 끈 값이 재시작 뒤에도 유지된다. 꺼져 있으면 아무것도 표시하지 않는다.
- A3: 토스트 내용의 원격 문자열(이름 · 호스트)은 XML 이스케이프된다 — 원격 입력이 토스트 구조를 바꾸지 못한다.
- A4: 켜져 있으면 Windows 토스트가 Windows 알림 시스템으로 표시되고, 클릭하면 웹UI 피어 화면이 열린다. 집중 지원 · 방해 금지 중에는
  OS 정책대로 억제된다(수동 — D28).
- A5: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 · 의존 경계 행이 통과한다.

### intent 결정 · 제외(사본)

- N1 WinRT `ToastNotificationManager`(`windows` 0.62.2) + AUMID `agentNetwork` 를 `HKCU\Software\Classes\AppUserModelId\agentNetwork`
  에 등록 · N2 클릭 = 프로토콜 활성화(진입 URL `#peers`) · N3 `Node` 이벤트 콜백 → app `Notifier` · N4 `settings` 표(스키마 3)
  `notify_inbound` 기본 `0` · N5 등록은 켤 때만, 표시 실패는 로그만.
- 제외: 원격 메시지 알림(03) · 잠금/절전 감지 · 알림 버튼 · 소리 · 수동 확인(D28).

## 설계

- `an_net::node::PeerEvent::{PairRequest{name, host, fingerprint}, Approved{name, host}}`. `Node::on_event(f)`(한 번 설정,
  `OnceLock`). `answer` 는 `(Response, Option<PeerEvent>)` 를 돌려주고 `handle` 이 DB 잠금을 놓은 뒤 콜백을 부른다.
  - hello 수락(새 기록 · 다시 추가) → `PairRequest`. approve 가 같은 세대에 반영 → `Approved`. 그 밖은 없음.
- `an_core::settings` — `get(conn, key) -> Option<String>` · `set(conn, key, value)`. 스키마 3: `settings(key TEXT PRIMARY KEY,
  value TEXT NOT NULL)`.
- `an_app::notify`:
  - `Toast { title, body, launch }` · `toast_xml(&Toast) -> String`(모든 문자열 XML 이스케이프 — `&` `<` `>` `"` `'`).
  - `Notifier::new(conn, launch_url, show)` — `show: Box<dyn Fn(&Toast) + Send + Sync>`(운영 = OS 표시, 테스트 = 대역).
    `enabled()` · `set_enabled(bool)`(DB 저장 + 켤 때 `register()` 호출) · `on_event(&PeerEvent)`(꺼져 있으면 무시).
  - OS 표시 `show_os(&Toast)`: Windows = AUMID 등록(windows-sys 레지스트리) + WinRT 토스트, macOS = `osascript` `display notification`
    (AppleScript 문자열 이스케이프), 그 밖은 로그. 실패는 `log::warn!`.
- API: `GET /api/settings` → `{notify_inbound}` · `PUT /api/settings {notify_inbound: bool}`. 웹UI 내 정보 아래 체크박스.
- 시작: `Notifier` 를 만들고 `node.on_event(move |e| notifier.on_event(&e))`.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | Node 이벤트 · settings 스키마 3 · Notifier · 토스트 XML | 테스트(events · notifier · settings_api) | E1~E6 |

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — 인바운드 이벤트 2건(accepted_hello_emits_pair_request · applied_approve_emits_approved_only)
  CHECK: cargo test -p an-net --test events
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A1·A2·A3 — Notifier 5건(disabled_shows_nothing · enabled_shows_event · toast_xml_escapes_remote_text · blocked_os_does_not_block_events · turning_off_discards_queued_toasts)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A2 — 설정 API 2건(notify_default_off · notify_toggle_persists)
  CHECK: cargo test -p agentnetwork-app --test settings_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A5 — 의존 경계 3 · 3 · 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: pending
- [ ] E7: A4 — Windows 실동작(수동): 알림 켬 → 다른 PC 의 초대로 페어링 요청 → 토스트 표시(출처 agentNetwork) · 클릭 시 피어 화면 · 집중 지원 켬 상태에서 배너 억제(알림 센터에는 남음)
  EVIDENCE: pending

## 종결 시 할 일

- `PRODUCT.md` 의존(`windows` 0.62.2 직접) · 도메인 규칙(알림) · 매뉴얼 `an_app` · `an_net` · `an_core` 갱신.
- masterPlan `#02` 한 줄. 적대적 평가(BASE = plan 커밋, 2 라운드) — 범위에 01 개정 3(초대 · 지문 고정)을 포함.

## 개정 기록

2. 2026-10-11 · 설계 "Notifier" · 평가 항목 E4(4 → 5건) — 표시 스레드가 켜짐을 다시 보고 끈 뒤에는 대기 중이던 토스트를 버린다
   (처리 중 1건만 예외). 대기열 포화는 이벤트 쪽에서 버린 수만 세고 로그는 표시 스레드가 모아 쓴다. 웹UI 토글은 요청 중 비활성,
   저장 실패 시 서버 값으로 되돌린다. 사유: 적대적 평가 라운드 2. 승인: 자동 진행 위임.
1. 2026-10-11 · 설계 "Notifier" · "이벤트" · 평가 항목 E4(3 → 4건) — OS 표시는 전용 스레드 + 유한 대기열(`QUEUE` 16, 가득 차면
   버림)로 넘겨 이벤트 콜백이 막히지 않는다. macOS `osascript` 는 5초 뒤 kill. 같은 세대 approve 재통지는 `ok` 이고 이벤트는
   false → true 전이 때 한 번(`set_their_ok` 에 `their_ok = 0` 조건). 사유: 적대적 평가 라운드 1. 승인: 자동 진행 위임.
