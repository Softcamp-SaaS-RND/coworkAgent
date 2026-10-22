# an_app

## 개요

`coworkagent-app`(`crates/app`, lib `an_app` · bin `coworkAgent`) — app Layer. 로컬 웹UI 서버 · 단일 인스턴스 핸드오프 ·
트레이 · 브라우저 열기 · 에이전트 · 대화 API · 사용자 AI 에이전트용 CLI 와 시작 순서(`main.rs`).

## 공개 계약

- `server::bind()` — `127.0.0.1:0`. `server::new_token()` — 32바이트 hex. `server::router(port, token, status, open_browser)` ·
  `router_with(…, extra)` — `extra` 경로도 같은 guard 뒤. `router_control(port, token, status, open_browser, mode, quit: Option<Quit>, extra)`(#1n —
  `router_with` 는 `"tray"` · 종료 없음으로 부른다): `/api/status` 에 `mode`(`"tray"` · `"headless"`), `quit` 이 있으면 `POST /api/quit`(204 를 보내고
  100 ms 뒤 `quit()` — `Quit = Box<dyn Fn() + Send + Sync>`).
- `peers_api::routes(node)` — `GET /api/me`(#1b: `ui_port` — 요청 Host 의 포트 · `peer_port_fixed`(47100 인가) · `peer_port_holder{name, pid}|null` ·
  `peer_port_note` · `open_connections`) · `GET/POST /api/peers`(본문 `{addr, code}` 또는 `{invite}` — 초대는 지문 대조 ·
  추가 · 내 승인, 주소+코드 직접 추가도 코드가 맞아 상대가 승인했으면 내 승인까지 — 양쪽 페어링(D8 개정), 아는 키 다시 추가는 승인 대기) ·
  `PATCH /api/peers/{id} {group?, model?, effort?, alias?, icon?}`(빈 문자열 = 지움, 틀린 값 400 — 이 PC 에만, 피어 JSON 에 같은 이름. `group` = 옛 자유 입력
  이름표 — #1j 웹UI 는 쓰지 않는다, 상대의 그룹은 `PUT /api/groups/{id}/peers`) ·
  `DELETE /api/peers/{id}` → `{notified}` · `POST /api/peers/{id}/approve` · `/ping`(204 — `Node::refresh(id, 60)`: 60초 안에 통신했으면 보내지
  않음, 받은 `Pong` 은 접속 상태로 — #1c) · 피어 JSON 에 `online`(마지막 통신 75초 안) · `focus`(온라인일 때만 — #18) · `seen_at`(마지막으로
  주고받기에 성공한 유닉스 초 · 없으면 `null` — 화면이 "n분 전 통신" 을 센다, #1c) ·
  `POST /api/pairing-window`(초대 생성) → `{seconds, code, invite}`. 오류 404 · 400 · 502, JSON 아닌 본문 415.
- `chat_api::routes(chat)` — `GET/POST /api/agents`(이름 1 ~ 64자 · 폴더는 있는 절대 경로 · 데이터 루트를 품거나 네트워크 경로면 400(`an_core::paths::agent_folder_problem`, #1a) · 같은 이름 409 · 도구 칸 없음 — `tools` 를 주면 422(#1d)) ·
  `PATCH /api/agents/{id} {folder?, prompt?, group?, parent_read?, shell?, color?}`(이름은 바꾸지 않는다 — 주면 400 · `color` 는 `agents::ICON_COLORS` 밖이면 400 "color 는 …", #1z-7) ·
  에이전트 JSON 에 `color`(#1z-7) · 에이전트 JSON 에 `group`(#1j 웹UI 는 쓰지 않는다) ·
  등록 `parent_read`(없으면 끔) · 에이전트 JSON `parent_read` · `parent_dir`(#05-2 — 실제 여는 위 폴더, 끔 · 막힌 곳이면 `null`) ·
  등록 · 수정 `shell`(#05-3 — 없으면 끔 · 그대로) · 에이전트 JSON `shell` — 웹UI 는 끔 → 켬으로 보낼 때만 `window.confirm(SHELL_WARN)`(`view.js` `shellConfirmNeeded`, 취소 = 보내지 않음) ·
  `DELETE /api/agents/{id}` · `GET /api/peers/{id}/agents` → `{names}` · `GET/POST /api/conversations`(목록 = 요약 `title` · `last_state` · `last_at` · `last{text, author, mine, agent, via}|null`(#1j — 마지막 글, `text` 앞 120자);
  `{peer_id, agent, text, title?}` → 201 `{conversation, message}`) · `PATCH /api/conversations/{id} {title}`(이 PC 에만, 빈 문자열 = 지움) ·
  `GET /api/messages?side=theirs|mine&peer=&kind=human|agent&q=&limit=`(대화를 가로지른 글 — 거른 뒤 최신순, 기본 200 · 최대 500) · `GET /api/runs` `{cap, running, waiting}`(항목 `message_id` · `conversation_id` · `peer_id` · `peer` · `agent` · `text` 앞 40자 — #1b) ·
  `PUT /api/runs {cap}`(정수 1 ~ 8 아니면 400, 저장 + 문에 바로) · `GET/POST /api/conversations/{id}/messages`(GET `limit` = 마지막 N 개만, 오래된 것부터 — 1 ~ 500 밖 400, #1z-4 개요 미리보기 · 대화 글 목록 메시지에
  `progress{state, ahead, detail}|null`(#1b · `detail` = 단계 요약 `{turns, tools, secs}`|null — #1c) · `outbox{state, tries, error}|null`(#1c — 상대에
  아직 닿지 않았거나(`pending` · `inflight`) 포기한(`failed`) 내 글) · `created_at` · `reply_to` ·
  `turns` · `duration_ms` · `mine` · `exec` · `via`; 새 대화 · 이어 쓰기 본문의 `via` 는 `cli` 만(그 밖 400, 자동 왕복 시작에 주면 400); `to` 는 생략 또는 `agent` 만(`human` · 그 밖 400 — 새 대화 ·
  이어 쓰기 모두, D23 개정); 이어 쓰기는 양쪽 대화 `{text, to?}`, 실행은 받았지만 미러 실패면 201 + `warning`;
  새 대화 `relay_agent?` · `relay_cap?` → 자동 왕복, 대화 JSON 에 `relay_agent` · `relay_cap` · `relay_count` · `relay_on` · `relay_blocked`) ·
  `POST /api/conversations/{id}/relay {on}`(멈춤 · 재개, 받은 쪽 재개 · 보통 대화는 400) · `POST /api/conversations/{id}/context {op}`(#1m — `compact` ·
  `clear` → 구분선 글, 틀린 값 400 · 없는 대화 404 · 진행 중 409; 글 JSON 에 `ctx_tokens` · `ctx_op`). #1o 공유: `POST /api/conversations/{id}/observers {peer_id}` ·
  `DELETE /api/conversations/{id}/observers/{peer}` · `POST /api/conversations/{id}/leave`(204 · 404 · 409 — `fail` 이 `Unsupported` · `ResourceBusy` 도 409, #1w 부터 모든 대화), 대화 목록 JSON
  `observers` · `watchers` · `share{other, owner_dir, ended}`, 글 JSON `owner`. 웹 — `view.js` `CONV_GROUPS`(내 대화 · 에이전트 대화 · 공유 대화) · `splitByDir` ·
  `splitKey` · `sharedAuthor` · `shareHead` · `shareCandidates` · `MAX_OBSERVERS`, `app.js` `renderShare`(세부정보 "관람자") · 사본 읽기 전용 · `#rail` 우클릭 막기.
  CLI `author_label` 공유 갈래. #1w 나가기 · 삭제: `POST /api/conversations/{id}/leave` = 모든 대화(`Chat::leave_conversation`) · `DELETE /api/conversations/{id}`
  (`Chat::delete_conversation`) → 204 · 없는 대화 404 · 도는 실행 409 "실행이 끝난 뒤 나갈 수 있습니다", 대화 JSON(`conversation_json` — 목록 · 한 대화) · 글의
  `conversation` 요약(요청 · 내 활동 · 가로지른 글)에 `left_at` · `peer_left_at`. 프레임 `Leave{conv}` · `Pong.leave` · 오류 코드 `left` 는 [an_net](an_net.md).
  웹 — `view.js` `convMenuItems` · `convClosed` · `refusedText`(`left`) · `reqMoves(m, closed)` · `reqResend(m, closed)` · `messageActions(m, closed)` ·
  `settingsGroupSections` · `USER_LIST` · `convCopy`, `app.js` `openConvMenu` 끝 나가기 · 삭제(`convEndForm`) · 닫힌 대화 읽기 전용 · `renderPeerList`(`groupSection` ·
  `userListSection` · `settingsConvRow` — 끌기 = 복사). 오류 404 · 400 · 409 ·
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
  #1d: `PATCH /api/messages/{id} {saved?, req_state?, note?}`(하나 이상 — 없으면 400, `req_state` = `wait`(#1y) · `accepted` · `rejected` · `done` 받은 요청만 ·
  `note` 는 `req_state` 와 함께만, 둘 다면 한 트랜잭션, 응답은 준 항목만 `{id, saved?, req_state?, req_note?, warning?}`) · 메시지 JSON 에
  `req_state` · `req_note` · `req_at` · `GET /api/messages` 의 `req=wait|accepted|rejected|done`(`ask=request` 와 함께만) · `GET /api/activity` 종류는
  `all` · `human` · `agent` · `relay` · `failed` 만(#1q 에서 바뀜 — 아래) · 요청 · 멘션 · CLI 함께면 400 · 모델은 `opus` · `sonnet` · `haiku` 만(설정 · 상대별).
  #1f: `PATCH /api/messages/{id} {text, mirror?}`(끝난 내 사람 글만 — 그 밖 400, `text` 는 다른 항목과 함께 못 씀 · `mirror` 는 `text` 와 함께만 →
  `{id, text, edited_at, warning?}`) · `POST /api/messages/delete {ids, mirror?}`(1 ~ 200개 · 한 대화 · 하나라도 안 되면 아무것도 안 지움 →
  `{deleted, warning?}`) · 대화 글 JSON 에 `editable`(`Chat::tidy_ok`) · 메시지 JSON 에 `edited_at` · 내 활동 항목 `kind = sync_failed`(`detail` = 사유).
  #1i(`groups_api.rs` — 이 PC 에만, 상대 PC 로 가는 프레임 없음): `GET /api/groups` → `[{id, name, icon, pos, convs}]`(레일 순서 — #1j 의 `peers` 는 #1w 에서 없앰) · `POST /api/groups {name}`
  → 201(이름 = 앞뒤 공백 뺀 1 ~ 40자 · 제어 문자 없음 — 400, 같은 이름 409 "같은 이름의 그룹이 있습니다") · `PATCH /api/groups/{id} {name?, icon?}`(둘 다
  검증한 뒤 쓴다, 빈 `icon` = 지움, 없으면 404) · `DELETE /api/groups/{id}` → 204(소속도, 대화는 그대로) · `PUT /api/groups/{id}/conversations {add?,
  remove?}`(이미 있음 · 없음은 무시, 없는 대화 400, 합쳐 10000개까지) · (#1j 상대 단위 `PUT /api/groups/{id}/peers` · `PUT /api/peers/{id}/groups` 는 #1w 에서 없앰 — 그룹 = 대화만, 404) · `PUT /api/order {kind: peers|groups|conversations, ids}` → 204(준 순서대로 자리
  0 ..; 겹침 · 없는 id 400 — 아무것도 안 바뀜, 빠진 id 는 제 자리 · 같은 자리면 id 순). `GET /api/messages` · `GET /api/saved` 에 `group`(`peer` 와 함께면
  400), 새 대화 본문 `group?`(없는 그룹 404 — 보내지 않음, 소속은 대화와 같은 트랜잭션), 대화 요약 JSON 에 `pos`, `GET /api/peers` 순서 = 자리.
  거절(`focus` 등)은 403 + 글 `refused`. CLI `ask --conv N --agent X` — X 가 대화 에이전트가 아니면 멘션(자동 왕복 대화는 오류, 상대 PC 목록에 없으면
  사용법 오류), `history` 발화자 = 실행한 에이전트.
  #1q: `GET /api/activity?kind=all|request|mine|answer|failed`(받은 요청 · 내 요청 · 받은 답 · 실패, 옛 `human` · `agent` · `relay` 는 400) — 항목 `kind` =
  `message`(받은 요청 · 받은 답) · `my_request`(시각 = 마지막 상태 바뀐 때, 안 본 = 받은 상태 알림 기록 `relay_events.kind = req_state`) · `failed` · `sync_failed`,
  자동 왕복 시작 · 끝과 상대 사람 일반 메시지는 목록 · 안 본 수에 없다(`core::chat::activity` · `activity_unseen` · `note_req_state`). `GET /api/messages?side=both
  &peer=|group=&before=&before_at=&limit=`(양쪽 글 · `(created_at, id)` 최신순 · 받은 마지막 글의 id · 시각 복합 커서 — 둘 다 주거나 둘 다 빼고, `kind` · `ask` · `req` · `q` 와 함께면 400 · 커서는 `both` 만) · `GET /api/messages/peers
  ?peer=|group=` → `[{peer, count, last}]`(주고받은 메시지 상대별 요약, context 글 제외). `GET /api/messages/req_counts?side=theirs|mine&peer=|group=` →
  `{wait, accepted, rejected, done, total}`(요청 개요 머리 — 범위 전체, 목록 상한과 무관 · `core::chat::req_counts`). 요청 화면 상태 거르기는 서버(`req=`). CLI `coworkAgent /?` · `-h` · `--help`(첫 인자) → 표준 출력에 `cli::help()`
  (소개 + `USAGE` + 출력 형식 · 종료 코드 · 연동 예) · 0, 앱 불필요.
  #1x 세션 생성 · 기록 삭제: `POST /api/conversations` 의 `text` 는 생략 가능(`#[serde(default)]`) — 비었고 `via` 없음 = 세션 생성(`Chat::open_session` — 글 없는
  대화, 201 `{conversation}` · `message` 없음, `relay_agent?` · `relay_cap?`(없으면 `relay_cap_default`) · `title?` · `group?` 은 대화에 붙는다), 빈 본문 + `via` · `request` 는
  400 "본문이 비었다"(CLI 는 본문 필수). 상대 PC 는 첫 이어 쓰기 글이 갈 때 대화를 받는다 — 자동 왕복 설정이 있는 빈 대화의 첫 글은 자동 왕복 시작(요청이면 400).
  `POST /api/conversations/{id}/clear {all?}`(기록 삭제 — `Chat::clear_conversation`, `all` 기본 거짓 = 내 글만) → 204 · 공유 사본 400 · 닫힌 대화에 `all` 400 ·
  실행 중 · 대기 글 409 "실행이 끝난 뒤 지울 수 있습니다" · 상대 앱이 모름 409 "상대 앱이 이 기능을 모릅니다"(아무것도 안 지움) · 페어링 아님 403 · 없는 대화 404.
  프레임 `Clear{conv, id}` · `Pong.clear` 는 [an_net](an_net.md). 웹UI 나가기 = `DELETE /api/conversations/{id}` 만(`POST .../leave` 는 CLI · API 호환).
  웹 — `view.js` `SESSION_NEW` · `convMenuItems(conv)`(`sub` — 기록 삭제 › 내 글만 · 모든 사용자, `LEAVE_CONFIRM`) · `sideSub` · `composeDefaults` · `requestNotice`,
  `app.js` `openCompose` · `chat_form`(생성 → `#c/{id}`) · `openConvMenu` 하위 메뉴 · `convEndForm`(`END_TEXT`) · `convGrab` · `applyTileSize` · `chime`.
  #1y 기간 · archive: `GET /api/messages?side=both` · `GET /api/messages/peers` 는 설정 `ui_peer_weeks` 기간 안 글만(`peer_since` = 지금 − 주 × 7일 →
  `chat::messages_page` · `peer_message_counts` 의 `since`, 질의 인자 없음). `GET /api/messages` 의 `archived=true|false`(`ask=request` 와 함께만 — 없이 400
  "archived 는 ask=request 와 함께", `side=both` 와 함께 400; 없거나 거짓 = archive 제외 · 참 = archive 만). `POST /api/messages/{id}/archive {archived}` → 200 메시지 JSON
  (`chat::set_req_archived` — 없는 메시지 404 · 요청 글 아님 · 공유 사본 400 · 완료 아닌데 `true` 409 "완료된 요청만 archive 할 수 있습니다", 이 PC 에만 — 프레임 · outbox 없음).
  메시지 JSON 에 `archived`(`Message::archived(unix_now())`), `req_counts` 응답에 `archived`(`done` · `total` 은 archive 제외). 웹 — `view.js` `reqArchiveAction` ·
  `PEER_WEEKS` · `peerWeeksOptions` · `reqHead`(`archive n`), `app.js` `renderReqArchive` · `archiveReq`.
- guard 순서(#1a): Host(`127.0.0.1:{port}`) 403 → Origin(있으면 일치) 403 → 정적 자산(`server::PUBLIC` GET — 토큰 없이, 진입 `/?t=` 는 틀리면
  401) → `Authorization: Bearer` 만(쿠키 읽지 않음 · `/api/events` 만 `?t=` 도) 401. 모든 응답(401 · 403 포함)에 `server::CSP` · `X-Frame-Options:
  DENY` · `X-Content-Type-Options: nosniff` · `Referrer-Policy: no-referrer`, `GET /` 는 `Cache-Control: no-store`. 웹UI 는 `view::entryToken` 으로
  주소의 `t` 를 sessionStorage 로 옮기고 주소창에서 지운다 · 401 이면 `#disconnected` 안내. 경로 `/` · `/app.js` · `/view.js` · `/app.css` · `/favicon.ico` · `/fonts/{파일}`(#1p — `server::FONTS`
  에 있는 이름만 토큰 없이, IBM Plex Sans KR woff2 3 · `LICENSE.txt` · `Cache-Control: public, max-age=31536000, immutable`, 그 밖 이름은 토큰 없으면 401 · 있으면 404) · `/api/status`(`version` · `data_dir` · `db` · `cli_api` = `server::CLI_API`(1) — CLI 가 보내기 전 확인) · `POST /api/open`.
- `POST /api/open` — `open_browser()` 가 `Err` 면 500, 아니면 204. 두 번째 실행은 5xx 를 재시도 없이 실패 종료.
- `notify::Notifier::new(conn, attention)` — 기본 OFF(`settings` `notify_inbound`). 켜짐 · 세부 설정(`Prefs`) · 건수가 한 잠금:
  `on_event`(켜져 있고 `Prefs` 가 허락하면 건수 +1 → `attention(n)` · broadcast 전송, 막히지 않음 — 걸러진 것은 건수에도 안 든다) ·
  `clear()` · `set_enabled`(끄면 건수 0) · `prefs()` · `update(&SettingsPatch)`(검증 → 한 트랜잭션, 틀리면 `InvalidInput` · 아무것도 안
  바꿈) · `relay_cap_default()` · `run_defaults()` · `screen()`(설정 DB 읽기 오류는 `Err` — 기본값으로 감추지 않는다) · `subscribe()` · `pending()`. `Prefs` = `notify_pair` · `notify_request` · `notify_reply`(기본 켬) ·
  `notify_relay`(`start_end` 기본 · `every` · `off`) · `notify_preview`(기본 켬 — 끄면 본문 = 이름 · PC 만, 끝 사유는 보임) · `notify_paused`(기본 끔 —
  켜 있는 동안 알림 · 트레이 표시 · 이벤트를 버린다, 건수 그대로) · `notify_sound`(#1x — 기본 켬, 웹UI 받은 요청 효과음 — 서버는 저장 · 읽기만).
  받은 요청(`MessageKind::Ask`, #1x — B안)은 kind `request` 묶음 알림 `request_notice(rows, total, preview_on)`: 제목 "받은 요청 {대기 수}", 본문 =
  `an_core::chat::waiting_requests` 최근 `REQUEST_LINES`(3) 줄 `{표시 이름} · {첫 줄 앞 40자}`(미리보기 끄면 이름만) + 넘으면 "외 {n}건"(`Notifier::waiting` — 상태 잠금을 놓고
  읽는다, 못 읽거나 대기 0 이면 보통 알림 "요청 받음 — {에이전트}"). `MessageKind::Clear`(#1x — 상대가 모든 사용자 글 삭제) = kind `message` "대화 기록 삭제 — {에이전트}" ·
  본문 "{이름} · {호스트}\n{이름} 이(가) 대화 기록을 모두 지웠습니다"(미리보기와 무관, `notify_request` 축).
  `notify::routes` — `GET/PUT /api/settings`(PUT 은 준 항목만 · 모르는 항목 · 틀린 값 400, `relay_cap_default` 1 ~ 50, `claude_model` ·
  `claude_effort`(실행 기본값 `sonnet` · `high`) · `ui_theme`(`light` 기본 — #1x · `dark`) · `ui_tile_size`(#1x — `small` 기본 · `normal`, 틀린 값 400 "ui_tile_size 는 normal · small", `an_core::run_opts::TILE_SIZE_KEY`) · `ui_home_view`(#1z-4 — `preview` 기본 · `list`, 틀린 값 400 "ui_home_view 는 preview · list", `an_core::run_opts::HOME_VIEW_KEY`) · `ui_home_zoom`(#1z-6 — 정수 100 기본 · 125 · 150 · 175 · 200, 틀린 값 400 "ui_home_zoom 는 100 · 125 · 150 · 175 · 200", `an_core::run_opts::HOME_ZOOM_KEY`) · `ui_peer_weeks`(#1y — 정수 1 ~ 12, 기본 4, 틀린 값 400 "ui_peer_weeks 는 1 ~ 12", `an_core::run_opts::PEER_WEEKS_KEY`) · `ui_side_width`(200 ~ 560) · `ui_folded`(접은 절 열쇠 목록 — `{화면}:{상대 id | me}` 300개까지, #1g) · `me_alias` · `me_icon`(빈 = 지움),
  설정 DB 읽기 오류는 500) ·
  `GET /api/events`(SSE, 상태 변경 없음 — 알림 `message` + 변경 신호 `event: change` `{res: [{kind, id?}]}`, 변경 신호는 `Notifier::on_event` 가 켜짐 · 세부 설정 앞에서 늘 보내고
  뒤처지면 빈 `res` 하나 — `notify::next_change`, D39 ②) · `POST /api/seen`(지움). 상한 없는 자동 왕복 새 대화는 `relay_cap_default`.
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
  `cli::run(args, instance_file, out, err, wait_limit)` — 테스트 진입(임시 경로 · 짧은 대기). `EXIT_START_FAILED` 8(#1n).
- `control`(#1n · D14 보강) — 부모 도구 메뉴 `start [--parent <PID>]` · `open [--parent <PID>]` · `stop`. `cli::main` 이 handshake 없이
  `control::run(args, paths, exe, out, err)` 로 넘긴다(`is_command`). 부모 = `--parent`(0 보다 큰 수) 또는 `proc::parent_pid()`, 살아 있지 않으면 2.
  `start` = 한 `START_LIMIT`(30초) 되풀이(응답 · 모드 확인 → 트레이면 `quit` → 띄운 감시 코드 확인 → 두 잠금이 풀려 있으면 `proc::spawn_detached(exe,
  [(PARENT_ENV, pid)])`), `open` = 응답 없으면 `start` 뒤 `POST /api/open`(204 → 0 · 아니면 4), `stop` = `POST /api/quit` → 두 잠금 풀림(`STOP_LIMIT`
  — 명령 전체 상한 하나 → 6, 404 = 이전 버전 → 4), 응답 없음 + 잠금 잡힘이면 `UNRESPONSIVE_WAIT`(5초) 뒤 잡힌 잠금의 PID 를 `proc::kill_tree`(감시
  먼저) · 300 ms 풀린 채 머무는지 확인. `probe(paths) -> Option<모드>`.
  출력 "시작함" · "이미 실행 중" · "열었음" · "종료함" · "실행 중이 아님", 실패 = 표준 오류 + 코드(2 · 4 · 6 · 8 — "시작 실패 — {사유}").
- `proc`(#1n) — `parent_pid()`(Windows Toolhelp · Unix `getppid`) · `alive(pid)`(권한 거부 = 살아 있음, Unix pid ≤ 0 = 아님) · `watch_parent(pid,
  on_gone)`(별도 스레드 — Windows `OpenProcess(SYNCHRONIZE)` 대기, 못 열면 · Unix `POLL` 1초) · `kill(pid)`(`TerminateProcess` · `SIGKILL`) ·
  `kill_tree(pid)`(감시 + 자식 — Windows 는 이 실행 파일과 이름이 같을 때만 · 끊은 뒤 부모 PID 로 자식을 찾음, Unix 는 그룹 우두머리면 그룹째) ·
  `spawn_detached(exe, envs)`(표준 입출력 비움 · `watchdog::ENV` 지움 · Windows `CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP` + 자기 표준 핸들
  상속 끔 · Unix `process_group(0)`).
- `browser::open(url 또는 파일 경로)` — Windows `ShellExecuteW`(반환 ≤ 32 = 실패) · macOS `open` 종료 상태(두 OS 같은 동작 — D30).
- `tray::Tray::new() -> (Tray, Attention)` — 메인 스레드 이벤트 루프. `Attention::set(n)` 은 프록시로 "새 요청" 표시(주황 점 · 툴팁).
  `Tray::run(url, note, on_seen, on_quit)`(`note` = 툴팁 둘째 줄 — 피어 포트를 남이 잡았을 때 `peer_port_note`, `tray::tip(n, note)`) — "열기" · "종료"(S4 개정 — 내 정보는 웹UI 프로필 "사용자 정보" 팝업 `#me_dialog`, #1l), 열기 성공 시 `on_seen`. 트레이를 못 만들면
  `tray::retry_after(n)`(1 · 2 · 4 · 8 · 15 · 30초 — #1h, `ControlFlow::WaitUntil`) 뒤 다시, 끝내 못 만들면 사유 파일을 열고 끝냄(종료 코드 0). `Tray::run_error(message, file)` — 붉은 아이콘 · "오류 보기"(열기 전에 사유 파일을 다시 쓴다) · "종료"(감시만 부른다 — #1n). `write_start_error(message)` —
  `start_error_file()`(임시 폴더 `coworkAgent-start-error.txt`, 디버그 빌드 + `COWORKAGENT_DATA_DIR` 면 그 폴더) · `read_start_error()`("사유:" 줄) ·
  `start_error_by(pid)`(그 PID 가 썼나 — 사유 파일 끝 줄 `PID: n`).
  `Tray::quitter()` — `/api/quit` 용 종료 핸들(`TrayEvent::Quit` 프록시). `Attention::none()` — 트레이 없는 모드. 종료 순서: 런타임 정지 →
  `instance.json` · `instance.pid` 삭제 → 잠금 해제.
- `watchdog`(#1e) — `ENV`(`COWORKAGENT_SUPERVISED`, 값 = 다시 띄운 횟수 · 없으면 감시 프로세스) · `MAX_CRASHES`(3) · `CRASH_WINDOW`(60초) ·
  `RESTART_DELAY`(1초) · `supervise(spawn: FnMut(u32) -> io::Result<i32>, delay) -> Outcome`(`Done` = 자식 0 · `StartFailed` = 자식 `START_FAILED`(3) —
  다시 띄우지 않음 · `GaveUp(사유)` = 창 안 3번 · 띄우기 실패) · `Crashes::record(at) -> bool` · `spawn_app(n)`(이 실행 파일 · 인자 없음 · `ENV = n` →
  종료 코드, 코드 3 인데 사유 파일을 그 자식(PID)이 쓰지 않았으면 -1 — MSVC `abort()` 는 다시 띄운다) · `restarts()`. #1n: `PARENT_ENV`(`COWORKAGENT_PARENT` — 트레이 없는 모드 · 부모 PID, 감시가 자식에 물려준다) · `parent()` ·
  `acquire(paths) -> Option<InstanceLock>`(`supervisor.lock` + `supervisor.pid`, 못 잡으면 `None`).
- `update`(#1y · #30 — S6 · D12 · D36 개정, 절차는 [개발자 가이드](developer-guide.md) 2.4) — `RELEASES`(`https://github.com/Softcamp-SaaS-RND/coworkAgent/releases`) · `SIGNER`(`"SOFTCAMP CO.,LTD."`) · `PRODUCT`(`"coworkAgent"`) ·
  `ASSET`(Windows `Some("coworkAgent.exe")` · 그 밖 `None` = 확인 안 함) · `MAX_ASSET`(64 MiB) ·
  `check(base, asset, current, skip, exe, trusted: Fn(&Path, tag)) -> Outcome`(`Updated(tag)` · `UpToDate` · `Skipped(사유)` — 받기 · `is_native_pe`(PE · 같은 아키텍처) · `.new` 쓰기 · `trusted(.new, tag)`(실패면 `.new` 지움) · 교체, `skip` = 되돌린 태그, 실패는 전부 `Skipped`) ·
  `authenticode(path, signer)`(Windows `WinVerifyTrust` — `WINTRUST_ACTION_GENERIC_VERIFY_V2` · 폐기 확인 `WTD_REVOKE_WHOLECHAIN`(루트 제외) · 서명자 인증서 `CERT_NAME_SIMPLE_DISPLAY_TYPE` = `signer`, 그 밖 OS 는 `Unsupported`) ·
  `identity(path, product, tag)`(버전 리소스 `\StringFileInfo\041204B0\ProductName` = `product` · `VS_FIXEDFILEINFO` 파일 버전 앞 셋 = 태그) · `verify(path, tag)` = `authenticode(SIGNER)` → `identity(PRODUCT)`(R1) ·
  `latest_tag(base)`(`{base}/latest` 넘겨주기 0 · 302 `Location` `…/tag/vX.Y.Z` 만, 아니면 `None`) · `newer(tag, current)`(정수 셋 · 엄격히 큼, 꼴이 틀리면 거짓) ·
  `swap(exe)`(받아 둔 `.new` — exe 를 `.old` 로 → `.new` 를 제자리,
  실패면 되돌림) · `rollback(exe)`(지금 → `.bad` · `.old` → 제자리) · `pending`(`.old` 있음) · `settled`(`.old` 지움 — 앱 준비 뒤, 실행 중 이미지라 못 지우면 스레드가 1초마다 최대 60번 · R2) · `clean(exe)`(`.new` · `.bad` 지움). HTTP = `ureq`(rustls · `platform-verifier`), 시간 제한 연결 5초 · `latest` 10초 · 받기 120초.
  `main.rs` `supervise`: 감시 잠금 뒤 · `watchdog::supervise` 전 — 트레이 모드(부모 없음) · 릴리스 빌드 · `ASSET` 있음이면 `clean` → `check(RELEASES, asset,
  CARGO_PKG_VERSION, update-skip.txt, current_exe, verify)` → `{outcome:?}` 를 `Paths::update_note_file()`(`update.txt`)에 덧붙임(R1 — 새 감시의 `UpToDate` 가 `Updated` 를 덮지 않게), `Updated` 면 새 exe 를 인자 없이 띄우고 0(못 띄우면 되돌림).
  자식 결과가 시작 실패 · `GaveUp` 이고 `pending` 이면 태그를 `update_skip_file` 에 · `rollback` · 잠금을 놓고 지난 exe 를 띄운다(R1). 앱 `start` 는 `instance.json` 뒤 `settled`.
  `start` 는 로거 뒤 `update.txt` 를 줄마다 `log::info!("자동 업데이트: …")` 로 옮기고 지운다 · 첫 `db::open` 뒤 `db::vacuum_if_sparse`(오류 = `warn` · 계속, `?` 아님).
  의존 `ureq` 3.4(`default-features = false`, `rustls` · `platform-verifier`) · `windows-sys` 기능 `Win32_Security_WinTrust` · `Win32_Security_Cryptography`(+ `_Catalog` · `_Sip`) · `Win32_Storage_FileSystem`. 시험 `tests/update.rs`(로컬 HTTP, `real_github_latest` · `real_signed_release` 는 `#[ignore]`).
- `autostart`(#1e, S7 · D13) — `sync(on)`(이 OS 의 실제 자리: Windows `RUN_KEY` 값 `VALUE_NAME` = `"exe"`, macOS `~/Library/LaunchAgents/{LABEL}.plist` —
  같으면 쓰지 않음 · 끄면 지움 · 없어도 성공) · `launch_agent_plist(exe)`(`RunAtLoad` · `KeepAlive` 없음 · 경로 XML 이스케이프) · Windows
  `registered(key)` · `register(key, exe) -> 바뀜` · `unregister(key)`(키 경로 인자 — 시험은 시험 키). 설정 `autostart`(`GET/PUT /api/settings`, 없으면
  켜짐) · 설정 `auto_compact`(#1m — 없으면 켜짐, `Notifier::auto_compact`): `Notifier::set_autostart_hook(f)` 가 있으면 켜기는 `f(true)` → 저장(저장 실패면 `f(이전 값)`), 끄기는 저장 → `f(false)`(실패면 저장을 이전 값으로 되돌림) — 훅 실패는 500.
  중단돼도 "저장 켜짐 · OS 꺼짐" 이 남지 않는 순서(남는 것은 "OS 켜짐 · 저장 꺼짐" 뿐 — 다음 로그인 시작 맞추기가 지움) · 훅 잠금을
  저장 끝까지 쥔다(동시 요청 순서 = 저장 순서). 앱은 릴리스 빌드에서만 훅 설치 · 시작마다 설정 값으로 `sync`. `write_atomic(path, content)` — 같은
  폴더 `.tmp` 에 쓰고 `sync_all` 뒤 이름 바꾸기(실패하면 원래 파일 그대로 — plist).
- 시작 실패는 두 OS 모두 트레이 오류 모드(+ stderr). 시작 시 `Chat::attach` · `Chat::recover`(남은 대화 대기열 · 남은 outbox) · 피어 수신 뒤
  `Node::announce`(시작 알림 — 주기 접속 확인은 없다, #1c).

- 웹UI(`web/`, 빌드 없는 ES 모듈 — D10): 시안 v4(Nocturne) 5영역(상단 바 · 레일 · 사이드바 · 대화 · 세부정보) + 설정 + 세션 생성(#1x — 옛 "새 대화"), 화면은 해시로만
  바뀐다(`#c/{id}`(`/m/{글}`) 대화 · `#p` 주고받은 메시지('나' 만) · `#activity` 내 활동 · `#starred` 즐겨찾기(#1j) · `#saved` 북마크 · `#agents`
  에이전트 · `#settings/{절}` · `#me` · `#new`, 옛 `#inbox` · `#t/` 는 첫 화면, 옛 `#s/{id}`(보낸 메시지 — #1j 에서 없앰)는 `#c/{id}`, 옛 `#p/{id}` 는 그 상대를 고르고 `#`). 색 · 글꼴 크기는 `app.css` 첫 `:root` 토큰만(다크 값은 `--dark-*` 토큰을
  `:root[data-theme="dark"]` 가 가리킨다). 상단 바 통합 검색(Ctrl K) · "+ 새로 만들기" · 알림(내 활동) · 프로필 단추(#1j — 아이콘 또는 사람 모양 + 표시 이름 한 번 `.avatar-name` 12em 말줄임 · `title` 전체,
  테두리 · 점 = `view.presenceOf(focus, offline)` → `st-online` 초록 · `st-focus` 주황 · `st-offline` 붉음 — 끊김 = `api()` 의 fetch 가 연달아 `OFFLINE_AFTER`(2)번
  닿지 못함, 닿으면 0 · `netResult` · `appOffline`) · 프로필 메뉴 `view.profileMenu(paused, focus, offline)`(머리 "상태" · `menuitemradio`
  받기 가능 / 집중 · 앱과 연결 끊김 자동 표시 | 프로필 편집 · 알림 일시 중지 · 설정 — 집중 칩 없음),
  레일 타일 A안(#1j — `tileFace(icon, group)` 사람 `PERSON_SVG` · 사람 + 그림자 `GROUP_SVG`, 그림이 있으면 그림 · 아래 이름 `.tile-name` 11px 말줄임, 레일 72px —
  #1h `tileText` 글자 · 겹친 카드 대체), 사이드바 내비 · 대화 목록(#1j — 줄 = ☆ `starButton`(`PATCH starred` — #1x 부터 줄에는 손잡이 `convGrab`, ☆ 는 대화 머리 `star_btn` 만) · 아바타 · 제목 · `view.roomPreview`
  "말한 이: 본문" · 시각 · 안 읽음, ★ 절 없음), 말풍선(#1j — `view.bubbleSide`(`mine` 오른쪽 · 그 밖 왼쪽 + 아바타 · 이름 · 시각) · 날짜 구분선
  `view.dayBreaks` · `dayLabel`(오늘 · 어제 · M.D) · 최대 폭 대화 칸의 90% · 방향 라벨 없음) · 우상단 아이콘 북마크(`bookmarkButton` — 넣기 · 빼기 `aria-pressed`,
  열쇠 `save`) · 복사(늘 보임 — #1f) · 말풍선 아래 작은 단추 `view.messageActions`(요청 상태 · 다시 알리기 — 북마크 없음, 다시 보내기는 #1k 에서 없앰 — 입력창에서 다시 쓴다) · 대화 머리 "대화 보기 / 원문 보기"
  (`role=group` + `aria-pressed`) · "북마크 n"(`saved_n` — 세부정보 북마크 절을 연다) · "세부정보" · 보낸 메시지 탭 없음 · 입력창 위 "편집"(편집 모드 — 끝난 내 사람 글 체크박스 ·
  연필, 입력창 자리 `#edit_bar` "n개 선택 · 삭제 · 완료", 반영 범위 `<dialog id="scope_dialog">` "내 창에서만" · "상대 창에도" · 취소 — 본문을 고치는
  동안은 5초 새로 고침이 대화를 다시 그리지 않는다, `view.keepPicked` · `editedTag`) · 입력창 서식 · Enter 보내기, 세부정보(진행 · 대화 정보 · 자동 왕복 · 북마크). 레일 범위(#1g, `view.sideNav`):
  '나' = 메뉴 7개 세 절(#1h · #1j — 개요 / 내 활동 · 요청 · 주고받은 메시지 / 즐겨찾기 · 북마크 · 에이전트, 절 사이 `{ sep: true }` → `li.nav-sep[role=separator]`), 목록은 `view.peerGroups`(레일 순서 · 풀린 상대 뒤 · 빈 상대 숨김)로 상대별
  `<details>` 절, '개요'(`#`, `#home_title`) = `view.peerSummary` 상대별 요약(대기 요청 = `view.waitingByPeer`), 사이드바 = `convSections(rows, paired)` 상대별 절(제목 "대화"). 즐겨찾기 `#starred` = `view.starredSections`(그 범위 ★ 대화 —
  '나' 는 상대별 절, 메뉴에 ★ 수 — `sideNav(tile, unseen, waiting, starred)`).
  상대 타일 = 메뉴 4개(#1j — 요청 / 즐겨찾기 · 북마크 · 에이전트), 첫 화면 `#` = 그 상대 하나의 개요(`renderHome` "개요 · {이름}"), 그 상대 것만 —
  `#activity` · `#p` 는 '나' 로. 절 접힘은 설정 `ui_folded`(`{화면}:{상대 id | me}`, 시작할 때만 읽는다). 사이드바 에이전트 절은 없다(에이전트 화면만). 레일 그룹(#1i): 그룹 타일 =
  메뉴 4개(`sideNav("group")` — 개요 / 요청 / 즐겨찾기 · 북마크 — #1j), 목록 · 요약은 그 그룹 대화만(`view.inGroup` — `convs`(#1w — 대화만) · `groupPeers` — 대화 있는 상대만,
  서버 거르기 `group=`), 그룹인 채 `#activity` · `#agents` 는 '나' 로(`view.groupLeaves`). 접힘 열쇠는 '나' 와 같은 화면 열쇠. 떠 있는 상자 `#float`(화면 기준 —
  "+ 그룹" · 대화 ⋯ "그룹" 체크 · "그룹에서 빼기" · "대화 추가" · 타일 메뉴 "타일 편집" · "이미지 지우기"("이미지를 지웠습니다") · "위로 옮기기 · 아래로 옮기기"(#1j `stepMove` — Alt + ↑ / ↓ 와 같음, 끝이면 꺼짐) ·
  "그룹 삭제"(상자 안 확인)) — 레일 우클릭 · 메뉴 키 ·
  사이드 머리 ⋯. 순서 = 손으로 놓은 자리(`view.sortByPos` — 대화 · 상대 · 그룹 모두): 끌어 놓기(HTML 기본 드래그 — #1x 부터 레일 타일만, 대화 줄은 손잡이 포인터 끌기 `convGrab`, `view.dropKind` — 같은 종류 · 대화는 같은
  절, 대화 → 그룹 타일 = 넣기) · Alt + ↑ / ↓(`view.stepTarget`, `#order_live` 알림 `movedText`) → 전체 순서에 끼워(`view.moveInOrder`) `PUT /api/order`. 레일 · 대화 목록은 다시 그려도 초점 항목을 되돌린다(`keepFocus` — #1j, 누른 뒤 바로 Alt + ↑ / ↓).
  설정(#1j): "페어링" 카드(초대 생성 · 초대 수락 같은 `primary` 단추, "주소 · 코드로 직접 추가" 없음 — 서버 경로는 남음) · 내 정보 · 연결 상세(`kvTable`)에 "읽기 전용"
  없음 · 그룹설정 카드 = `view.settingsGroupSections`(#1w — 그룹 절 = 대화 줄 · "그룹에서 빼기" + 끝 `USER_LIST` "사용자 목록" = 관리 줄 + 대화 트리, 대화 줄 끌기 =
  `view.convCopy` 복사) · "+ 그룹 추가" · 절 머리 이름 바꾸기 · 삭제 · 검색 `view.searchItems` · 내 에이전트 평평한 목록(그룹 칸 없음).
  설정(#1k — 디자인 캔버스 ④): `.settings-cols` 두 열(왼쪽 페어링 · 그룹설정, 오른쪽 내 정보 · 내 에이전트 — 1100px 미만 한 열, 실행 · 화면 · 시작 · 알림은
  아래 전체 폭) · 페어링 큰 단추 둘 `.pair-btns`(`window_btn` "초대 생성" · `accept_btn` "초대 수락" = `showAccept(on)` — `invite_accept` 여닫기 ·
  `aria-expanded`, 제출 단추 "수락") · 레일 "+" = `showAccept(true)` + `goSettings("conn", "invite_text")`(카드 맨 위 · 칸 초점), 트리 · 새로 만들기
  "초대 수락"(`#settings/invite_accept`)도 `render` 가 펼친다 · 관리 줄 `manageItem(key, label, head, ...body)`(머리 + "관리" `{key}_manage` 로 여닫는
  `detail_{key}`, 펼침 `state.open` — `<details>` `remembered` 대체) · 줄 타일 `rowTile(cls, icon, group)`(`tileFace`) · 절 머리 그림 단추
  `svgButton(svg, label, onClick, cls)`(`PENCIL_SVG` 이름 바꾸기 · `TRASH_SVG` 삭제) · 연결 줄 = 타일 · 이름 · 부제(이름 · PC · 접속 상태) ·
  "관리"(지문 · 주소 · 모델 · effort · 단추 — #1w 부터 연결 줄은 끌지 않는다) · 내 에이전트 줄 = 타일 · 이름 · 폴더 · "관리", 머리 `agent_new_btn` "에이전트 등록"
  (1l 에서 `showAgentNew(on)` 보이기 · 숨기기로 바뀜). 에이전트 화면(#1k): 머리 안내문 없음 · 내 에이전트 줄 오른쪽 ✎(`aria-label` "{이름} 편집") · 줄 누르기 =
  `editAgent(id)`(`state.open` 에 넣고 `goSettings("agents", "agent{id}_manage")` — 상세 펼침 · "관리" 초점), 상대 에이전트 = 세션 생성. 주고받은 메시지(#1k):
  머리 안내문 없음 · `rowEls(rows, empty, {tile: true})`(주고받은 메시지만 — 왼쪽 상대 타일 `.row-tile` · 이름 옆 에이전트 칩 `.agent-chip`) ·
  `view.inboxRows` 줄 `agent`(보낸 대화 = 상대 에이전트 · 받은 대화 = "내 {에이전트}").
  타일 색 = `view.colorOf(id)`. 아이콘 그림은 data URL 로 읽는다(CSP `img-src` 가 `blob:` 을 막는다). 타일을 누르면 사이드바에 그 타일 대화
  목록, 메시지 본문은 markdown 뷰어 · HTML 문서는 `<iframe sandbox srcdoc>`(대화 머리 "대화 보기 / 원문 보기" 로 전체 전환), 설정은 전체 폭 트리. 판단 로직은 `view.js`(DOM 없음) — `node --test` 로 시험(`view.test.mjs`),
  `app.js` 는 조립만(`textContent` · `replaceChildren`). #1c: `view.presence(p, now)`(피어 `seen_at` → "방금 통신"(75초 안) · "n분 · n시간 · n일 전
  통신" · "통신 기록 없음", 방금일 때만 점 · 집중 "집중 중 · …") · `outboxText`("보내는 중" · "보내는 중 · 다시 보냄 n회") ·
  `stepText`("2턴 · Read 3 · Grep 1 · 41초") · `progressText`("처리 중 · {단계}") · `refusedText`(`unreachable` → "보내지 못함 · 오래 닿지 않음"). `app.js` 는
  웹UI 를 열 때 페어링된 상대 모두 · 레일 타일을 고를 때 그 상대 ping(`pingPeers` — 서버가 60초로 거른다).
  아이콘 · 설정 정리(#1l): 아이콘 한 벌 = `app.css` `.ico`(16px 하나 · `currentColor` 를 SVG 모양 `mask` 로) + `.ico-{이름}`(data URI), HTML ·
  JS 는 `<span class="ico ico-{이름}">` · `ico(name)` 만 — 사이드바(`view.sideNav` 항목 `icon`) · 화면 머리 제목(동적 제목은 안쪽 `span` 이 id) · 설정 트리
  (`view.settingsTree` `icon`) · 타일 ⋯ · 대화 ⋯ · 상대 그룹 체크 · "+ 새로 만들기"(`view.newMenu` `icon`, "자동 왕복 대화" 없음 — 세션 생성 화면 안). 말풍선
  색 = `--surface` 바탕 + `::before` 색 층(내 글 `--accent` · 상대 글 `article[data-tone=colorOf(peer_id)]` → `--tone`, 불투명도 `--bubble-mine-alpha` · `--bubble-theirs-alpha` — Safari 15 때문에 `color-mix` 아님) · 옆 띠 없음(#1p). 설정 한 열(`.settings-cols`
  없앰 — 페어링 → 그룹설정 → 내 에이전트 → 실행 → 화면 → 시작 → 알림) · 카드 안내 = 제목 옆 `.help-wrap`(`button.help` + `.tip[role=tooltip]`,
  `aria-describedby`, `:hover` · `:focus` 에 뜸) · 내 정보 = `<dialog id="me_dialog">`(프로필 `profileMenu` 열쇠 `me` → `showModal`, 트리 · `#me` 경로
  없음) · 내 에이전트 검색 칸 없음 · 등록 칸 `<div id="agent_new" hidden>` = `showAgentNew(on)`("에이전트 등록" · 새로 만들기). 타일 순서 = 설정
  `ui_rail_order`(`peers_first` 기본 · `groups_first`, `an_core::run_opts::RAIL_ORDER_KEY` — 틀린 값 400) → `applyRailOrder` 가 레일 DOM 을 옮긴다
  (Tab 순서 = 화면 순서).

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
