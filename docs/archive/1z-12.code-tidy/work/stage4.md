# stage 4 — 주석(결정 3)

## 목표

plan "설계 · stage 4" — 제품 원본의 주석에서 결정 이력(Task 번호 · 리뷰 라운드 · 사용자 지시 · 날짜 · 시안 이름 · plan · intent 항목 번호)을 빼고,
남은 주석은 코드만 봐서는 모를 왜 · 보안 · 함정 · 프로토콜 제약만 짧게. 코드는 한 글자도 바꾸지 않는다.

## 준비(커밋 `2a088ef`)

- 주석 문장에 기대던 `ux_literals` 있음 행 7개 → 같은 동작을 드러내는 코드 문자열(`uxcomment.py` — 있음 행 442개 전부 파싱, 주석에만 있는 문구 7 → 0):

| 행(출처) | 전 | 후 |
| --- | --- | --- |
| #18 문구 | `web` · `글 종류`(1p 에서 `메시지 종류` 로 바뀐 뒤 주석에만 남아 빈 검사였다 — `git log -S` 로 확인) | `index.html` · `role="radiogroup" aria-label="메시지 종류"` |
| #1z-9 도구 없는 등록 없음 | `if (!newEngine) return; // 1단계에서 …` | `if (!newEngine) return;` |
| #1z-6 배율은 시작할 때만 | `    applyHomeZoom(s.ui_home_zoom); // 미리보기 크기…` | `applyHomeZoom(s.ui_home_zoom);`(한 곳뿐) |
| #1z-4 첫 연결 안내 | `$("home_intro").hidden = … // 두 보기 공통(…)` | 코드 부분만 |
| #1z-4 다시 붙으면 미리보기 전부 새로 | 주석 한 줄 | `es.onopen` 안 `state.previewAll = true; refreshHome(); }; es.onmessage` |
| #1r 등록 뒤 초점 | `…?.focus(); // 숨긴 폼에 …` | 코드 부분만 |
| #1z-10 /info 실패 뒤 다시 읽기 | `infoView.key = ""; // 실패한 읽기는 …` | catch 의 `if (infoView.gen !== gen) return;` + `infoView.key = "";` |

- `tools/check_code_tidy.py`: 표식 · 1,000줄 상한 검사기. 주석은 문자열 · 문자 · 정규식 · 템플릿을 건너뛰는 렉서로 가른다. 처음 FAIL — 이력 표식 1,186.
  - 렉서 대조: scratchpad `cmtonly.py` 렉서와 116 파일 주석 목록이 같다. 처음 쓴 단순 렉서(`markers.py`)는 `logic/markdown.js`(정규식 안 백틱) · `core/instance.rs`(`'"'` 문자)에서 주석을 놓쳤다 — 그래서 버렸다.
  - 작업 중 오탐 하나를 고쳤다: `PKCS#8` 의 `#8` 을 Task 번호로 셌다 → `#` 앞이 영숫자면 세지 않음(에이전트가 피하려고 바꾼 `PKCS8` 3곳은 되돌림).

## 방법

겹치지 않는 파일 묶음 7개를 서브 에이전트가 동시에 고쳤다(주석만 · 한국어 그대로 · cargo · node · git 금지 · 시험 · `index.html` · 다른 세션 파일 금지 · 이상한 로직은 목록으로만 · `static` 금지 규칙 명시).

| 묶음 | 파일 | 이력 표식 전 → 후 |
| --- | --- | --- |
| an-core | `crates/core/src/**`(23) | 176 → 0 |
| an-net · an-agent 공통 | `crates/net/src/**`(6) · `crates/agent/src/{gate,lib,runner}.rs` | 102 → 0 |
| an-agent chat | `crates/agent/src/chat/**`(13) | 193 → 0 |
| app Rust | `crates/app/src/**`(22) | 153 → 0 |
| 웹 logic · CSS | `crates/app/web/js/logic/**`(18) · `app.css` | 281 → 0 |
| 웹 UI 1 | `js/app.js` · `js/core/*` · `shell` · `events` · `rail` · `tiles` · `sidebar` · `home` · `conv` · `conv-menu` | 149 → 0 |
| 웹 UI 2 | `composer` · `detail` · `activity` · `requests` · `agents` · `peers` · `compose` · `drag` · `search` · `js/settings/*` | 132 → 0 |

## 주석 전용 확인

- `cmtonly.py 2a088ef` — 주석을 뺀 코드 토큰(문자열 · 문자 · 정규식 · 템플릿은 통째로, 공백 무시)이 같다: `files 116 · differ 0`.
  - 검사기 확인(자체 시험): 주석만 바꾸면 통과, 문자열 · 정규식 플래그 · Rust 문자 `'"'` · raw 문자열 · CSS 값 하나를 바꾸면 실패. 116 파일 모두 조각을 이어 붙이면 원문과 같다.
