# eval.md — 03.native-free 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 7dbe37b2e0dfa30eccd37906138fd3c3120147ea

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.22s

- [x] E3: A1·A2·A3 — Notifier 6건(disabled_sends_nothing · enabled_sends_notice_and_marks_tray · clear_and_off_reset_tray · slow_subscriber_does_not_block · concurrent_events_and_clear_keep_tray_in_sync · turning_off_during_events_leaves_nothing_pending)
  CHECK: cargo test -p agentnetwork-app --test notifier
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.56s

- [x] E4: A1·A2 — 이벤트 스트림 4건(events_stream_delivers_without_clearing · events_need_token · seen_post_clears · cookie_post_without_origin_403)
  CHECK: cargo test -p agentnetwork-app --test events_api
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s

- [x] E5: A4 — 시작 실패 파일 1건(start_error_file_has_reason)
  CHECK: cargo test -p agentnetwork-app --test start_error
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E6: A5 — OS 전용 UI API 없음(의존 · 코드 검색)
  CHECK: python -c "import pathlib,re;src=' '.join(p.read_text(encoding='utf-8') for p in pathlib.Path('crates').rglob('*.rs'))+pathlib.Path('crates/app/Cargo.toml').read_text(encoding='utf-8');bad=[w for w in ['ToastNotification','MessageBoxW','RegSetKeyValueW','osascript','windows = ','Win32_System_Registry'] if w in src];print('native=' + (','.join(bad) or 'none'))"
  EXPECT: native=none
  EVIDENCE: native=none

- [x] E7: A6 — 의존 경계 3 · 3 · 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: crates=3 edges=3 violations=0

- [ ] E8: A1·A2·A4 — 실동작(수동, Windows · macOS 같은 절차): 알림 켬 → 권한 허용 → 페어링 요청 시 브라우저 알림 · 트레이 주황 점 · 툴팁 건수 → 웹UI 열면 지워짐 · 데이터 폴더를 읽기 전용으로 만든 시작 실패 → 트레이 오류 모드 · "오류 보기"
  EVIDENCE: pending

