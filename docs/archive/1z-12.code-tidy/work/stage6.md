# stage 6 — 중복 · 늘 같은 쪽 분기(intent 결정 9)

## 목표

plan "설계 · stage 6" — [work/stage4](stage4.md) stage4-7 중 동작이 같아 보인 다섯을 하나씩 판정해 동작 불변인 것만 지운다(사용자 "권장 방안으로 진행." — 종결 report 의 report-5 (a)).
지우는 조건 = 순차 실행 · 동시 실행 · 오류 응답 순서 · 시험 계약(A1)이 모두 같고, 합친 꼴이 지금보다 읽기 어렵지 않을 것.

## 산출 파일

- `crates/agent/src/chat/delivery.rs` — `kick`.
- `crates/app/src/chat_api/conversation.rs` — `start_conversation`.
- `crates/app/tests/chat_api.rs` — 새 특성 시험 `session_relay_cap_default_from_settings`(기존 시험은 그대로).

## 설계 결정

| 대상 | 판정 | 근거 |
| --- | --- | --- |
| `delivery.rs` `kick` 의 늘 참인 `if let` | 지움 | 같은 잠금 안에서 앞 블록이 줄이 없거나 끝났으면 넣으므로 뒤 `lanes.get` 은 늘 `Some` 이었다. `match` 하나가 살아 있는 줄의 `wake` 를 쓰거나 새 줄을 띄워 그 `wake` 를 돌려주고, 끝에 `notify_one` 한 번. 띄우기 → 넣기 → 깨우기 순서 · 잠금 범위 그대로(새 줄이 먼저 돌아도 `Notify` 허가가 남아 첫 `notified()` 가 바로 지나간다 — 전과 같음) |
| `chat_api/conversation.rs` 상한 기본값 계산 두 번 | 한 곳으로 | 같은 식 `req.relay_cap.map_or_else(…default_relay_cap…, Ok)` 이 세션 생성 · 자동 왕복 시작 두 갈래에 있었다 → 지연 클로저 `relay_cap` 하나를 두 자리에서 부른다. 부르는 자리 · 차례가 그대로라 검증 400(본문 빔 + `via` · `request`, 자동 왕복 + `via`)이 DB 읽기보다 먼저인 것, 자동 왕복이 아닌 요청은 DB 를 읽지 않는 것 그대로. 새 함수 없음 |
| `tray.rs` 종료 두 갈래 같은 몸 | 그대로 | `TrayEvent::Quit` 과 메뉴 `quit_id` 를 한 갈래로 합치려면 or 패턴이 한쪽에서만 `e` 를 묶을 수 없어 `matches!` 둘을 겹친 조건식(또는 새 도우미)이 필요하다 — 몸 6줄보다 읽기 어렵다(plan stage 6 규칙) |
| `set_blocked` 두 번 | 그대로 — 동작 불변 아님 | 받은 대화에 사람이 쓰면 `send.rs` 가 바로 거절을 켜고 `block_relay` 를 띄우며, `block_relay` 가 다시 켠 뒤 연 쪽에 알린다. `block_relay` 의 쓰기는 `relay()` 받은 쪽 끄기에도 필요하다. 그 쓰기를 `relay()` 로 옮겨 사람 글 길을 한 번으로 줄이면, 두 쓰기 사이에 연 쪽 재개(`inbound_relay` → `set_blocked(false)`)가 끼는 경우 지금은 다시 거절로 끝나고 바뀐 뒤에는 풀린 채 남는다 — 동시 실행에서 끝 상태가 다르다 |
| `logic/conv.js` `convListParts` 다시 거르기 | 그대로 — 시험 계약 | 호출자(`sidebar.js`)가 `visibleConvRows` 로 거른 줄로 절 · 빈 안내를 만들고 `convListParts` 도 거른다. 바깥은 절 · 빈 안내에 필요하고(`visibleConvRows` 주석), 안쪽은 `view.test.mjs` 가 함수 계약으로 고정한다(거르지 않은 줄을 주는 "에이전트 대화뿐이면 빈 목록"). 빼면 기존 시험이 바뀐다 — A1 의 바뀌는 시험 셋 밖 |

- 범위 밖(plan): 서명 · 문구가 바뀌는 것(안 쓰는 `peer` · `acked` 반환값 · `sharing.rs` 같은 검사 두 번 · 종류 목록 네 곳)과 stage4-7 의 나머지(`notify.rs` `Changed` 갈래 · `VIEW_EL.sent` · `conv.js` 매개변수 가림) — 그대로, 다음 Task 후보.
- 주석: `kick` 의 주석("줄 작업이 끝났으면 다시 띄운다")은 바뀐 꼴에도 맞아 그대로. 새 주석 · 이력 표식 없음.

## 테스트 결과