- Edit 도구가 바꿀 글 끝 공백을 잘라 `—다음말` 처럼 붙는 문제가 묶음마다 생겼다(에이전트들이 고침 — core 51 · app 48 · chat 35 …). `cmtonly` · 표식 검사는 이걸 못 보므로
  scratchpad `spacing.py`(주석 안 `—` · `·` 붙음 · 겹친 공백 수를 `2a088ef` 와 비교)를 셋째 검사로 돌렸다: 늘어난 파일 0.
- 바뀐 파일은 제품 원본 · 내 문서 · `tools/` 뿐(시험 · `index.html` · 다른 세션 파일 없음).

## 주석이 코드와 달라 고친 곳

서브 에이전트가 주석 쪽에서 고친 것(묶음마다 코드로 근거 확인 — 예): `runner::run_claude` → `run_agent`, `SHARE` "포기하지 않는다" 뺌(한도에서 관람 끝),
옮겨진 문서 줄 제자리(`el` · `pickRail` · `unshare` · `SHELL_ON` — 원본 `app.js` 에서부터 어긋나 있던 것), 지운 기능을 설명하던 문장(타일 메뉴 그룹 체크 · 편집 모드 ·
미리보기 크기별 줄 수), 옛 화면 이름(그룹설정 → 연결, 에이전트 대화 → 에이전트 응답, 북마크 항목 → 북마크).

보고를 받아 구현자가 코드와 대조해 고친 것(같은 개념의 낡은 설명은 저장소 전체를 grep 해 함께):

| 곳 | 고친 것 | 근거 |
| --- | --- | --- |
| `core/outbox.rs` | `failing` 설명이 `sending` 위에 붙어 있던 것 제자리 | 두 질의 |
| `core/outbox.rs` `SHARE` · `CLEAR` | `SHARE` "포기하지 않는다" → 한도에서 관람 끝, `CLEAR` "`conv_id` 없이" → 대화 줄에 넣고 나가기 · 삭제 때 뗌 | `delivery.rs` `backoff` · `lifecycle.rs` · `conversation.rs` |
| `core/run_opts.rs` | 타일 크기 기본값 `normal` → `small` | `TILE_SIZES[0]` |
| `core/chat/message.rs` | 지운 글 번호를 SQLite 가 다시 준다 → 글 번호는 다시 쓰지 않고(스키마 22) `relay_events` 번호만 | `db.rs` 스키마 22 · `lower_marks` |
| `core/chat/message.rs` `pending` | 자동 압축 구분선 먼저 · 대상 `exec` | 질의 `ORDER BY` |
| `core/chat/mod.rs` `ReqState` · `app/chat_api/message.rs` · `agent/chat/request.rs` | "대기 → 수용 · 반려, 수용 → 완료" → 네 상태 어디로든 | `can_become` |
| `core/chat/mod.rs` `LastMessage` | "마지막 상태 · 시각도 같은 글" → 상태 · 시각은 컨텍스트 구분선까지 센 글 | `summaries` 질의 |
| `core/groups.rs` | "대화는 지워지지 않아" 뺌 | `delete_conversation` |
| `agent/gate.rs` · `lib.rs` · `chat/mod.rs` · `core/agents.rs` · `net/proto.rs` `StepDetail` · `app/main.rs` | `claude -p` 만 → 에이전트 실행(`claude -p` · `codex exec`) | Codex 도 같은 문 · 단계 수(`count_codex_step`) |
| `agent/runner.rs` `SessionMissing` | Claude 만 → Claude · Codex | `run_codex` 도 냄 |
| `net/proto.rs` | `Post` · `ids_ok` 설명 제자리, 오류 코드에 `retry` 더함 | `chat/inbound.rs` · `lifecycle.rs` |
| `net/node.rs` | "`agents` · `message` · `reply` 처리기" → 에이전트 목록 · 대화 요청 | `on_request` 가 모든 대화 요청을 받음 |
| `app/server.rs` | "상태 페이지에 보이는 값" 을 `Status` 위로 | — |
| `app/chat_api/activity.rs` · `tx.rs` | 응답 필드 목록(`archived` 더함 · `again` 뺌 · `key` · `stalled` 더함) | `json!` |
| `app/notify.rs` `Notice.kind` | 알림 종류 6개 | `notice_with` |
| `app/chat_api/conversation.rs` | 자동 왕복 상한 기본 10 → 설정 `relay_cap_default`(없으면 10) | `default_relay_cap` |
| `js/settings/agents.js` · `js/detail.js` · `js/logic/message.js` | 초안 꼴에 `owner` · 복사 결과는 `#status_msg` · `refusedText` 갈래 | 코드 |

## 이상한 로직 — 보고만(코드 그대로)

