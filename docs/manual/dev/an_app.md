# an_app

## 개요

`coworkagent-app`(`crates/app`, lib `an_app` · bin `coworkAgent`) — app Layer. 로컬 웹UI 서버 · 단일 인스턴스 핸드오프 ·
트레이 · 브라우저 열기 · 에이전트 · 대화 API 와 시작 순서(`main.rs`).

## 공개 계약

- `server::bind()` — `127.0.0.1:0`. `server::new_token()` — 32바이트 hex. `server::router(port, token, status, open_browser)` ·
  `router_with(…, extra)` — `extra` 경로도 같은 guard 뒤.
- `peers_api::routes(node)` — `GET /api/me` · `GET/POST /api/peers`(본문 `{addr, code}` 또는 `{invite}` — 초대는 지문 대조 ·
  추가 · 내 승인) · `DELETE /api/peers/{id}` → `{notified}` · `POST /api/peers/{id}/approve` · `/ping` ·
  `POST /api/pairing-window` → `{seconds, code, invite}`. 오류 404 · 400 · 502, JSON 아닌 본문 415.
- `chat_api::routes(chat)` — `GET/POST /api/agents`(이름 1 ~ 64자 · 폴더는 있는 절대 경로 · 같은 이름 409 · 도구 비면 읽기 전용 · 목록 밖 도구 400 — D31) ·
  `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` → `{names}` · `GET/POST /api/conversations`(목록 = 요약 `title` · `last_state` · `last_at`;
  `{peer_id, agent, text}` → 201 `{conversation, message}`) · `GET/POST /api/conversations/{id}/messages`(메시지에 `created_at` · `reply_to` ·
  `turns` · `duration_ms` · `mine` · `exec`; 이어 쓰기는 양쪽 대화 `{text, to: agent|human}`, 실행은 받았지만 미러 실패면 201 + `warning`;
  새 대화 `relay_agent?` · `relay_cap?` → 자동 왕복, 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`) ·
  `POST /api/conversations/{id}/relay {on}`(멈춤 · 재개, 받은 쪽 재개 · 보통 대화는 400). 오류 404 · 400 · 409 ·
  403(페어링 아님 · 상대 거절, 본문에 코드) · 502.
- guard 순서: Host(`127.0.0.1:{port}`) 403 → Origin(있으면 일치) 403 → 진입 `/?t=` (쿠키 `an_token` HttpOnly · SameSite=Strict 설정 후 `/` 로
  303) → 쿠키 또는 `Authorization: Bearer` 401 → 쿠키 인증의 GET/HEAD 밖 요청은 Origin 필수 403(Bearer 예외). 경로 `/` · `/app.js` · `/view.js` · `/app.css` · `/api/status` · `POST /api/open`.
- `POST /api/open` — `open_browser()` 가 `Err` 면 500, 아니면 204. 두 번째 실행은 5xx 를 재시도 없이 실패 종료.
- `notify::Notifier::new(conn, attention)` — 기본 OFF(`settings` `notify_inbound`). 켜짐 · 세부 설정(`Prefs`) · 건수가 한 잠금:
  `on_event`(켜져 있고 `Prefs` 가 허락하면 건수 +1 → `attention(n)` · broadcast 전송, 막히지 않음 — 걸러진 것은 건수에도 안 든다) ·
  `clear()` · `set_enabled`(끄면 건수 0) · `prefs()` · `update(&SettingsPatch)`(검증 → 한 트랜잭션, 틀리면 `InvalidInput` · 아무것도 안
  바꿈) · `relay_cap_default()` · `subscribe()` · `pending()`. `Prefs` = `notify_pair` · `notify_request` · `notify_reply`(기본 켬) ·
  `notify_relay`(`start_end` 기본 · `every` · `off`) · `notify_preview`(기본 켬 — 끄면 본문 = 이름 · PC 만, 끝 사유는 보임).
  `notify::routes` — `GET/PUT /api/settings`(PUT 은 준 항목만 · 모르는 항목 · 틀린 값 400, `relay_cap_default` 1 ~ 50) ·
  `GET /api/events`(SSE, 상태 변경 없음) · `POST /api/seen`(지움). 상한 없는 자동 왕복 새 대화는 `relay_cap_default`.
- `handoff::request_open(&InstanceInfo)` — 전체 5초 상한(`HANDOFF_DEADLINE`), 상태 줄만 읽는다.
- `browser::open(url 또는 파일 경로)` — Windows `ShellExecuteW`(반환 ≤ 32 = 실패) · macOS `open` 종료 상태(두 OS 같은 동작 — D30).
- `tray::Tray::new() -> (Tray, Attention)` — 메인 스레드 이벤트 루프. `Attention::set(n)` 은 프록시로 "새 요청" 표시(주황 점 · 툴팁).
  `Tray::run(url, on_seen, on_quit)` — "열기" · "내 정보 보기"(`{url}#me`) · "종료", 열기 성공 시 `on_seen`. 트레이를 못 만들면
  사유 파일을 열고 끝냄. `Tray::run_error(message, file)` — 붉은 아이콘 · "오류 보기" · "종료". `write_start_error(message)` — 임시 폴더
  `coworkAgent-start-error.txt`. 종료 순서: 런타임 정지 → `instance.json` 삭제 → 잠금 해제.
- 시작 실패는 두 OS 모두 트레이 오류 모드(+ stderr). 시작 시 `Chat::attach` · `Chat::recover`(남은 대화 대기열).

- 웹UI(`web/`, 빌드 없는 ES 모듈 — D10): 시안 A 3단(레일 · 사이드바 · 대화) + 설정 + C 타임라인 + 새 대화, 화면은 해시로만 바뀐다
  (`#c/{id}` · `#t/{id}` · `#settings` · `#me` · `#new`). 판단 로직은 `view.js`(DOM 없음) — `node --test` 로 시험(`view.test.mjs`),
  `app.js` 는 조립만(`textContent` · `replaceChildren`).

## 사용 예

```rust
let listener = server::bind().await?;
let port = listener.local_addr()?.port();
let app = server::router(port, server::new_token(), status, move || browser::open(&url));
axum::serve(listener, app).await?;
```

## 금지 사항

- 웹UI 값은 `textContent` 로만 그린다(HTML 주입 금지).