- 특성 시험 먼저: `session_relay_cap_default_from_settings`(글 없는 세션 생성 + `relay_agent` — 설정이 없으면 10, 설정 4 면 4, 둘 다 201). 고치기 전 코드에서
  `cargo test -p coworkagent-app --test chat_api session_relay_cap_default_from_settings` → `test result: ok. 1 passed`. 기존 `relay_cap_default_from_settings` 는 글 있는 갈래만 잰다.
- 고친 뒤: `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`. `cargo test --workspace` → `test result: ok` 84줄 · `FAILED` · `panicked` · `error` 0 · 통과 522(새 시험 +1).
  `node --test` → `# pass 172` · `# fail 0`. `check_code_tidy` → `PASS — 제품 원본 116개 · 최대 960줄 · 이력 표식 0`. `check_crates` `violations=0` · `check_readme` ok · `check_ddd_docs` PASS.
- `kick` 은 보낼 함에 넣는 모든 길(`delivery.rs` · `context.rs` · `lifecycle.rs` · `mod.rs`)이 부른다 — an-agent `durable` · `roundtrip` · `two_sided` 등 상대에게 보내는 시험이 지난다(바뀐 시험 없음).
- 실제 앱(평가 단계 — 디버그 빌드 = `4cff605` 코드 · scratchpad `ui1z12\data`, E8 57 단계 뒤): 글 없는 세션 생성 + `relay_agent`(`coder`) → 201 · `relay_cap` 10 = 설정 `relay_cap_default` 10.
  그 대화에 글 → 201, 보낼 함 행(`relay_open`) `tries` 3 · `last_error` 연결 거부(os error 10061) · 다음 시도 예약 — `kick` 이 줄을 띄우고 깨워 전달을 시도했다(시험 데이터의 상대는 원격 없음).

## A2 예외 범위 대조(라운드 2/2 지적 R2-1)

[work/stage2](stage2.md) 상태 변경 표의 쓰기를 속성 소유와 대조했다 — 소유는 코드에 정해져 있지 않아 `core/state.js` 의 그 속성 설명이 가리키는 화면으로 셌다(구현자 해석).
다른 도메인 상태를 쓰는 곳만:

| 쓰는 모듈 | 속성(소유) | 흐름 | 결정 8 |
| --- | --- | --- | --- |
| `settings/prefs.js` | `folded`(사이드바) · `homeView`(개요) · `peerPages`(주고받은 메시지) · `runs`(실행 상세) | 설정값 적용 | 안 |
| `events.js` | `previewAll` · `previewStale`(개요) · `reqBox`(요청) | 변경 신호 | 안 |
| `drag.js` | `convs`(사이드바) · `groups` · `peers`(레일) | 순서 저장 | 안 |
| `shell.js` · `rail.js` · `sidebar.js` · `conv-menu.js` | `current`(열린 대화) | 타일 · 대화 고르기 · 닫기 | 안 |
| `shell.js` · `compose.js` · `settings/index.js` · `settings/agents.js` | `focusAfter`(다음 화면 초점) | 다음 화면 초점 | 안 |
| `shell.js` | `previewAll`(개요) | 개요에 들어올 때 미리보기 전부 다시 | 밖 |
| `shell.js` | `focusMsg`(대화) | 주소 `#c/{대화}/m/{글}` 의 갈 글 | 밖 |
| `rail.js` | `unseen`(사이드바 내비) | 타일 범위를 바꾸면 안 본 수 0 | 밖 |
| `sidebar.js` | `waitingBy`(개요 요약) · `unseenAll`(상단 종 점) | 개수를 한 번 읽어 여러 화면에 | 밖 |
| `conv.js` | `pickMsg`(실행 상세) | 글 고르기 → 실행 상세 '진행' | 밖 |
| `composer.js` | `pickMsg`(실행 상세) | 보낸 뒤 실행 상세 = 방금 보낸 요청 | 밖 |

- 결정 8 밖 일곱(`shell.js` 둘 · `rail.js` · `sidebar.js` 둘 · `conv.js` · `composer.js`)은 결정 8 의 흐름과 같은 종류(화면 전환 · 고르기 · 개수)지만 사용자가 승인한 목록에 없다 —
  구현자가 예외를 넓히지 않는다. report 에 A2b "결정 8 안 충족 · 밖 일곱 미충족" 으로 남기고 사용자 결정으로 올린다(예외에 넣을지 · 도메인별로 쪼갤지).
- 소유를 "그 상태를 채우는 모듈" 로 세면 `sidebar.js` 둘(개수를 읽는 곳) · `conv.js`(글을 고르는 곳)는 자기 상태가 되어 밖은 넷이다 — 어느 기준이든 `shell.js` 둘 · `rail.js` · `composer.js` 는 밖.

## 다음 단계

평가(원장 다시 · E8 다시 — [eval](../eval.md)) → 종결.