| 인덱스 | 곳 | 내용 | 판정 |
| --- | --- | --- | --- |
| stage4-1 | `js/rail.js` 그룹 이름 바꾸기 | 성공 문구가 숨은 연결 화면 `peer_msg` 로, 초점도 숨은 `peer_q` 로 간다(사용자에게 안 보임) | 에이전트가 `index.html` 위치로 확인 — 다음 Task 후보 |
| stage4-2 | `js/requests.js` `boardGrab` · `js/drag.js` `convGrab` | 끄는 동안 다시 그리기를 막지 않는다(`tileGrab` 은 `railDragging`) — 목록이 바뀌면 사본 · 강조가 남을 수 있음 | 확인 필요(실행 미확인) |
| stage4-3 | `core/chat/conversation.rs` `summaries` | 마지막 상태 · 시각(구분선 포함)과 미리보기 글(구분선 뺌)이 다른 글일 수 있다 — 의도인지 | 확인 필요(주석은 지금 동작대로 고침) |
| stage4-4 | `core/share.rs` `forget_peer` | 지운 상대 앞 `clear` · `context_done` 행이 남을 수 있다 | 확인 필요 |
| stage4-5 | `js/compose.js` `loadChatAgents` · `js/peers.js` | 실패 갈래가 늦은 응답을 거르지 않음 · 빈 목록에서 서명을 안 바꿈 | 확인 필요 |
| stage4-6 | `js/rail.js` `/api/peers` 실패 문구 | "그룹설정 오류" 가 연결 화면에 뜬다 | 문구 — 다음 Task 후보 |
| stage4-7 | 중복 · 늘 같은 쪽 | 대화 요청 종류 목록 네 곳(`node.rs` 둘 · `proto.rs` `conv` · `ids_ok`), `tray.rs` 종료 두 갈래 같은 몸, `chat_api/conversation.rs` 상한 기본값 계산 두 번, `delivery.rs` `kick` 의 늘 참인 `if let`, `send_human` · `block_relay` 의 안 쓰는 `peer`, `acked` 반환값 안 씀, `sharing.rs` 같은 검사 두 번(문구만 다름), `set_blocked` 두 번, `notify.rs` 닿지 않는 `Changed` 갈래(완전성 때문에 필요), `logic/conv.js` `convListParts` 다시 거르기, `VIEW_EL.sent`(`#s/` 는 바로 넘김), `conv.js` 매개변수 `state` 가림 | 동작이 같아도 서명 · 문구 · 시점이 바뀌는 것이 섞여 이번엔 그대로 |
| stage4-8 | CSS | `.msg-actions` · `.msg-icons` 기본 규칙이 뒤 규칙에 늘 덮임, `.feedback-note` 두 번, 상대 글 `--tone` 층(불투명도 0 — 시험이 고정) | 다음 정리 후보 |
| stage4-9 | 기타 | `tls.rs` `client_config` 는 시험만, `chat_api` 글 JSON `editable` 이 글마다 잠금 · 조회를 더 함(다시 그리기 서명만 씀) | 보고 |

stage 3 보완에서 고친 것(동작 같음 — 다음 커밋): 안 쓰는 import `lead`(`js/agents.js` — 매개변수가 가려 stage 3 분석이 놓침) · `renderTilePreview` 의 안 쓰는 `name` ·
`sidebar.js` 의 같은 빈 줄 요소 둘 · 안 쓰는 CSS 토큰 `--fs-3xs` · `--fs-2xs`. `delivery.rs` 의 `"unsupported"` 는 프로토콜 일반 코드라 `CTX_UNSUPPORTED` 와 뜻이 달라 그대로.

## 남긴 D · S 번호

보안 모델 · 프로토콜 근거를 찾아갈 곳만: D8(초대 자동 승인) · D9(웹UI 보안) · D12(업데이트 서명) · D21 · D31(실행 가드 · 권한) · D22 · D24(자동 왕복 · 실행 한도) ·
D30(OS UI 없음), `ponytail:` 주석의 D39 ②(후속 후보 단서). 나머지는 같은 줄에 규칙이 말로 있어 뺐다.

## 편차

- 서브 에이전트 몇은 허용한 두 스크립트 밖에 읽기 전용 명령(`git show` · `grep` · 줄 세기 Python)을 썼다고 보고했다 — 쓰기 없음.
- 서브 에이전트는 `graphify update` 를 돌리지 않았다(맡은 파일 밖 쓰기 금지와 충돌) — 종결 때 구현자가 한 번.

## 테스트 결과

- `python tools/check_code_tidy.py` → `PASS — 제품 원본 116개 · 최대 960줄 · 이력 표식 0`(PROJECT.md 검증 명령에 "코드 정리" 로 등록 — 구현자 해석).
- `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`. `cargo test --workspace` → `test result: ok` 84줄 · `FAILED` · `panicked` 0 · 통과 521(doctest 늘지 않음).
- `node --test` → `# pass 172` · `# fail 0`. `check_ddd_docs` · `check_crates`(`violations=0`) · `check_readme` PASS.

## 다음 단계

stage 3 보완(위 네 곳) → stage 5 문서 마무리 → 평가 · 리뷰.
