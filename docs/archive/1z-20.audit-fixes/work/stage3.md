# stage 3 — 개요 · 지울 코드 · 문서 · 버전(지적 20 · 결정 2)

## 목표

[plan](../plan.md) stage 3 — 개요 미리보기 절 머리의 접속 글을 지우고(결정 2), 지울 코드를 정리하고, 같은 개념 문서를 맞추고, 버전을 1.4.5 로.

## 산출 파일

- `crates/app/web/js/home.js` — `previewSectionEl` 머리 `.pv-pres` = 점만(상대가 없으면 "페어링 해제"), 화면 서명은 `presence().key`.
- `crates/app/tests/ux_literals.rs` — PRESENT 310행 literal 갱신 · ABSENT `("app", "p ? pres.text")`.
- 지울 코드: `crates/app/src/server.rs`(`api_status` = `Json(json!({…}))`, `json_escape` · 시험 삭제), `crates/app/src/tray.rs`(`user_guide_url` 시험 삭제),
  `crates/agent/src/chat/relay.rs`(`block_relay` 의 `peer` 인자 삭제), `crates/agent/src/runner.rs`(`check_version` — 실행 파일 경로별로 통과만 기억, `check_version_now`).
- 버전 1.4.5 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`.
- 문서: [사용자 가이드](../../../manual/user/user-guide.md)(3 폴더 칸 · 도구 · MCP · 폴더 밖 읽기 ⚠, 7 접속 상태 — 개요 절 머리는 점만, 10.3 상대별 실행 10건) ·
  [web-ui](../../../manual/dev/web-ui.md)(`.pv-pres` · 접속 글 자리 · markdown) · [an_agent](../../../manual/dev/an_agent.md)(실행 인자 — `--restricted` · `--strict-mcp-config` 없음 ·
  읽기 범위 문장 · 버전 확인 · 상대별 상한 · `taken` · 키 칸 · `open_refused`) · [an_core](../../../manual/dev/an_core.md)(스키마 30 · 사본 임시 이름 · 옛 폴더 · 건너뛰기 파일 ·
  `open_refused` · `peer_pending_runs`) · [an_app](../../../manual/dev/an_app.md)(`Rejected` · `hand_off` 기다림) · [developer-guide](../../../manual/dev/developer-guide.md)(건너뛰기) ·
  PRODUCT(#1z-20 줄) · README(1.4.5 줄).
- 캡처 `docs/images/home.png` · `group.png` 다시 찍음(절 머리에 접속 글 없음 · 머리 v1.4.5) — mock = 1z-19 `design/mock.py` 의 임시 사본(버전 글자만 1.4.5).

## 설계 결정

- 접속 점의 글 대체(aria-label 등)는 넣지 않았다 — "…통신" 을 다른 말로 다시 싣는 셈. 점은 지금처럼 `aria-hidden`.
- 목록 보기 · 레일 · 대화 머리글 · 설정 연결 줄의 접속 글은 그대로(결정 2).
- `check_version` 캐시는 통과만 — 낮은 버전은 매번 다시 본다(올린 뒤 바로 반영). 통과 = "최소 이상" 이라 `claude update` 뒤에도 맞다.
- an_agent 의 옛 실행 인자 설명(`--restricted` · `--strict-mcp-config` · `--tools`)은 #1z-9 이후 코드와 달랐다 — 지금 코드대로 고침.

## 테스트 결과

- `cargo test --workspace` rc 0 · `cargo clippy --workspace --all-targets -- -D warnings` Finished · node `# pass 181 · # fail 0` ·
  `check_code_tidy` PASS · `check_crates` violations=0 · `check_readme` readme ok · `check_ddd_docs` PASS.
- 화면(E12): mock + ego-browser 1440×900 — 개요 미리보기 절 머리 글 = "이준호· 대화 5· 안 읽음 1· 대기 요청 1" · "최유나· 대화 4· …" · "build· 대화 1"(DOM `.pv-head` 글), "…통신" 없음, 점은 남음.

## 다음 단계

커밋 → `check_gates` → 적대적 평가(`codex-peer-review` 라운드 1/2) → eval · report · 원장 수용 3행 · 종결.
