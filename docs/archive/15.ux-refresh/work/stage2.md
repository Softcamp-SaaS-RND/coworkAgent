# stage2 — 스키마 9 · 실행 인자 · 초대 자동 등록 · 로컬 API · 사람에게 보내기 제거

## 목표

[plan](../plan.md) Stage 2 — 화면(Stage 3)이 기댈 데이터 · 프로토콜 · API 를 먼저 만든다. 대상 gate: E6 · E8 · E9 · E10 · E12
(E12 는 plan 개정 2 에서 생긴 CLI `say` 삭제 — 서버 쪽 `to: "human"` 400 과 같은 변경이라 여기서 함께).

## 산출 파일

- `crates/core/src/db.rs` — `SCHEMA_VERSION` 9: `peers.grp` · `peers.model` · `peers.effort` · `agents.grp`(TEXT, NULL = 없음 · 전역 기본값).
  한 트랜잭션, 이미 있는 열은 건너뜀, 올리기 전 `{db}.v8.bak`(기존 규칙).
- `crates/core/src/run_opts.rs`(신규) — 검증 단일 지점: `valid_model`(1 ~ 64자 `[A-Za-z0-9._:[]-]`, `-` 시작 금지) · `valid_effort`
  (`low` · `medium` · `high` · `xhigh` · `max`) · `clean_group`(trim, 32자) · `defaults`(설정 `claude_model` · `claude_effort`, 틀리면
  `sonnet` · `high`) · `resolve`(상대 덮어쓰기 → 전역) · `theme`(`ui_theme`, 틀리면 `light`). 단위 테스트 1건.
