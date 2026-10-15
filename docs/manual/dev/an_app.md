# an_app

## 개요

`coworkagent-app`(`crates/app`, lib `an_app` · bin `coworkAgent`) — app Layer. 로컬 웹UI 서버 · 단일 인스턴스 핸드오프 ·
트레이 · 브라우저 열기 · 에이전트 · 대화 API · 사용자 AI 에이전트용 CLI 와 시작 순서(`main.rs`).

## 공개 계약

- `server::bind()` — `127.0.0.1:0`. `server::new_token()` — 32바이트 hex. `server::router(port, token, status, open_browser)` ·
  `router_with(…, extra)` — `extra` 경로도 같은 guard 뒤.
- `peers_api::routes(node)` — `GET /api/me`(#1b: `ui_port` — 요청 Host 의 포트 · `peer_port_fixed`(47100 인가) · `peer_port_holder{name, pid}|null` ·
  `peer_port_note` · `open_connections`) · `GET/POST /api/peers`(본문 `{addr, code}` 또는 `{invite}` — 초대는 지문 대조 ·
  추가 · 내 승인, 주소+코드 직접 추가도 코드가 맞아 상대가 승인했으면 내 승인까지 — 양쪽 페어링(D8 개정), 아는 키 다시 추가는 승인 대기) ·
  `PATCH /api/peers/{id} {group?, model?, effort?, alias?, icon?}`(빈 문자열 = 지움, 틀린 값 400 — 이 PC 에만, 피어 JSON 에 같은 이름) ·
  `DELETE /api/peers/{id}` → `{notified}` · `POST /api/peers/{id}/approve` · `/ping`(204 — `Node::refresh(id, 60)`: 60초 안에 통신했으면 보내지
  않음, 받은 `Pong` 은 접속 상태로 — #1c) · 피어 JSON 에 `online`(마지막 통신 75초 안) · `focus`(온라인일 때만 — #18) · `seen_at`(마지막으로
  주고받기에 성공한 유닉스 초 · 없으면 `null` — 화면이 "n분 전 통신" 을 센다, #1c) ·
  `POST /api/pairing-window`(초대 생성) → `{seconds, code, invite}`. 오류 404 · 400 · 502, JSON 아닌 본문 415.
- `chat_api::routes(chat)` — `GET/POST /api/agents`(이름 1 ~ 64자 · 폴더는 있는 절대 경로 · 데이터 루트를 품거나 네트워크 경로면 400(`an_core::paths::agent_folder_problem`, #1a) · 같은 이름 409 · 도구 칸 없음 — `tools` 를 주면 422(#1d)) ·
  `PATCH /api/agents/{id} {folder?, prompt?, group?}`(이름은 바꾸지 않는다 — 주면 400) · 에이전트 JSON 에 `group` ·
  `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` → `{names}` · `GET/POST /api/conversations`(목록 = 요약 `title` · `last_state` · `last_at`;
  `{peer_id, agent, text, title?}` → 201 `{conversation, message}`) · `PATCH /api/conversations/{id} {title}`(이 PC 에만, 빈 문자열 = 지움) ·
  `GET /api/messages?side=theirs|mine&peer=&kind=human|agent&q=&limit=`(대화를 가로지른 글 — 거른 뒤 최신순, 기본 200 · 최대 500) · `GET /api/runs` `{cap, running, waiting}`(항목 `message_id` · `conversation_id` · `peer_id` · `peer` · `agent` · `text` 앞 40자 — #1b) ·
  `PUT /api/runs {cap}`(정수 1 ~ 8 아니면 400, 저장 + 문에 바로) · `GET/POST /api/conversations/{id}/messages`(대화 글 목록 메시지에
  `progress{state, ahead, detail}|null`(#1b · `detail` = 단계 요약 `{turns, tools, secs}`|null — #1c) · `outbox{state, tries, error}|null`(#1c — 상대에
  아직 닿지 않았거나(`pending` · `inflight`) 포기한(`failed`) 내 글) · `created_at` · `reply_to` ·
  `turns` · `duration_ms` · `mine` · `exec` · `via`; 새 대화 · 이어 쓰기 본문의 `via` 는 `cli` 만(그 밖 400, 자동 왕복 시작에 주면 400); `to` 는 생략 또는 `agent` 만(`human` · 그 밖 400 — 새 대화 ·
  이어 쓰기 모두, D23 개정); 이어 쓰기는 양쪽 대화 `{text, to?}`, 실행은 받았지만 미러 실패면 201 + `warning`;
  새 대화 `relay_agent?` · `relay_cap?` → 자동 왕복, 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`) ·
  `POST /api/conversations/{id}/relay {on}`(멈춤 · 재개, 받은 쪽 재개 · 보통 대화는 400). 오류 404 · 400 · 409 ·
  403(페어링 아님 · 상대 거절, 본문에 코드) · 502. 상대에 닿지 않은 새 글은 오류가 아니다 — 보관하고 201(글 `outbox`).
  CLI 트랜잭션(#1c): 새 대화 · 이어 쓰기 본문 `tx_key?`(`via: cli` 일 때만 — 아니면 400, 규칙 `an_core::cli_tx::key_ok` 밖 400) — 같은 키가 있으면
  보내지 않고 그 트랜잭션 JSON 을 200, 없으면 보내고 201 `{conversation, message, tx}`(`via: cli` 면 트랜잭션을 만든다 — 키 없어도, 기록 실패면
  `tx: null`). `GET /api/tx/{id}` → `{tx, key, state, conversation, message, reply, progress, outbox, stalled}`(`state` = `sending` · `sent` ·
  `queued` · `running` · `answered` · `failed` · `stalled` — 조회가 감시도 띄운다, `Chat::tx_view`) · `POST /api/tx/{id}/resume`(멈춤을 풀고 지금
  상태 — `wait`). 없는 트랜잭션 404.
  작업 화면(#17, 이 PC 에만): `GET /api/search?q=&limit=` → `{messages, conversations, peers, agents}`(1 ~ 100자) · `GET /api/activity?peer=&kind=all|human|
  agent|relay|failed&q=&limit=` → `{items, unseen, unseen_all, upto{message, event}}`(`limit=0` = 수만) · `POST /api/activity/seen {peer?, message, event}`
  (받은 `upto` 까지) · `GET /api/saved?peer=&conversation=&limit=` · `PATCH /api/messages/{id} {saved}` · `PATCH /api/conversations/{id} {title?, starred?}` ·
  `POST /api/conversations/{id}/read {upto}`(받은 마지막 글 id — 그 대화 글이 아니면 400). 요약 JSON 에 `starred` · `unread`, 메시지에 `model` · `effort` · `saved`.
  #18: 새 대화 · 이어 쓰기 본문 `request?`(요청 — 기본 질의), 이어 쓰기 `agent?`(멘션 — 보낸 대화 · 자동 왕복 아님, 대화 에이전트면 멘션 아님, 받은
  대화 · 자동 왕복 · 빈 이름 400) · `GET /api/messages` 의 `ask=query|request`(사람 글만, 그 밖 400) · 메시지 JSON 에 `request` · `agent` · `refused` ·
  설정 `focus`(`GET/PUT /api/settings` — 켜면 알림도 멈춘다).
  #1d: `PATCH /api/messages/{id} {saved?, req_state?, note?}`(하나 이상 — 없으면 400, `req_state` = `accepted` · `rejected` · `done` 받은 요청만 ·
  `note` 는 `req_state` 와 함께만, 둘 다면 한 트랜잭션, 응답은 준 항목만 `{id, saved?, req_state?, req_note?, warning?}`) · 메시지 JSON 에
  `req_state` · `req_note` · `req_at` · `GET /api/messages` 의 `req=wait|accepted|rejected|done`(`ask=request` 와 함께만) · `GET /api/activity` 종류는
  `all` · `human` · `agent` · `relay` · `failed` 만 · 요청 · 멘션 · CLI 함께면 400 · 모델은 `opus` · `sonnet` · `haiku` 만(설정 · 상대별).
  #1f: `PATCH /api/messages/{id} {text, mirror?}`(끝난 내 사람 글만 — 그 밖 400, `text` 는 다른 항목과 함께 못 씀 · `mirror` 는 `text` 와 함께만 →
  `{id, text, edited_at, warning?}`) · `POST /api/messages/delete {ids, mirror?}`(1 ~ 200개 · 한 대화 · 하나라도 안 되면 아무것도 안 지움 →
  `{deleted, warning?}`) · 대화 글 JSON 에 `editable`(`Chat::tidy_ok`) · 메시지 JSON 에 `edited_at` · 내 활동 항목 `kind = sync_failed`(`detail` = 사유).
  #1i(`groups_api.rs` — 이 PC 에만, 상대 PC 로 가는 프레임 없음): `GET /api/groups` → `[{id, name, icon, pos, convs}]`(레일 순서) · `POST /api/groups {name}`
  → 201(이름 = 앞뒤 공백 뺀 1 ~ 40자 · 제어 문자 없음 — 400, 같은 이름 409 "같은 이름의 그룹이 있습니다") · `PATCH /api/groups/{id} {name?, icon?}`(둘 다
  검증한 뒤 쓴다, 빈 `icon` = 지움, 없으면 404) · `DELETE /api/groups/{id}` → 204(소속도, 대화는 그대로) · `PUT /api/groups/{id}/conversations {add?,
  remove?}`(이미 있음 · 없음은 무시, 없는 대화 400, 합쳐 10000개까지) · `PUT /api/order {kind: peers|groups|conversations, ids}` → 204(준 순서대로 자리
  0 ..; 겹침 · 없는 id 400 — 아무것도 안 바뀜, 빠진 id 는 제 자리 · 같은 자리면 id 순). `GET /api/messages` · `GET /api/saved` 에 `group`(`peer` 와 함께면
  400), 새 대화 본문 `group?`(없는 그룹 404 — 보내지 않음, 소속은 대화와 같은 트랜잭션), 대화 요약 JSON 에 `pos`, `GET /api/peers` 순서 = 자리.
  거절(`focus` 등)은 403 + 글 `refused`. CLI `ask --conv N --agent X` — X 가 대화 에이전트가 아니면 멘션(자동 왕복 대화는 오류, 상대 PC 목록에 없으면
  사용법 오류), `history` 발화자 = 실행한 에이전트.
- guard 순서(#1a): Host(`127.0.0.1:{port}`) 403 → Origin(있으면 일치) 403 → 정적 자산(`server::PUBLIC` GET — 토큰 없이, 진입 `/?t=` 는 틀리면
  401) → `Authorization: Bearer` 만(쿠키 읽지 않음 · `/api/events` 만 `?t=` 도) 401. 모든 응답(401 · 403 포함)에 `server::CSP` · `X-Frame-Options:
  DENY` · `X-Content-Type-Options: nosniff` · `Referrer-Policy: no-referrer`, `GET /` 는 `Cache-Control: no-store`. 웹UI 는 `view::entryToken` 으로
  주소의 `t` 를 sessionStorage 로 옮기고 주소창에서 지운다 · 401 이면 `#disconnected` 안내. 경로 `/` · `/app.js` · `/view.js` · `/app.css` · `/favicon.ico` · `/api/status`(`version` · `data_dir` · `db` · `cli_api` = `server::CLI_API`(1) — CLI 가 보내기 전 확인) · `POST /api/open`.
- `POST /api/open` — `open_browser()` 가 `Err` 면 500, 아니면 204. 두 번째 실행은 5xx 를 재시도 없이 실패 종료.
- `notify::Notifier::new(conn, attention)` — 기본 OFF(`settings` `notify_inbound`). 켜짐 · 세부 설정(`Prefs`) · 건수가 한 잠금:
  `on_event`(켜져 있고 `Prefs` 가 허락하면 건수 +1 → `attention(n)` · broadcast 전송, 막히지 않음 — 걸러진 것은 건수에도 안 든다) ·
  `clear()` · `set_enabled`(끄면 건수 0) · `prefs()` · `update(&SettingsPatch)`(검증 → 한 트랜잭션, 틀리면 `InvalidInput` · 아무것도 안
  바꿈) · `relay_cap_default()` · `run_defaults()` · `screen()`(설정 DB 읽기 오류는 `Err` — 기본값으로 감추지 않는다) · `subscribe()` · `pending()`. `Prefs` = `notify_pair` · `notify_request` · `notify_reply`(기본 켬) ·
  `notify_relay`(`start_end` 기본 · `every` · `off`) · `notify_preview`(기본 켬 — 끄면 본문 = 이름 · PC 만, 끝 사유는 보임) · `notify_paused`(기본 끔 —
  켜 있는 동안 알림 · 트레이 표시 · 이벤트를 버린다, 건수 그대로).
  `notify::routes` — `GET/PUT /api/settings`(PUT 은 준 항목만 · 모르는 항목 · 틀린 값 400, `relay_cap_default` 1 ~ 50, `claude_model` ·
  `claude_effort`(실행 기본값 `sonnet` · `high`) · `ui_theme`(`dark` 기본 · `light`) · `ui_side_width`(200 ~ 560) · `ui_folded`(접은 절 열쇠 목록 — `{화면}:{상대 id | me}` 300개까지, #1g) · `me_alias` · `me_icon`(빈 = 지움),
  설정 DB 읽기 오류는 500) ·
  `GET /api/events`(SSE, 상태 변경 없음) · `POST /api/seen`(지움). 상한 없는 자동 왕복 새 대화는 `relay_cap_default`.
- `handoff::request_open(&InstanceInfo)` — 전체 5초 상한(`HANDOFF_DEADLINE`). `handoff::local_request(info, method, path, body, deadline) ->
  (상태, 본문)` — 원시 HTTP/1.1(bearer · `Connection: close` · 끝까지 읽기, 전체 상한). HTTP 아닌 응답은 `InvalidData`, 보낸 뒤 읽기 오류 ·
  빈 응답은 `UnexpectedEof`(받았는지 모름).
- `cli::printable(s)`(#1a) — 결과 · 실패 사유 출력에서 C0(줄바꿈 · 탭 제외) · DEL · C1 제거. `main()` 첫 줄은 Windows `SetDefaultDllDirectories(System32)`.
- `cli`(D14 보강) — `main()` 은 인자가 있으면 트레이 · 잠금 · 로거보다 먼저 `cli::main(args)` 로 끝낸다(주 인스턴스가 되지 않는다,
  `is_command` = `peers` · `ask` · `wait` · `status` · `history`). `coworkAgent peers [--json]` · `ask <상대> "<질문>" [--agent] [--conv] [--id <키>]
  [--no-wait] [--json]` · `wait <트랜잭션> [--json]` · `status <트랜잭션> [--json]` · `history [<상대>] [--conv] [--json]`(`USAGE` 가 정본 ·
  `say` 는 #15 에서 삭제 — 모르는 명령 2), `--` 뒤는 전부 인자. `<상대>` = 페어링된 피어 이름(대소문자 무시) 또는 지문 hex 앞자리(4자 이상),
  겹치면 2. `ask` 는 본문에 `via: cli` · `tx_key`(= `--id`)를 싣고 받은 `tx` 를 `wait_tx` 로 본다(#1c — 기다릴 때 `tx` 가 없으면 4). `wait` 은
  `POST /api/tx/{id}/resume` 뒤 `wait_tx`, `status` 는 `GET /api/tx/{id}` 한 번(평문 `{tx}\t{state}\t{진행}` · `--json` 은 응답 그대로).
  `wait_tx` — `GET /api/tx/{id}` 를 1초 간격으로: `answered` → 답 출력 · 0, `failed` → 5(실패 답 "상대 에이전트 실행 실패: …" · 글 실패 "질문을 상대에
  보내지 못했다({거절 코드})"), `stalled` → 7, 안전판 `WAIT_LIMIT` 24시간 → 6. 진행이 바뀔 때마다 표준 오류에 한 줄(평문 "보내는 중 — 상대에 아직
  닿지 않아 이 PC 에 보관했다(다시 보냄 {n}회)" · "대기 중 — 상대 PC 에 처리 중인 작업이 많다(앞에 {k}건)" · "처리 중" · "처리 중 — 2턴 · Read 3 ·
  41초", `--json` 이면 `{"progress":"sending","tries":n}` · `{"progress":"queued","ahead":k}` · `{"progress":"running"[, "detail":{…}]}`). `--json`
  결과 `{conversation, asked, reply, tx}` · `--no-wait --json` 은 `{conversation, asked, tx}`. 1b 의 "queued 동안 시계 멈춤" 은 앱 판정으로 대체.
  종료 코드 `EXIT_USAGE` 2 · `EXIT_NOT_RUNNING` 3(`instance.json` 없음 · 연결 거부 · HTTP 아님) · `EXIT_API` 4(401 · 4xx · 5xx · `cli_api`
  불일치 · 보낸 뒤 응답 유실 — 다시 보내지 말고 `history` 확인) · `EXIT_REPLY_FAILED` 5 · `EXIT_TIMEOUT` 6 · `EXIT_STALLED` 7(상대에 10분 닿지
  않아 멈춤 — 트랜잭션은 남는다, `wait <tx>`).
  `cli::run(args, instance_file, out, err, wait_limit)` — 테스트 진입(임시 경로 · 짧은 대기).
- `browser::open(url 또는 파일 경로)` — Windows `ShellExecuteW`(반환 ≤ 32 = 실패) · macOS `open` 종료 상태(두 OS 같은 동작 — D30).
- `tray::Tray::new() -> (Tray, Attention)` — 메인 스레드 이벤트 루프. `Attention::set(n)` 은 프록시로 "새 요청" 표시(주황 점 · 툴팁).
  `Tray::run(url, note, on_seen, on_quit)`(`note` = 툴팁 둘째 줄 — 피어 포트를 남이 잡았을 때 `peer_port_note`, `tray::tip(n, note)`) — "열기" · "종료"(S4 개정 — 내 정보는 웹UI 설정 `#me`), 열기 성공 시 `on_seen`. 트레이를 못 만들면
  사유 파일을 열고 끝냄. `Tray::run_error(message, file)` — 붉은 아이콘 · "오류 보기" · "종료". `write_start_error(message)` — 임시 폴더
  `coworkAgent-start-error.txt`. 종료 순서: 런타임 정지 → `instance.json` 삭제 → 잠금 해제.
- `watchdog`(#1e) — `ENV`(`COWORKAGENT_SUPERVISED`, 값 = 다시 띄운 횟수 · 없으면 감시 프로세스) · `MAX_CRASHES`(3) · `CRASH_WINDOW`(60초) ·
  `RESTART_DELAY`(1초) · `supervise(spawn: FnMut(u32) -> io::Result<bool>, delay) -> Outcome`(`Done` = 자식 정상 종료 · `GaveUp(사유)` = 창 안 3번 ·
  띄우기 실패) · `Crashes::record(at) -> bool` · `spawn_app(n)`(이 실행 파일 · 인자 없음 · `ENV = n`) · `restarts()`.
- `autostart`(#1e, S7 · D13) — `sync(on)`(이 OS 의 실제 자리: Windows `RUN_KEY` 값 `VALUE_NAME` = `"exe"`, macOS `~/Library/LaunchAgents/{LABEL}.plist` —
  같으면 쓰지 않음 · 끄면 지움 · 없어도 성공) · `launch_agent_plist(exe)`(`RunAtLoad` · `KeepAlive` 없음 · 경로 XML 이스케이프) · Windows
  `registered(key)` · `register(key, exe) -> 바뀜` · `unregister(key)`(키 경로 인자 — 시험은 시험 키). 설정 `autostart`(`GET/PUT /api/settings`, 없으면
  켜짐): `Notifier::set_autostart_hook(f)` 가 있으면 켜기는 `f(true)` → 저장(저장 실패면 `f(이전 값)`), 끄기는 저장 → `f(false)`(실패면 저장을 이전 값으로 되돌림) — 훅 실패는 500.
  중단돼도 "저장 켜짐 · OS 꺼짐" 이 남지 않는 순서(남는 것은 "OS 켜짐 · 저장 꺼짐" 뿐 — 다음 로그인 시작 맞추기가 지움) · 훅 잠금을
  저장 끝까지 쥔다(동시 요청 순서 = 저장 순서). 앱은 릴리스 빌드에서만 훅 설치 · 시작마다 설정 값으로 `sync`. `write_atomic(path, content)` — 같은
  폴더 `.tmp` 에 쓰고 `sync_all` 뒤 이름 바꾸기(실패하면 원래 파일 그대로 — plist).
- 시작 실패는 두 OS 모두 트레이 오류 모드(+ stderr). 시작 시 `Chat::attach` · `Chat::recover`(남은 대화 대기열 · 남은 outbox) · 피어 수신 뒤
  `Node::announce`(시작 알림 — 주기 접속 확인은 없다, #1c).

- 웹UI(`web/`, 빌드 없는 ES 모듈 — D10): 시안 v4(Nocturne) 5영역(상단 바 · 레일 · 사이드바 · 대화 · 실행 상세) + 설정 + 새 대화, 화면은 해시로만
  바뀐다(`#c/{id}`(`/m/{글}`) 대화 · `#s/{id}` 보낸 메시지 · `#p/{id}` 상대와 주고받은 글(`#p` = 모든 상대) · `#activity` 내 활동 · `#saved` 저장된 항목 · `#agents`
  에이전트 · `#settings/{절}` · `#me` · `#new`, 옛 `#inbox` · `#t/` 는 첫 화면). 색 · 글꼴 크기는 `app.css` 첫 `:root` 토큰만(다크 값은 `--dark-*` 토큰을
  `:root[data-theme="dark"]` 가 가리킨다). 상단 바 통합 검색(Ctrl K) · "+ 새로 만들기" · 알림(내 활동) · 프로필 메뉴, 사이드바 내비 · ★ 즐겨찾기 ·
  대화 섹션, 말풍선 우상단 복사 아이콘(늘 보임 — #1f) · 메시지 작업(저장 · 다시 보내기) · 입력창 위 "편집"(편집 모드 — 끝난 내 사람 글 체크박스 ·
  연필, 입력창 자리 `#edit_bar` "n개 선택 · 삭제 · 완료", 반영 범위 `<dialog id="scope_dialog">` "내 창에서만" · "상대 창에도" · 취소 — 본문을 고치는
  동안은 5초 새로 고침이 대화를 다시 그리지 않는다, `view.keepPicked` · `editedTag`) · 입력창 서식 · Enter 보내기, 실행 상세(진행 · 대화 정보 · 자동 왕복 · 저장된 항목). 레일 범위(#1g, `view.sideNav`):
  '나' = 메뉴 6개(대화 · 내 활동 · 요청 · 저장된 항목 · 주고받은 글 · 에이전트), 목록은 `view.peerGroups`(레일 순서 · 풀린 상대 뒤 · 빈 상대 숨김)로 상대별
  `<details>` 절, '대화'(`#`) = `view.peerSummary` 상대별 요약(대기 요청 = `view.waitingByPeer`), 사이드바 = `convSections(rows, paired)` ★ 아래 상대별 절.
  상대 타일 = 메뉴 4개(요청 · 저장된 항목 · 주고받은 글 · 에이전트), 첫 화면 `#p/{id}`, 그 상대 것만 — 상대 타일인 채 `#` 는 `#p/{id}`, `#activity` 는
  '나' 로. 절 접힘은 설정 `ui_folded`(`{화면}:{상대 id | me}`, 시작할 때만 읽는다). 사이드바 에이전트 절은 없다(에이전트 화면만). 레일 그룹(#1i): 그룹 타일 =
  메뉴 4개(`sideNav("group")` — 대화 · 요청 · 저장된 항목 · 주고받은 글 `#p`), 목록 · 요약은 그 그룹 대화만(`view.inGroup` · `groupPeers` — 대화 있는 상대만,
  서버 거르기 `group=`), 그룹인 채 `#activity` · `#agents` 는 '나' 로(`view.groupLeaves`). 접힘 열쇠는 '나' 와 같은 화면 열쇠. 떠 있는 상자 `#float`(화면 기준 —
  "그룹 +" · 대화 ⋯ "그룹" 체크 · "그룹에서 빼기" · "대화 추가" · 타일 메뉴 "타일 편집" · "이미지 지우기" · "그룹 삭제"(상자 안 확인)) — 레일 우클릭 · 메뉴 키 ·
  사이드 머리 ⋯. 순서 = 손으로 놓은 자리(`view.sortByPos` — 대화 · 상대 · 그룹 모두): 끌어 놓기(HTML 기본 드래그, `view.dropKind` — 같은 종류 · 대화는 같은
  절, 대화 → 그룹 타일 = 넣기) · Alt + ↑ / ↓(`view.stepTarget`, `#order_live` 알림 `movedText`) → 전체 순서에 끼워(`view.moveInOrder`) `PUT /api/order`.
  타일 색 = `view.colorOf(id)`. 아이콘 그림은 data URL 로 읽는다(CSP `img-src` 가 `blob:` 을 막는다). 타일을 누르면 사이드바에 그 타일 대화
  목록, 메시지 본문은 markdown 뷰어 · HTML 문서는 `<iframe sandbox srcdoc>`(머리글 "원문 보기" 하나로 전체 전환), 설정은 전체 폭 트리. 판단 로직은 `view.js`(DOM 없음) — `node --test` 로 시험(`view.test.mjs`),
  `app.js` 는 조립만(`textContent` · `replaceChildren`). #1c: `view.presence(p, now)`(피어 `seen_at` → "방금 통신"(75초 안) · "n분 · n시간 · n일 전
  통신" · "통신 기록 없음", 방금일 때만 점 · 집중 "집중 중 · …") · `outboxText`("보내는 중" · "보내는 중 — 상대에 닿지 않아 다시 보냄(n회)") ·
  `stepText`("2턴 · Read 3 · Grep 1 · 41초") · `progressText`("처리 중 — {단계}") · `refusedText`(`unreachable` → "보내지 못함 — …"). `app.js` 는
  웹UI 를 열 때 페어링된 상대 모두 · 레일 타일을 고를 때 그 상대 ping(`pingPeers` — 서버가 60초로 거른다).

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
coworkAgent ask user-b "빌드 명령은?" --agent review --id build-q1 --json
coworkAgent wait 7
coworkAgent status 7 --json
coworkAgent history user-b --conv 12
```

## 금지 사항

- 웹UI 값은 `textContent` 로만 그린다(HTML 주입 금지).
- CLI 종료 코드 4 중 "전달 여부를 모른다" 는 재시도하지 않는다 — 상대가 이미 실행했을 수 있다.
