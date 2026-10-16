# eval.md — 1j.messenger-ui 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 755d8e0f26483b2b297492d2610f09a549045386

- [x] E1: A11 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A11 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.31s

- [x] E3: A1·A2·A4·A5·A6·A8·A11 — 화면 로직 node 테스트 95건 이상 · 실패 0(말풍선 쪽 · 날짜 구분 · 라벨 없음 · 대화방 미리보기 · 메뉴 세 범위 · 상태 색 · 그룹에 상대
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (9[5-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 134.534

- [x] E4: A2·A3·A4·A6·A7·A10 — 웹UI 문구 · 자리("대화 보기" · "원문 보기" · "북마크" · "즐겨찾기" · "페어링" · 프로필 상태 · 사람 아이콘, 없어진 것 0 — "보낸 메시지" ·
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E5: A8 — 상대 그룹 API(넣기 · 빼기 · 모르는 상대 400 · 그 상대의 뒤에 생긴 대화도 그룹 거르기에 · 삭제 시 함께 지움 · 대화를 하나씩 넣은 것은 남음)
  CHECK: cargo test -p coworkagent-app --test groups_api
  EXPECT: /test result: ok\. (1[6-9]|[2-9][0-9]) passed; 0 failed/
  EVIDENCE: test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.97s

- [x] E6: A8 — 스키마 18(`group_peers` · 옛 자유 입력 그룹 → 같은 이름 레일 그룹 · 사본 `.v17.bak`)
  CHECK: cargo test -p an-core --test store schema18
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.11s

- [x] E7: A11 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E8: A11 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E9: A11 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E10: A1·A2·A3·A4·A5·A6·A7·A8·A9·A10 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력은 바꾸지 않고, 즐겨찾기 · 북마크 · 그룹은 바꿨다 되돌린다): 말풍선 쪽 · 날짜 · 라벨 없음 · 머리 ·
  EVIDENCE: 2026-10-15 headless Edge · playwright(scratchpad `j1/e10.mjs`, 이 PC 실제 앱 — 릴리스 `task/1j-messenger-ui` 빌드(`tools/rebuild_run.py`)) · 스키마 17 → 18: 상태 "ok (스키마 18)" · 사본 `coworkAgent.db.v17.bak` · 그룹 2 그대로 · `group_peers` 0(옛 `peers.grp` 없음) / 화면 27/27 PASS — 그룹 JSON `peers` · 대화 JSON `last` · 레일 72px · 모양 4 · 이름 11px · 글자 타일 0 · 받기 가능 칩 없음 · 프로필 `st-online` · 이름 한 번 · 프로필 메뉴 menuitemradio 2 · '나' 메뉴 7 · 상대 타일 → "개요 · {상대}" · 상대 메뉴 4 · 누른 뒤 초점 = 타일 · Alt + ↓ 뒤 초점 유지 · 말풍선 mine 2 / theirs 2 · 날짜 구분선 2 · 방향 라벨 · 보낸 메시지 탭 없음 · 머리 4 단추 · 말풍선 북마크 아이콘(막대에 없음) · 북마크 넣고 빼기 → 머리 수 ""→1→"" · 원문 보기 · 대화방 ☆ 넣고 빼기 · 즐겨찾기 화면 · 타일 메뉴 "+ 새 그룹에 넣기…" → 임시 그룹에 상대 · 레일 그룹 타일 반영 · 연결 목록 절 · "그룹 ▾" 로 빼기 · 절 머리 ✕ 로 임시 그룹 삭제 · 설정 페어링 · 초대 두 단추 같은 class · 읽기 전용 · 직접 추가 · 에이전트 그룹 칸 없음 · JS 오류 0. 끝난 뒤 DB — 즐겨찾기 · 북마크 · 그룹 · `group_peers` 올리기 직후와 같음(첫 실행에서 스크립트 오류로 남은 ☆ 1건은 API 로 되돌리고 다시 실행). 앱과 연결 끊김(붉음)은 실제 앱을 끊지 않으려 mock 에서만 확인(stage 2).


WEAK: E10 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-15 · `/codex:adversarial-review --base 35ac69b --scope branch` · 세션 codex-review · HEAD `8005c22`(작업 트리 깨끗) · job `review-mv9j7zjr-s4plkh` —
  Verdict needs-attention(medium 2). Codex 는 읽기 전용 환경이라 Rust 시험을 다시 돌리지 못했다고 적음(node 95/95 확인) — 이 세션에서 다시 실행.

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 대화방 미리보기가 실제 마지막 말풍선과 달라질 수 있음 (chat.rs:1123-1126) — `last` 가 로컬 `id` 최대, 화면은 `(lc, uuid)` 순 | 사실 | `last` 네 칸을 `ORDER BY lc DESC, uuid DESC LIMIT 1` 로(화면과 같은 공통 순서). 시험 `store.rs` `summary_last_follows_canonical_order`(로컬 `id` 가 더 커도 공통 순서가 앞인 글) |
| [medium] 연결 그룹 이동이 부분 성공하면 상대가 두 그룹에 남음 (app.js:1101-1103) — 넣기 · 빼기 두 `PUT` | 사실 | `PUT /api/peers/{id}/groups {add, remove}` — `groups::change_peer_groups` 한 트랜잭션(모르는 그룹이 있으면 아무것도 안 바꿈 400 · 모르는 상대 404). 끌어 옮기기는 이것 한 번. 시험 `groups_api.rs` `peer_groups_move_atomic`(17건) · 매뉴얼 an_app |

- 수정 뒤: `cargo test --workspace` 실패 0 · clippy Finished · node 95/95 · `check_ddd_docs` 는 gate 재측정 요구(아래 재측정).

## 적대적 평가 — 라운드 2/2

- 2026-10-15 · `/codex:adversarial-review --base 35ac69b --scope branch` · 세션 codex-review(라운드 1 과 같은 세션) · HEAD `9f0933b`(작업 트리 깨끗) · job
  `review-mv9jwuch-0jd9af` — Verdict needs-attention(medium 2). 라운드 1 두 지적은 다시 나오지 않음.

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 미리보기와 시각 · 상태가 서로 다른 메시지에서 계산된다 (chat.rs:1117-1126) — `last` 는 `(lc, uuid)`, `last_state` · `last_at` 은 `id DESC` | 사실 | `last_state` · `last_at` 도 `ORDER BY lc DESC, uuid DESC LIMIT 1`(같은 글). 시험 `summary_last_follows_canonical_order` 에 순서가 엇갈리고 상태 · 시각이 다른 경우 추가 |
| [medium] CLI 가 쓴 마지막 글의 발화자가 사람으로 오표시된다 (chat.rs:167-174) — `LastMessage` 에 `via` 없음 | 사실 | `LastMessage.via` · 요약 SQL · API `last.via` → `roomPreview`(`authorLabel` 이 `via` 를 씀). 시험 `store.rs`(via cli) · `chat_api.rs`(웹UI 글 `via` null) · `view.test.mjs` `room_preview_cli`(내 · 상대 CLI) · 매뉴얼 an_core · an_app |

- 수정 뒤: `cargo test --workspace` 실패 0 · clippy Finished · node 96/96. **상한 도달 — 재리뷰하지 않는다**(전역 지침 · PROJECT.md).

- 라운드 1 · 2 지적 4건 모두 고침(위 표). 재리뷰는 하지 않아 2 라운드 수정분 자체는 리뷰되지 않았다.

## 미조치 지적

없음

