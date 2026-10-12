# an_app

## 개요

`coworkagent-app`(`crates/app`, lib `an_app` · bin `coworkAgent`) — app Layer. 로컬 웹UI 서버 · 단일 인스턴스 핸드오프 ·
트레이 · 브라우저 열기 · 에이전트 · 대화 API · 사용자 AI 에이전트용 CLI 와 시작 순서(`main.rs`).

## 공개 계약

- `server::bind()` — `127.0.0.1:0`. `server::new_token()` — 32바이트 hex. `server::router(port, token, status, open_browser)` ·
  `router_with(…, extra)` — `extra` 경로도 같은 guard 뒤.
- `peers_api::routes(node)` — `GET /api/me` · `GET/POST /api/peers`(본문 `{addr, code}` 또는 `{invite}` — 초대는 지문 대조 ·
  추가 · 내 승인, 주소+코드 직접 추가도 코드가 맞아 상대가 승인했으면 내 승인까지 — 양쪽 페어링(D8 개정), 아는 키 다시 추가는 승인 대기) ·
  `PATCH /api/peers/{id} {group?, model?, effort?, alias?, icon?}`(빈 문자열 = 지움, 틀린 값 400 — 이 PC 에만, 피어 JSON 에 같은 이름) ·
  `DELETE /api/peers/{id}` → `{notified}` · `POST /api/peers/{id}/approve` · `/ping`(204 — 받은 `Pong` 은 접속 상태로) · 피어 JSON 에 `online`
  (마지막 `Pong` 75초 안) · `focus`(온라인일 때만 — #18) ·
  `POST /api/pairing-window`(초대 생성) → `{seconds, code, invite}`. 오류 404 · 400 · 502, JSON 아닌 본문 415.
- `chat_api::routes(chat)` — `GET/POST /api/agents`(이름 1 ~ 64자 · 폴더는 있는 절대 경로 · 같은 이름 409 · 도구 비면 읽기 전용 · 목록 밖 도구 400 — D31) ·
  `PATCH /api/agents/{id} {folder?, tools?, prompt?, group?}`(이름은 바꾸지 않는다 — 주면 400) · 에이전트 JSON 에 `group` ·
  `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` → `{names}` · `GET/POST /api/conversations`(목록 = 요약 `title` · `last_state` · `last_at`;
  `{peer_id, agent, text, title?}` → 201 `{conversation, message}`) · `PATCH /api/conversations/{id} {title}`(이 PC 에만, 빈 문자열 = 지움) ·
  `GET /api/messages?side=theirs|mine&peer=&kind=human|agent&q=&limit=`(대화를 가로지른 글 — 거른 뒤 최신순, 기본 200 · 최대 500) · `GET/POST /api/conversations/{id}/messages`(메시지에 `created_at` · `reply_to` ·
  `turns` · `duration_ms` · `mine` · `exec` · `via`; 새 대화 · 이어 쓰기 본문의 `via` 는 `cli` 만(그 밖 400, 자동 왕복 시작에 주면 400); `to` 는 생략 또는 `agent` 만(`human` · 그 밖 400 — 새 대화 ·
  이어 쓰기 모두, D23 개정); 이어 쓰기는 양쪽 대화 `{text, to?}`, 실행은 받았지만 미러 실패면 201 + `warning`;
  새 대화 `relay_agent?` · `relay_cap?` → 자동 왕복, 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`) ·
  `POST /api/conversations/{id}/relay {on}`(멈춤 · 재개, 받은 쪽 재개 · 보통 대화는 400). 오류 404 · 400 · 409 ·
  403(페어링 아님 · 상대 거절, 본문에 코드) · 502.
  작업 화면(#17, 이 PC 에만): `GET /api/search?q=&limit=` → `{messages, conversations, peers, agents}`(1 ~ 100자) · `GET /api/activity?peer=&kind=all|human|
  agent|relay|failed&q=&limit=` → `{items, unseen, unseen_all, upto{message, event}}`(`limit=0` = 수만) · `POST /api/activity/seen {peer?, message, event}`
  (받은 `upto` 까지) · `GET /api/saved?peer=&conversation=&limit=` · `PATCH /api/messages/{id} {saved}` · `PATCH /api/conversations/{id} {title?, starred?}` ·
  `POST /api/conversations/{id}/read {upto}`(받은 마지막 글 id — 그 대화 글이 아니면 400). 요약 JSON 에 `starred` · `unread`, 메시지에 `model` · `effort` · `saved`.
  #18: 새 대화 · 이어 쓰기 본문 `request?`(요청 — 기본 질의), 이어 쓰기 `agent?`(멘션 — 보낸 대화 · 자동 왕복 아님, 대화 에이전트면 멘션 아님, 받은
  대화 · 자동 왕복 · 빈 이름 400) · `PATCH /api/messages/{id} {saved?, done?}`(하나 이상 — 없으면 400, `done` 은 받은 요청만, 둘 다면 한 트랜잭션,
  응답은 준 항목만 `{id, saved?, done?, warning?}`) · `GET /api/messages` 의 `ask=query|request`(사람 글만, 그 밖 400) · `GET /api/activity` 종류
  `query` · `request` · 메시지 JSON 에 `request` · `done` · `agent` · `refused` · 설정 `focus`(`GET/PUT /api/settings` — 켜면 알림도 멈춘다).
  거절(`focus` 등)은 403 + 글 `refused`. CLI `ask --conv N --agent X` — X 가 대화 에이전트가 아니면 멘션(자동 왕복 대화는 오류, 상대 PC 목록에 없으면
  사용법 오류), `history` 발화자 = 실행한 에이전트.
- guard 순서: Host(`127.0.0.1:{port}`) 403 → Origin(있으면 일치) 403 → 진입 `/?t=` (쿠키 `an_token` HttpOnly · SameSite=Strict 설정 후 `/` 로
  303) → 쿠키 또는 `Authorization: Bearer` 401 → 쿠키 인증의 GET/HEAD 밖 요청은 Origin 필수 403(Bearer 예외). 경로 `/` · `/app.js` · `/view.js` · `/app.css` · `/favicon.ico` · `/api/status`(`version` · `data_dir` · `db` · `cli_api` = `server::CLI_API`(1) — CLI 가 보내기 전 확인) · `POST /api/open`.
- `POST /api/open` — `open_browser()` 가 `Err` 면 500, 아니면 204. 두 번째 실행은 5xx 를 재시도 없이 실패 종료.
- `notify::Notifier::new(conn, attention)` — 기본 OFF(`settings` `notify_inbound`). 켜짐 · 세부 설정(`Prefs`) · 건수가 한 잠금:
  `on_event`(켜져 있고 `Prefs` 가 허락하면 건수 +1 → `attention(n)` · broadcast 전송, 막히지 않음 — 걸러진 것은 건수에도 안 든다) ·
  `clear()` · `set_enabled`(끄면 건수 0) · `prefs()` · `update(&SettingsPatch)`(검증 → 한 트랜잭션, 틀리면 `InvalidInput` · 아무것도 안
  바꿈) · `relay_cap_default()` · `run_defaults()` · `screen()`(설정 DB 읽기 오류는 `Err` — 기본값으로 감추지 않는다) · `subscribe()` · `pending()`. `Prefs` = `notify_pair` · `notify_request` · `notify_reply`(기본 켬) ·
  `notify_relay`(`start_end` 기본 · `every` · `off`) · `notify_preview`(기본 켬 — 끄면 본문 = 이름 · PC 만, 끝 사유는 보임) · `notify_paused`(기본 끔 —
  켜 있는 동안 알림 · 트레이 표시 · 이벤트를 버린다, 건수 그대로).
  `notify::routes` — `GET/PUT /api/settings`(PUT 은 준 항목만 · 모르는 항목 · 틀린 값 400, `relay_cap_default` 1 ~ 50, `claude_model` ·
  `claude_effort`(실행 기본값 `sonnet` · `high`) · `ui_theme`(`dark` 기본 · `light`) · `ui_side_width`(200 ~ 560) · `me_alias` · `me_icon`(빈 = 지움),
  설정 DB 읽기 오류는 500) ·
  `GET /api/events`(SSE, 상태 변경 없음) · `POST /api/seen`(지움). 상한 없는 자동 왕복 새 대화는 `relay_cap_default`.
- `handoff::request_open(&InstanceInfo)` — 전체 5초 상한(`HANDOFF_DEADLINE`). `handoff::local_request(info, method, path, body, deadline) ->
  (상태, 본문)` — 원시 HTTP/1.1(bearer · `Connection: close` · 끝까지 읽기, 전체 상한). HTTP 아닌 응답은 `InvalidData`, 보낸 뒤 읽기 오류 ·
  빈 응답은 `UnexpectedEof`(받았는지 모름).
- `cli`(D14 보강) — `main()` 은 인자가 있으면 트레이 · 잠금 · 로거보다 먼저 `cli::main(args)` 로 끝낸다(주 인스턴스가 되지 않는다).
  `coworkAgent peers [--json]` · `ask <상대> "<질문>" [--agent] [--conv] [--no-wait] [--json]` · `history [<상대>] [--conv] [--json]`
  (`say` 는 #15 에서 삭제 — 모르는 명령 2), `--` 뒤는 전부 인자. `<상대>` = 페어링된 피어 이름(대소문자 무시) 또는 지문 hex 앞자리(4자 이상),
  겹치면 2. `ask` 는 `reply_to` = 질문 로컬 id 인 답을 1초 간격으로 30분(`WAIT_LIMIT`)까지 기다린다. 종료 코드 `EXIT_USAGE` 2 · `EXIT_NOT_RUNNING` 3(`instance.json` 없음 · 연결 거부 · HTTP 아님) · `EXIT_API` 4(401 · 4xx ·
  5xx · `cli_api` 불일치 · 보낸 뒤 응답 유실 — 다시 보내지 말고 `history` 확인) · `EXIT_REPLY_FAILED` 5 · `EXIT_TIMEOUT` 6.
  `cli::run(args, instance_file, out, err, wait_limit)` — 테스트 진입(임시 경로 · 짧은 대기).
- `browser::open(url 또는 파일 경로)` — Windows `ShellExecuteW`(반환 ≤ 32 = 실패) · macOS `open` 종료 상태(두 OS 같은 동작 — D30).
- `tray::Tray::new() -> (Tray, Attention)` — 메인 스레드 이벤트 루프. `Attention::set(n)` 은 프록시로 "새 요청" 표시(주황 점 · 툴팁).
  `Tray::run(url, on_seen, on_quit)` — "열기" · "종료"(S4 개정 — 내 정보는 웹UI 설정 `#me`), 열기 성공 시 `on_seen`. 트레이를 못 만들면
  사유 파일을 열고 끝냄. `Tray::run_error(message, file)` — 붉은 아이콘 · "오류 보기" · "종료". `write_start_error(message)` — 임시 폴더
  `coworkAgent-start-error.txt`. 종료 순서: 런타임 정지 → `instance.json` 삭제 → 잠금 해제.
- 시작 실패는 두 OS 모두 트레이 오류 모드(+ stderr). 시작 시 `Chat::attach` · `Chat::recover`(남은 대화 대기열).

- 웹UI(`web/`, 빌드 없는 ES 모듈 — D10): 시안 v4(Nocturne) 5영역(상단 바 · 레일 · 사이드바 · 대화 · 실행 상세) + 설정 + 새 대화, 화면은 해시로만
  바뀐다(`#c/{id}`(`/m/{글}`) 대화 · `#s/{id}` 보낸 메시지 · `#p/{id}` 상대와 주고받은 글 · `#activity` 내 활동 · `#saved` 저장된 항목 · `#agents`
  에이전트 · `#settings/{절}` · `#me` · `#new`, 옛 `#inbox` · `#t/` 는 첫 화면). 색 · 글꼴 크기는 `app.css` 첫 `:root` 토큰만(다크 값은 `--dark-*` 토큰을
  `:root[data-theme="dark"]` 가 가리킨다). 상단 바 통합 검색(Ctrl K) · "+ 새로 만들기" · 알림(내 활동) · 프로필 메뉴, 사이드바 내비 · ★ 즐겨찾기 ·
  대화 섹션, 메시지 작업(저장 · 복사 · 다시 보내기) · 입력창 서식 · Enter 보내기, 실행 상세(진행 · 대화 정보 · 자동 왕복 · 저장된 항목). 타일('나' · 상대)을 누르면 사이드바에 그 타일 대화
  목록, 메시지 본문은 markdown 뷰어 · HTML 문서는 `<iframe sandbox srcdoc>`(머리글 "원문 보기" 하나로 전체 전환), 설정은 전체 폭 트리. 판단 로직은 `view.js`(DOM 없음) — `node --test` 로 시험(`view.test.mjs`),
  `app.js` 는 조립만(`textContent` · `replaceChildren`).

## 사용 예

```rust
let listener = server::bind().await?;
let port = listener.local_addr()?.port();
let app = server::router(port, server::new_token(), status, move || browser::open(&url));
axum::serve(listener, app).await?;
```

```text
coworkAgent peers --json
coworkAgent ask user-b "이 저장소의 빌드 명령은?" --agent review
coworkAgent ask user-b -- "--json 옵션은 무엇을 하나?"
coworkAgent history user-b --conv 12
```

## 금지 사항

- 웹UI 값은 `textContent` 로만 그린다(HTML 주입 금지).
- CLI 종료 코드 4 중 "전달 여부를 모른다" 는 재시도하지 않는다 — 상대가 이미 실행했을 수 있다.