ABANDON: E8 원격 세션이라 화면 확인 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 48ec624 --scope branch` · 대상 HEAD `0bcb535` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] clear와 on_event 경쟁으로 트레이 상태가 영구적으로 어긋난다 (crates/app/src/notify.rs:89-101)
  pending 원자값 변경과 Attention 전송이 하나의 순서로 보호되지 않는다. 예를 들어 on_event가 fetch_add(1) 후 멈추고 clear가 store(0)·Attention(0)을 보낸 뒤 on_event가 Attention(1)을 보내면 내부 pending은 0인데 트레이는 1건으로 남는다. 반대 순서에서는 실제 pending이 1인데 트레이가 지워진다. 피어 연결은 별도 Tokio task에서 처리되므로 SSE 연결·설정 OFF·트레이 열기와 실제로 병행될 수 있다. A1/A2의 건수와 소거 계약을 위반한다.
  Recommendation: pending 변경과 Attention 전송을 같은 Mutex 또는 단일 순서화 채널로 묶고, on_event 대 clear/OFF 및 복수 on_event 교차 실행을 barrier 기반 테스트로 고정한다.
- [high] 오류 트레이 생성 실패 시 시작 실패가 완전히 보이지 않는다 (crates/app/src/tray.rs:154-159)
  오류 모드 TrayIconBuilder 실패는 build가 None을 반환하고 event loop를 즉시 종료하는 것으로 끝난다. 데이터 경로나 FileLogger 설치 전 발생한 시작 실패라면 build 내부 log::error도 기록되지 않으며, Windows release는 콘솔도 없다. 결과적으로 A4가 요구한 트레이 오류 모드 없이 프로세스가 조용히 사라지고, 생성된 오류 파일 위치도 사용자에게 노출되지 않는다.
  Recommendation: 트레이 생성 실패를 Result로 main까지 전달하고, 이 경우 허용된 OS 기본 앱으로 오류 파일을 직접 여는 2차 fallback을 실행한다. TrayIconBuilder 실패를 주입하는 테스트도 추가한다.
- [medium] SSE GET이 인증 쿠키만으로 알림 상태를 지우는 부작용을 가진다 (crates/app/src/notify.rs:152-157)
  GET /api/events는 응답을 만들면서 즉시 clear한다. server guard는 Origin이 존재할 때만 검사하고 an_token은 127.0.0.1 전체에 적용되는 host cookie다. 따라서 다른 loopback 포트의 같은-site 페이지가 쿠키를 포함하되 Origin을 보내지 않는 no-CORS GET을 발생시키면, 응답을 읽지 못해도 handler 부작용으로 트레이 알림을 지울 수 있다는 추론이 성립한다. 이는 토큰 값을 몰라도 알림을 억제하는 CSRF 경로다.
  Recommendation: SSE GET에서는 상태를 변경하지 말고, EventSource onopen 뒤 별도 POST acknowledgement를 보내게 한다. 상태 변경 API는 정확한 Origin과 인증을 모두 필수로 검사한다.
- [medium] 브라우저 열기 실패에도 새 요청 표시를 지운다 (crates/app/src/tray.rs:120-123)
  browser::open이 실패해도 오류를 로그로만 남긴 뒤 on_seen을 무조건 호출한다. 기본 브라우저 연결 오류나 OS open 실패 시 웹UI는 보이지 않았는데 트레이 표시는 사라져 사용자가 요청을 놓친다. A2의 '트레이에서 웹UI를 열면' 조건을 클릭 시도로 축소한 구현이다.
  Recommendation: browser::open이 Ok인 경우에만 on_seen을 호출한다. Err이면 표시를 유지하고 재시도 가능한 오류 상태를 노출한다.
- [medium] Notification 권한 거부를 무시한 채 설정을 ON으로 확정한다 (crates/app/web/app.js:144-155)
  Notification.requestPermission 결과를 await하거나 검사하지 않는다. 사용자가 거부하거나 API가 지원되지 않아도 PUT은 실행되고 체크박스와 상태 문구는 '인바운드 알림 켬'으로 남는다. X2가 요구한 '거부되면 트레이 표시만 남는다고 안내'가 없어서 브라우저 알림이 조용히 비활성화된다.
  Recommendation: 사용자 동작 안에서 requestPermission을 호출해 결과를 처리하고, denied 또는 API 부재 시 설정 저장 후 '브라우저 알림 불가 — 트레이만 사용' 상태를 명시한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| clear ↔ on_event 경쟁 | 사실 | 건수(`Mutex<usize>`) 변경과 `attention` 호출을 같은 잠금 안에서. 회귀 `concurrent_events_and_clear_keep_tray_in_sync`(표시 중 clear 를 끼워 넣는 순서를 강제, E3) — "건수 · 표시 분리" mutation FAIL 확인(처음 쓴 무작위 스트레스 테스트는 이 mutation 을 못 잡아 결정적 순서 테스트로 바꿨다) |
| 오류 트레이 생성 실패 무표시 | 사실 | 오류 모드에서 트레이를 못 띄우면 사유 파일을 OS 기본 앱으로 바로 연다(+ stderr). 트레이 생성 실패 주입 테스트는 미추가(아래 미조치 후보 — 라운드 2 뒤 확정) |
| SSE GET 부작용(CSRF) | 사실(추론 성립 — 쿠키가 포트를 구분하지 않음, #00 개정 1) | GET `/api/events` 는 상태 변경 없음, `POST /api/seen` 이 지움(웹UI `onopen`). guard: 쿠키 인증의 GET/HEAD 밖 요청은 Origin 필수(Bearer 핸드오프 예외). 회귀 `seen_post_clears` · `cookie_post_without_origin_403` · `events_stream_delivers_without_clearing`(E4) — "Origin 필수 제거" · "GET 이 지움" mutation FAIL 확인. 기존 쿠키 상태 변경 테스트에 Origin 추가 |
| 브라우저 열기 실패 시 소거 | 사실 | `browser::open` 이 `Ok` 일 때만 `on_seen`. 자동 테스트 없음(트레이 이벤트 루프 — 수동 E8) |
| 알림 권한 거부 무안내 | 사실 | 클릭 안에서 `requestPermission()` 결과를 받아 granted 가 아니면 "브라우저 알림 불가(권한 거부 또는 미지원), 트레이 표시만" 안내. JS 테스트 없음 |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 · 같은 base `48ec624` · 대상 HEAD `17f6457` · 2026-10-11 (리뷰 쪽 테스트 재실행은 read-only 샌드박스로 못 함)
- Verdict: needs-attention

### 원문

```text
- [high] OFF 전환과 `on_event` 경쟁으로 알림이 다시 살아난다 (crates/app/src/notify.rs:73-94)
  `on_event`가 line 86에서 `enabled=true`를 읽고 멈춘 뒤 `set_enabled(false)`가 OFF 저장과 `clear()`를 완료할 수 있다. 이후 기존 `on_event`가 `pending` 잠금을 얻어 1로 올리고 트레이 표시와 SSE를 전송한다. 최종 상태가 `enabled=false`, `pending=1`이 되어 A1·A2를 위반한다. `concurrent_events_and_clear_keep_tray_in_sync`는 이벤트가 이미 잠금 안에 들어간 순서만 검사해 이 창을 잡지 못한다.
  Recommendation: `enabled`와 `pending`을 같은 상태 잠금으로 보호하고 OFF 전환·clear·건수 변경·attention 전송을 한 순서로 직렬화한다. `on_event`가 enabled 확인 후 잠금 전 정지하고 OFF가 완료되는 barrier 회귀 테스트를 추가한다.
