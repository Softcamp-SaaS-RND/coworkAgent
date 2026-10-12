# stage1 — 스키마 8 · 프로토콜 `via` · `Chat` · API 필드 · 웹UI 라벨

## 목표

[plan](../plan.md) Stage 1 — CLI 글의 출처 표시(`via = 'cli'`)를 저장 · 프로토콜 · 조율 · API · 화면까지 잇는다. 대상 gate: E3 · E4 · E5.

## 산출 파일

- `crates/core/src/{db.rs,chat.rs}` — 스키마 8(`messages.via TEXT`, 이전은 기존 규칙 — 한 트랜잭션 · 있는 열 건너뛰기 · 사본),
  `Message.via` · `set_via`(`cli` 만 남김) · `VIA_CLI`
- `crates/net/src/proto.rs` — `Request::Message` · `Request::Post` 에 `#[serde(default)] via`
- `crates/agent/src/chat.rs` — `send_via` · `post_via`(기존 `send` · `post` 는 `None` 으로 위임), `send_human` 에 `via`, 받은
  `message` · `post`(사람 글)의 `via` 저장
- `crates/app/src/chat_api.rs` — `POST /api/conversations` · `POST /api/conversations/{id}/messages` 의 `via`(`cli` 만, 그 밖 400),
  `message_json` 에 `via`
- `crates/app/web/{view.js,app.js}` — `authorLabel` CLI 라벨 · `kindText`(글 종류 칸) · 타임라인 요청 단계 제목
- 테스트: `paths`(`schema8_via_column` + 버전 단정 7 → 8) · `cli_origin`(신규 3건) · `chat_api`(`via_api`) · `view.test.mjs`(CLI 라벨 1건) ·
  기존 `relay` · `roundtrip` · `two_sided` 의 요청 리터럴에 `via: None`

## 설계 결정

- `Chat::send` · `post` 의 시그니처는 그대로 두고 `send_via` · `post_via` 를 더했다 — 기존 호출(웹UI · 테스트)이 바뀌지 않는다(plan 개정 1).
- `via` 는 삽입 뒤 `set_via` 로 **같은 트랜잭션** 안에서 쓴다(받은 글은 `observe` 까지) — `add_message` · `add_human` 시그니처와
  그 테스트 호출을 건드리지 않는다(plan 개정 1 ③).
- 자동 왕복 시작에 `via` 가 오면 400 — CLI 범위 밖이고, 조용히 버리면 출처가 사라진다(plan 개정 1 ④).
- 받은 쪽은 `cli` 만 저장, 그 밖 값은 무시(NULL) — 원격 입력을 신뢰하지 않는다.
- 답 대기 판정은 `reply_to`(질문의 **로컬 id**)로 한다 — 실측: `message_json` 의 `reply_to` 는 uuid 가 아니라 로컬 id
  (`via_api` 단정). plan 설계의 "`message_json` 에 `uuid` 추가" 는 필요 없어 빼고 `via` 만 더했다(plan 개정 1).
- 라벨: 내 PC `내 AI 에이전트(CLI)`, 상대 PC `{상대}/AI 에이전트(CLI)`, 글 종류 칸 `AI 에이전트(CLI) → 에이전트` · `AI 에이전트(CLI)`,
  타임라인 요청 단계 `요청 — AI 에이전트(CLI)`.

## 테스트 결과

- `cargo test --workspace` — `passed=139 failed=0`(test result 줄 합산).
- `cargo clippy --workspace --all-targets -- -D warnings` — `Finished`(처음엔 `via_of` 의 `result_large_err` 1건 → 오류 타입을 `&'static str` 로).
- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` — `# pass 13` · `# fail 0`.
- `cargo test -p an-core --test paths` — `10 passed`(E4) · `cargo test -p an-agent --test cli_origin` — `3 passed`(E5).
- RED 확인: 구현 전 `paths` · `cli_origin` 컴파일 실패(`set_via` · `via` · `send_via` · `post_via` 없음), `via_api` 단정 실패, node `# fail 1`.
- mutation 3종 모두 FAIL: 받은 `message` 의 `set_via` 제거 → `cli_origin` 1 failed · 받은 `post` 의 `set_via` 제거 → 1 failed ·
  API `via_of` 가 `cli` 를 버림 → `via_api` failed.
- `cargo fmt --check` 통과.

## 리뷰

분리된 리뷰 에이전트(`385ea3e..6b47a38`) — high · medium 0, low 3. 전부 코드로 확인 후 반영:

| 지적 | 판단 | 조치 |
|---|---|---|
| 글 저장과 `set_via` 가 한 트랜잭션이 아니다(4곳) — 사이 실패면 출처 없는 글 · 받은 쪽 실행 요청은 남고 보낸 쪽은 `error` | 사실 | 4곳 모두 `unchecked_transaction` 으로 묶음 |
| `relay_agent` + `via` 를 주면 `via` 를 조용히 버리고 201 | 사실 | 400 거절 + `via_api` 단정 |
| 테스트 범위 — 받은 대화의 CLI 실행 요청 미러 · 받은 쪽 답의 `via` 미검사 | 사실 | `cli_origin` 두 테스트에 단정 추가(건수 3 유지) |

반영 뒤: `passed=139 failed=0` · clippy(강제 재검사) `Finished` · node `# fail 0` · `cargo fmt --check` 통과.

## 다음 단계

Stage 2 — CLI(`cli.rs` · `local_request` · `main` 분기) · 릴리스 파이프. 대상 gate: E6 · E7.