- `crates/core/src/peers.rs` — `Peer.group` · `model` · `effort`, `set_meta`. `agents.rs` — `Agent.group`, `update`(이름 제외) · `by_id`.
- `crates/core/src/chat.rs` — `messages_across(mine, peer, q, limit)` — 대화를 가로지른 글, 최신순, `LIKE … ESCAPE '\'`(`\` · `%` · `_` 이스케이프).
- `crates/net` — `Response::Hello.approved`(`#[serde(default)]`) · `PeerEvent::AutoPaired` · `Node::set_peer_meta` · 초대 텍스트 둘째 줄
  `"초대 수락"`(해석 1).
- `crates/agent` — `RunInput.model` · `effort`, `claude_args(agent, session, model, effort)` 끝에 `--model <m> --effort <e>`,
  `Chat::execute` 가 그 대화 상대의 피어 기록으로 `run_opts::resolve`.
- `crates/app` — `GET /favicon.ico`(`assets/coworkAgent.ico`, `image/x-icon`) + `index.html` `<link rel="icon">` ·
  `PATCH /api/peers/{id}` · `PATCH /api/agents/{id}` · `GET /api/messages` · `/api/settings` 의 `claude_model` · `claude_effort` · `ui_theme` ·
  `to: "human"` 400 · CLI `say` 삭제 · 알림 `auto_paired`("새 연결 등록", `notify_pair` 를 따름).
- 테스트(신규): `core/tests/paths.rs` `schema9_group_model_columns` · `net/tests/auto_pair.rs` 2 · `agent/tests/run_options.rs` 2 ·
  `app/tests/ux_api.rs` 8 · `app/tests/cli.rs` `say_is_unknown_command`.

## 설계 결정

- **자동 승인 전달 = Hello 응답의 `approved`** — plan 설계는 "응답 뒤 그 상대에 `Approve{round}` 를 작업으로 보낸다" 였다. 그러면
  수락 측이 Hello 응답을 받아 피어를 저장하기 전에 Approve 가 도착할 수 있다(`unknown` 으로 버려짐 — 경쟁). 같은 hello 응답에 실어
  수락 측이 `upsert` 직후 같은 세대의 `set_their_ok` 를 하게 했다. **plan 편차 1**(설계 문단만, gate 무관) — 이전 버전 수락 측은
  `approved` 를 모르므로 생성 측이 승인한 것을 모른다(수락 측에 "상대 승인 대기"로 남음 — 생성 측이 연결 목록의 승인(다시 알리기)으로
  `Approve` 를 다시 보내면 풀린다). 이 경우는 실화면 수동 확인 대상(19 TC-K).
- 상대가 `approved: true` 를 거짓으로 실어도 수락 측에서 바뀌는 것은 그 세대의 `their_ok` 뿐 — 원래 `Approve{round}` 로 할 수 있던 것과
  같아 새 위험이 아니다. 수락 측 자신의 승인은 그대로 필요(초대 붙여넣기는 원래 붙인 사람의 승인으로 본다).
- 자동 승인은 **처음 보는 키** 가 창 코드로 들어올 때만(해석 2) — 아는 키는 창이 열려 있고 코드를 담아 와도 사람 승인(테스트로 고정).
- 미승인 상한(`MAX_UNAPPROVED`)은 그대로 — 초대로 들어온 키는 자동 승인이라 미승인 기록은 아는 키의 재 hello 로만 생긴다.
  `limits.rs` `unapproved_peer_cap` 은 그 상태를 DB 에 직접 만들어 같은 규칙(33번째 처음 보는 키 busy · 아는 키 통과 · 승인하면 자리)을 본다.
- 상대별 model · effort 는 받은 요청 실행(`execute`)에만 쓰인다 — 자동 왕복의 내 에이전트 실행도 같은 경로(해석 4).
- PATCH 의미: 키 없음 = 그대로, 빈 문자열 = 지움(피어 model · effort → 전역 기본값, 그룹 → 없음). 틀린 값 400, 저장 안 됨.
  에이전트 `name` 을 주면 400(해석 3), `tools` 빈 문자열은 등록과 같이 읽기 전용 기본값.
- `/api/settings` 의 `claude_model` · `claude_effort` 는 빈 값 거부(전역 기본값은 지울 대상이 아니다). 응답은 늘 유효한 값(틀린 저장값이면 기본값).
- `GET /api/messages`: `side` 필수(`theirs` | `mine`), `limit` 1 ~ 500(기본 200, 범위 밖 400 — plan "최대 500" 을 자르기 대신 거절로 읽음),
  `q` 는 trim 후 비면 없음. 항목 = 메시지 JSON + `conversation{id, peer_id, agent, dir}`.
- `to: "human"` 400 은 로컬 API 만 — `Chat::post(…, to_agent = false)` 는 남긴다(이전 버전 상대가 보낸 사람 글 받기 · 기존 이력 표시).
- `--effort` 지원 실측(해석 5): 이 PC 에 남은 이전 실행 파일로 `claude --help` — 2.1.293 · 2.1.294 · 2.1.295 · 2.1.296 모두 `--effort <level>`
  (low · medium · high · xhigh · max) 있음. `MIN_CLAUDE` 2.1.267 ~ 2.1.292 구간은 **확인 필요**(설치본 없음) — 올리지 않았다(없다는 근거도 없음).
- 웹UI 의 "사람에게" 체크(`reply_to_human`)는 Stage 3 에서 지운다 — 그 사이 체크하면 400.

## 테스트 결과

- `cargo test --workspace` — `passed=170 failed=0`(test result 줄 합산).
- E6 `auto_pair` 2 · E8 `ux_api` 8 · E9 `run_options` 2 · E10 `paths` 11 · E12 `cli` 15 — `check_gates` 측정 ✅(E3 · E5 · E7 · E4 · E11 은 Stage 3 · 수동).
- 기존 테스트 개정(D8 · D23 개정 반영): `net` `pairing.rs`(`add_leaves_both_sides_pending` → `add_registers_both_sides`, 생성 측 자동 승인 ·
  `one_side_approval_is_not_paired` 는 다시 추가로 사람 승인 경로) · `events.rs`(첫 hello 는 `AutoPaired`, 재 hello 는 `PairRequest`) ·
  `limits.rs`(위) · `app` `chat_api.rs`(`conversation_post_targets` 의 사람에게 → 400 · `via_api` 의 이어 쓰기 → 에이전트에게) ·
  `cli.rs`(`say_*` 2건 삭제 → `say_is_unknown_command`) · `agent` `runner.rs`(`claude_args` 인자 · `Agent.group`).
- `cargo clippy --workspace --all-targets -- -D warnings` — 처음 `result_large_err` 3 · `map_entry` 1 · `useless_format` 1 → 수정 후 `Finished`.
- `cargo fmt --all --check` 통과 · `node --test` `# pass 13 # fail 0`(웹 변경 없음).
- mutation 5종 모두 FAIL: 자동 승인을 아는 키에도(`auto = true`) → auto_pair 1 failed · `_` 이스케이프 제거 → ux_api inbox 1 failed ·
  상대 덮어쓰기 무시(`resolve(None)`) → run_options 1 failed · `to: "human"` 허용 → ux_api to_human 1 failed · 피어 model 검증 제거 →
  ux_api peer_patch 1 failed.

## 리뷰

분리된 리뷰 에이전트(`034bf73..c8542e6`, 읽기 전용) — medium 2, low 3, nit 3. 코드로 확인 후:

| 지적 | 판단 | 조치 |
|---|---|---|
| **주소+코드 직접 추가는 키를 고정하지 않아(`Pin::Any`) 같은 LAN MITM 이 hello 의 평문 코드를 얻어 생성 측에 자기 키로 다시 보내면 자동 승인된다** — 전에는 생성 측 사람이 지문을 보고 승인했다 | 사실 · 해석 2(승인됨)가 직접 추가를 자동 등록 범위에 넣었다 | **미조치 — 위험 수용**(사용자 결정 2026-10-12, 원문 "2 위험수용. 왜냐하면 사내망에서만 사용할 내부용 툴이기 때문에 사용자간 보안보다는 성능/사용성이 높아야함.") → intent D8 "추가 수용" · 종결 시 masterPlan D8 환류. 초대 텍스트 경로는 지문을 고정해 안전 |
| 에이전트 폴더를 바꾸면 기존 대화 세션을 새 폴더에서 `--resume` 해 실패(세션은 폴더에 묶임) | 사실(Claude Code 세션 저장 위치가 폴더별 — 실측은 확인 필요) | `agents::update` 가 폴더가 바뀌면 그 에이전트의 세션(받은 대화 · 내가 연 자동 왕복)을 비운다(한 트랜잭션) + `paths` `schema9` 단정. mutation(조건 제거) FAIL |
| `AgentPatch` 오타 키를 조용히 무시 · `{"name": null}` 통과 | 사실 | `deny_unknown_fields`(`PeerPatch` 와 같게). `name: null` 은 이름을 바꾸지 않으므로 그대로 |
| 그룹 이름 제어 문자 · 개행 허용 | 사실 | `clean_group` 이 제어 문자 거부 + 단위 단정. mutation FAIL |
| 동시 PATCH 두 건의 갱신 유실(읽기 · 쓰기가 한 잠금 밖) | 사실(드묾) | 미조치 — 이 PC 사용자 한 명의 로컬 화면, 같은 피어를 동시에 고칠 경로가 없다 |
| `\` 이스케이프 제거 변이를 테스트가 못 잡음 | 사실 | `q=\1` 단정(막지 않으면 `\1` 이 `1` 로 읽혀 걸린다). mutation FAIL |
| 아는 키가 코드를 담아 와도 창을 쓰지 않는지 미검증 | 사실 | 같은 코드로 처음 보는 키가 뒤이어 자동 등록되는 단정 |
| 저장된 틀린 model · effort 가 건너뛰어지는 경로 미검증 | 사실 | `run_options` 에 `--dangerously-skip-permissions` · `turbo` · 전역 `-x` → `sonnet` · `low` 단정 |

반영 뒤: `passed=170 failed=0`(테스트 수 그대로 — 기존 테스트에 단정 추가) · clippy `Finished` · `cargo fmt --check` · E6 · E8 · E9 · E10 · E12 ✅.

## 다음 단계

Stage 3 — 웹UI(토큰 · 테마 · 설정 트리 · 표 · 아코디언 · 편집 폼 · markdown 뷰어 · HTML sandbox · '나' 목록 · 보낸 메시지 탭 · 방향 태그 ·
보내기 대상 제거) · E3 · E5 · E7.