- [high] 정상 트레이 생성 실패가 오류 모드 없이 조용히 종료된다 (crates/app/src/tray.rs:94-99)
  정상 모드의 `TrayIconBuilder` 실패는 `build()`가 `None`을 반환한 뒤 이벤트 루프를 `Exit`할 뿐이다. 오류 파일 생성, `run_error`, 기본 앱 fallback이 없고 Windows release는 콘솔도 숨겨져 사용자는 시작 실패 사유를 볼 수 없다. 라운드 1 fallback은 이미 오류 모드에 진입한 뒤의 트레이 실패만 처리한다.
  Recommendation: 정상 트레이 구성도 `Result`로 처리해 실패 시 사유 파일을 만들고 직접 열거나 동등한 가시적 fallback을 실행한다. `TrayIconBuilder` 실패 주입 테스트를 추가한다.
- [medium] 트레이 아이콘·툴팁 갱신 실패가 화면과 내부 건수를 분리한다 (crates/app/src/tray.rs:102-111)
  `set_icon`과 `set_tooltip`의 두 `Result`를 모두 버린다. 예를 들어 Windows의 `Shell_NotifyIconW(NIM_MODIFY)` 실패 시 pending은 증가하지만 점 또는 건수 툴팁 한쪽만 갱신되거나 둘 다 이전 상태로 남으며 로그·재시도도 없다. 라운드 1 테스트는 프록시에 보낸 숫자만 검사하므로 실제 트레이 실패를 탐지하지 않는다.
  Recommendation: 두 결과를 검사하고 부분 실패를 기록한다. 갱신 실패 시 트레이 재생성 또는 일관된 오류 상태로 전환하며 backend 실패 주입 테스트를 둔다.
- [medium] 개정 1과 plan 본문이 서로 반대 계약을 선언한다 (docs/tasks/03.native-free/plan.md:31-32)
  설계 본문은 여전히 `GET /api/events` 연결 시 `clear()`하라고 명시하지만, 개정 기록은 GET 무부작용과 `POST /api/seen`을 요구한다. 이 문서는 스스로 '구현자는 이 문서만 보고 일한다'고 선언하므로 향후 구현자가 라운드 1의 CSRF 수정사항을 되돌릴 수 있는 보안 계약 모순이다.
  Recommendation: 설계 본문을 `GET /api/events` 무부작용 및 `POST /api/seen` 계약으로 수정하고, `stage1.md`의 구형 설명·테스트 건수도 개정 1과 동기화한다.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| OFF ↔ on_event 경쟁 | 사실 | 켜짐 · 건수를 한 잠금(`Status`)으로 — 켜짐 확인 · 증가 · 표시 · 전송 · 끄기가 한 순서. 회귀 `turning_off_during_events_leaves_nothing_pending`(8 스레드 × 3000 이벤트 중 끄기, 30회) — 고치기 전 코드에서 round 0 에 FAIL(RED) 확인 후 GREEN |
| 정상 트레이 생성 실패 무표시 | 사실 | `build` 가 `Result` — 실패하면 사유 파일을 쓰고 OS 기본 앱으로 연 뒤 정리(`on_quit`)하고 끝냄. 생성 실패 주입 테스트는 미추가(아래 미조치) |
| 트레이 갱신 실패 무시 | 사실 | `set_icon` · `set_tooltip` 실패를 건수와 함께 로그. 재생성 · 주입 테스트는 미추가(아래 미조치) |
| plan 본문 ↔ 개정 1 모순 | 사실 | 설계 본문을 GET 무부작용 · `POST /api/seen` · guard Origin 계약으로 정정, `stage1.md` 동기화(plan 개정 2) |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E8 수동 gate 미충족 | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 트레이 생성 실패 주입 테스트(라운드 1 · 2) | 사실 — 코드는 고침 | tray-icon 생성을 대역으로 바꾸려면 트레이 추상화가 필요하다 — 실패 경로는 수동(E8)에서 확인 |
| 트레이 갱신 실패 시 재생성 · 주입 테스트(라운드 2) | 사실 — 로그만 | OS 트레이 갱신 실패는 드물고 재생성은 같은 OS 호출을 반복한다. 건수는 웹UI 가 정본(트레이는 보조 표시) |
| 브라우저 열기 실패 · 권한 거부 안내 자동 테스트(라운드 1) | 사실 — 코드는 고침 | 트레이 이벤트 루프 · JS 테스트 기반이 없다 — JS 기반 도입 시(#00 R1-F4 와 같은 조건) |

