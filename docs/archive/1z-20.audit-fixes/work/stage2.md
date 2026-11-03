# stage 2 — 버그(지적 2 · 7 ~ 19)

## 목표

[plan](../plan.md) stage 2 — 점검 지적 2 · 7 ~ 19 를 시험과 함께 고친다.

## 산출 파일

| 지적 | 고친 곳 | 시험 |
|---|---|---|
| 2 자동 왕복 재개 멈춤 | `crates/agent/src/chat/execute.rs` — 대화 작업이 `pending` 을 읽을 때 `taken` 을 대기열에 남은 글로 줄임 | `tests/relay.rs` `stop_while_waiting_then_resume`(실행 자리 1 · 긴 질문이 자리를 쥔 채 상대 답 대기 → 멈춤 → 재개 → 실행) |
| 7 번호 목록 시작 번호 | `web/js/logic/markdown.js`(블록 `start`, 1 이면 없음) · `web/js/conv.js`(`el(b.type, { start })`) | `view.test.mjs` `markdown_blocks_safe`(2 시작 · 날짜 줄 2026) |
| 8 `*` 기울임 오판 | `markdown.js` 정규식 — 닫는 `*` 앞이 공백 아님 | 같은 시험(`match *.rs or *.toml` 그대로 · `*기울임*`) |
| 9 두 번째 실행 거짓 오류 창 | `crates/app/src/main.rs hand_off` — 감시 잠금이 잡혀 있으면 `HANDOFF_WAIT`(150초)까지, 아니면 3초. `watchdog::held` | 수동 재현(아래) |
| 10 DB 사본 | `crates/core/src/db.rs backup_before_migration` — `….vN.bak.tmp` 에 `VACUUM INTO` → `rename`, 남은 `.tmp` 지움 | `core/tests/paths.rs` `migration_backup_ignores_half_written_temp` |
| 11 옛 폴더 반쯤 옮김 | `crates/core/src/paths.rs move_legacy` — 같은 항목(신원 · DB)이 양쪽에 다 있을 때만 충돌, `agentNetwork.db*` 곁 파일은 새 DB 이름으로 | `legacy_half_moved_resumes` |
| 12 색인 | `db.rs` 스키마 30 — `outbox_msg(msg_id)` · `messages_conv_lc(conversation_id, lc, uuid)` | `schema30_indexes_used`(EXPLAIN QUERY PLAN 이 색인을 씀) |
| 13 공유 `exists` | `crates/agent/src/chat/sharing.rs share_refused` — 공유 머리의 `exists` 도 관람자 빼기 | `tests/share.rs` `second_owner_share_exists_drops_viewer` |
| 14 알림 상태 어긋남 | `crates/app/src/notify.rs update` — 저장 직후 `st.enabled` 반영 | `tests/autostart.rs` `autostart_off_failure_keeps_notify_in_sync` |
| 15 검증 실패 릴리스 반복 받기 | `crates/app/src/update.rs` — `Outcome::Rejected(태그, 사유)`(실행 파일 꼴 · 서명), `main.rs` 가 건너뛰기 파일에 씀 | `tests/update.rs` `update_rejects_non_exe` · `update_rejects_untrusted` 가 `Rejected` 를 기대 |
| 16 CLI 키 잠금 | `crates/agent/src/chat/{mod,tx}.rs` — 키 해시 16칸(`KEYED_LANES`), `keyed(key)` · 호출 두 곳 | 기존 같은 키 시험 그대로 |
| 17 CLI `history` 줄 위조 | `crates/app/src/cli.rs` — 본문 둘째 줄부터 두 칸 들여 씀 | `tests/cli.rs` `history_indents_multiline_body` |
| 18 초대 소모 | `crates/net/src/node.rs` — 등록 · 내 승인을 한 `write_tx`, 실패면 `reopen_window` | `net/tests/auto_pair.rs` `failed_accept_keeps_invite`(트리거로 승인 쓰기 실패) |
| 19 프로토콜 `_ =>` | `crates/net/src/proto.rs conv` · `ids_ok` — 대화 없는 종류를 하나씩 적음, `inbound.rs` 의 `ids_ok` 뒤 못 걸리는 빈 id 검사 3곳 삭제 | 컴파일 · 기존 시험 |

## 설계 결정

- 9 — plan 은 `control::START_LIMIT`(30초)였으나 감시의 자동 업데이트 확인 상한(연결 5 · 최신 10 · 받기 120초)을 덮지 못해 `HANDOFF_WAIT` 150초로(편차 — 항목 · EXPECT 그대로).
  감시가 죽었으면(잠금이 잡히면 바로 놓음) 3초에 끝 — 예전과 같다.
- 15 — `Skipped` 를 나누어 `Rejected` 를 새로 둠(부른 쪽만 건너뛰기 파일을 안다). 받기 실패 · 교체 실패는 그대로 `Skipped`(일시 오류 — 다음 시작에 다시).
- 16 — 키별 잠금 지도 대신 고정 16칸(커지지 않음). 같은 칸에 걸린 다른 키는 예전처럼 기다린다. 원래 `ponytail:` 주석(전역 한 줄)을 대신한다.
- 18 — DB 오류 타입은 `an_core::db::Error`(an-net 은 rusqlite 직접 의존 없음).
- 같은 개념 문서(an_app 업데이트 · 건너뛰기 파일 · an_core 경로 · web-ui 마크다운)는 stage 3 문서에서.

## 테스트 결과

- RED 먼저: `stop_while_waiting_then_resume`(마지막 기다림 relay.rs:676) · `markdown_blocks_safe`(start undefined) · `second_owner_share_exists_drops_viewer`(관람자 남음) ·
  `autostart_off_failure_keeps_notify_in_sync`(enabled 거짓) · `history_indents_multiline_body` · `failed_accept_keeps_invite`(반쪽 행) FAILED → 고친 뒤 ok.
  사본 · 옛 폴더 · 색인 시험은 고친 코드와 함께 썼다(고치기 전 코드로 돌리지 않음).
- 수동 재현(지적 9, 디버그 빌드 `target/t15/debug/coworkAgent.exe`, 데이터 루트 `%TEMP%\an-e9\data`): Python 이 `supervisor.lock` 을 쥐고(`msvcrt.locking`) 8초 뒤 `instance.json`
  (204 를 주는 로컬 HTTP) 을 씀 → 2초 뒤 띄운 앱 = 6.5초 기다려 `exit=0`, `/api/open` 1회 받음, 시작 오류 파일 없음(예전 = 3초에 "기존 인스턴스에 전달 실패" 창).
- `cargo test --workspace` rc 0(네 번 중 한 번 rc 101 — 실패 이름 없이 다시 돌리면 통과, `os error 10055` 류 간헐 · STATE 기존 후보) · clippy Finished ·
  node `# pass 181 · # fail 0` · `check_code_tidy` PASS.

## 다음 단계

stage 3 — 개요 절 머리 · 지울 코드 · 문서 · 1.4.5.
