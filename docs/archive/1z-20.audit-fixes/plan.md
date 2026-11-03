# plan — 1z-20.audit-fixes

> 근거: [intent](intent.md)(승인 2026-11-03 — 결정 1 ~ 5). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-20-audit-fixes`(원 작업 폴더 — worktree 없음). 기준 커밋: `6233339`(main — v1.4.4 릴리스 뒤).
> 승인: 2026-11-03 — 사용자 "승인."(라운드 2 개정본)
> 보정 2026-11-03(stage 1) — CHECK 만: E2 `--lib runner::tests` → `--test runner`(시험이 통합 시험 파일에 있다), E5 받기 거절 판정 시험 자리 `an-agent --lib inbound` → `an-core --test store open_refused`
>   (판정 `chat::open_refused` 를 rusqlite 를 가진 an-core 에 둠 — an-agent 는 rusqlite 직접 의존 없음). 항목 · EXPECT 변경 없음.
> 개정 2026-11-03 — intent 라운드 2(결정 6 — 도구 · 탐색 · MCP 최대 허용): 경로 금지 규칙 · `mcp__*` 금지 · Codex 환경 변수 목록 철회, A1 = 안내문 사실화 + 원장 수용. gate 다시 번호.

## 목표

2026-11-03 저장소 점검 지적 1 ~ 20 을 고치고(1 · 4 · 5 는 결정 6 — 최대 허용 · 원장 수용), 개요 미리보기 절 머리의 접속 글을 지운다. 버전 1.4.5.

### 성공 기준(intent 사본)

- A1: 원격 실행의 도구 · 탐색 · MCP 범위가 지금(최대 허용) 그대로이고, 안내문이 실제 범위와 같으며(사실 아닌 장치 주장 없음), 그 위험(폴더 밖 읽기 · MCP · Codex 환경 변수)이 원장에 `수용` 으로 올라 있다(라운드 2 개정).
- A2: 자동 왕복을 실행 대기 중에 멈추고 재개하면 그 상대 답이 앱을 다시 켜지 않고 실행된다 — 시험.
- A3: 범위의 나머지 지적(3 ~ 20)이 각각 고쳐졌거나(시험 · 실측) 결정으로 수용돼 원장에 올라 있다.
- A4: 개요 미리보기 절 머리에 실제 대화 시각과 다른 "…통신" 글이 보이지 않는다(결정에 따른 표시).
- A5: 기존 동작 회귀 없음 — `cargo test --workspace` · clippy · node 시험 · 문서 규약 · README · 코드 정리 · 크레이트 경계 PASS, 버전 1.4.5.

## 실측(plan 작성 시)

- 실행 인자 `claude_args` — 허용 `Read,Grep,Glob,WebFetch,WebSearch`(+ 셸 켬 `Bash`/`PowerShell`), 금지 `Edit,Write,NotebookEdit`(+ 셸 끔이면 셸) — `crates/agent/src/runner.rs:160-229`.
  경로 범위 없음 · 상위 읽기 안내문 "그 위는 읽을 수 없다"(`runner.rs:192`).
- claude 2.1.317 실측(scratchpad probe, `CLAUDE_CONFIG_DIR` 유지 — 기본 `~/.claude` 는 이 PC 에서 "OAuth token revoked"):
  - 금지 없음 + 폴더 밖 파일 Read → 내용 그대로(`leak=True`).
  - `--disallowedTools "…,Read(//c/Users/…/secret/**)"` → Read · Grep 모두 "Permission to read … has been denied".
  - `Glob(//…)` 규칙 → CLI 오류 "is not matched by file permission checks — only Read(path) rules are … (Read rules cover all file-reading tools)".
  - `mcp__*` 금지 → init 도구 273 → 30, MCP 243 → 0. `mcp__`(별 없음)는 효과 없음.
- 보호 경로 함수: `an_core::paths::protected_roots()`(지금 · 옛 데이터 루트) — `crates/core/src/paths.rs:165`.
- 대화 작업 `taken` — `crates/agent/src/chat/execute.rs:88-97`, 재개 재대기 — `relay.rs:186-214`, 멈춤 완료 — `execute.rs:160-162`.
- 받기 `open_conversation` 오류 = 모두 `bad_request` — `inbound.rs:196-198 · 423-425`. `open_conversation` 은 묘비면 `QueryReturnedNoRows`, 다른 상대 uuid 면 UNIQUE 위반.
- 실행 대기 질의 `pending` — `crates/core/src/chat/message.rs:342`(exec = 1 · state queued/running). `RETRY` = `"retry"`(`chat/mod.rs:115`), 보낸 쪽은 재시도(`delivery.rs:125`).
- Codex 실행 `env: None` — `runner.rs:824-860`, 셸 변수 목록 `SHELL_ENV_KEEP` · `shell_env` — `runner.rs:129-153`.
- 웹 마크다운 `logic/markdown.js:19 · 72-76`, 그리기 `conv.js:38`. 시험 `view.test.mjs:534 markdown_blocks_safe`.
- 개요 절 머리 `home.js:171-174`(`.pv-pres` = 점 + `pres.text`), 서명 `home.js:141`. 화면 literal `ux_literals.rs:310`.
- 그 밖 위치는 점검 보고(대화 2026-11-03) 그대로 — 아래 stage 항목에 적는다.

## 설계

### 공통 규칙

새 함수 · 상수는 둘레 규약대로(파일 전용 `static` 함수 없음). 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저(RED 확인).
같은 개념 주석 · 매뉴얼은 함께 고친다. `cargo test` 는 PowerShell · `CARGO_TARGET_DIR=target\t15`. `cargo fmt` 쓰지 않음.
원 작업 폴더의 다른 세션 미커밋(`PRODUCT.md` 끝 2줄 · 미추적 tech 노트 security-posture)은 커밋에서 뺀다.

### stage 1 — 보안(지적 1 안내문 · 3 · 6)

1. 지적 1 — 막지 않는다(결정 6). `runner.rs` 셸 끔 상위 읽기 안내 "이 폴더와 바로 위 폴더 {up} 아래를 읽을 수 있다. 그 위는 읽을 수 없다." →
   셸 켬과 같은 사실 문장("파일 도구는 이 폴더와 바로 위 폴더 {up} 아래를 읽는다.") — 장치가 막지 않는 것을 막는다고 말하지 않는다. 같은 개념 주석(`runner.rs:192` 위 주석) 함께.
   `GUARD_PROMPT` · 편집 도구 금지 · `tools_canary` 는 그대로. MCP · 탐색 · Codex 환경 변수도 그대로(지적 4 · 5).
2. 지적 3 — `inbound.rs` 두 곳: `open_conversation` 오류가 `QueryReturnedNoRows` · `SqliteFailure(ConstraintViolation)` 면 `bad_request`, 그 밖은 `db_err` 로 올려 `retry`.
   함수 `open_refusal(e) -> io::Result<Response>` 하나로 두 곳 공유 + 단위 시험(BUSY 류 → Err, 제약 → bad_request).
3. 지적 6 — `an_core::chat::peer_pending_runs(conn, peer_id) -> i64`(받은 대화 `dir = 'in'` · exec = 1 · queued/running) + 상수 `PEER_RUN_CAP = 10`.
   `inbound_message` 의 집중 판정 뒤 · 새 글 기록 전: 재전송(`resent`)이 아니고 cap 이상이면 `RETRY`. 시험(`tests/capacity.rs`): 10건 대기 중 11번째 = retry, 재전송은 받음.

### stage 2 — 버그(지적 2 · 7 ~ 19)

1. 지적 2 — `execute.rs` 대화 작업: `pending` 을 읽은 뒤 `taken.retain(|id| ms.iter().any(|m| m.id == *id))`. 시험(`tests/relay.rs`): 실행 자리를 막아 둔 채 상대 답 대기 → 멈춤 → 재개 → 실행됨.
2. 지적 7 · 8 — `markdown.js`: 번호 목록 블록에 `start`(첫 항목 숫자), `conv.js` 가 `el("ol", { start })`(1 이면 생략). `*` 기울임 정규식 = 닫는 `*` 앞이 공백 아님.
   `view.test.mjs`: "2. a" 시작 · "match *.rs or *.toml" 그대로.
3. 지적 9 — `main.rs` 감시 `Ok(None)`(이미 실행 중): 감시 잠금이 잡혀 있는 동안은 `hand_off` 를 `control::START_LIMIT` 까지 다시 시도(지금 10 × 300ms 고정). 시험이 어려우면 수동 gate(E9).
4. 지적 10 — `db.rs backup_before_migration`: 같은 폴더 임시 이름(`….vN.bak.tmp`)에 `VACUUM INTO` → `rename`. 이미 있는 사본은 그대로 둔다(되돌리기 사본 = 처음 것 — 매뉴얼 규칙 유지), 남은 `.tmp` 는 지우고 다시.
   시험: 남은 `.tmp` 가 있어도 사본이 온전히 생김.
5. 지적 11 — `paths.rs move_legacy`: 충돌 = 같은 이름이 양쪽에 다 있을 때만(반쯤 옮긴 상태는 이어서 옮김). `-journal` · `-wal` · `-shm` 도 새 DB 이름으로. 시험(`tests/paths.rs`): DB 만 옮겨진 상태 → 다음 실행이 끝까지 옮김.
6. 지적 12 — 스키마 30: `CREATE INDEX outbox_msg ON outbox(msg_id)` · `messages_conv_lc ON messages(conversation_id, lc, uuid)`. `SCHEMA_VERSION` 30, 이전 시험 기대 갱신.
7. 지적 13 — `sharing.rs share_refused`: `SHARE` 머리의 `exists` 도 `not_paired` · `unsupported` 처럼 관람자 해제 + 알림. 시험(`tests/share.rs`).
8. 지적 14 — `notify.rs update`: 커밋 직후 `st.enabled` 반영(자동 시작 되돌리기 실패 경로에서도). 시험(가능하면 훅 주입).
9. 지적 15 — `update.rs check`: 받은 파일의 `trusted` 실패(서명 · 제품 · 버전) · PE 아님이면 그 태그를 건너뛰기 파일에 쓴다. 시험(`tests/update.rs` 로컬 HTTP).
10. 지적 16 — CLI 키 잠금: tx 행을 기록한 뒤 `dispatch` 전에 잠금을 놓는다(`chat_api/conversation.rs:127-131` · `tx.rs:42-43`). 같은 키 두 번 = 같은 tx(지금 시험 유지).
11. 지적 17 — `cli.rs history`: 본문 둘째 줄부터 두 칸 들여 쓴다. 시험(본문 줄바꿈 → 들여 씀).
12. 지적 18 — `node.rs` 초대 수락: `upsert` · `set_my_ok` 를 한 `write_tx` 로, 실패면 창을 되돌린다(또는 창 소모를 기록 성공 뒤로). 시험(`crates/net/tests`).
13. 지적 19 — `proto.rs ids_ok` · `conv()` 의 `_ =>` 를 모든 종류 나열로. `inbound.rs:154 · 285 · 501` 의 `conv.is_empty() || msg.is_empty()`(ids_ok 뒤라 못 걸림) 삭제.

### stage 3 — 개요 · 지울 코드 · 문서 · 버전(지적 20 · 결정 2)

1. 결정 2 — `home.js previewSectionEl`: `.pv-pres` = 점만(`p` 없으면 "페어링 해제" 글), 서명은 `pres.key`. `ux_literals` 310 행 literal 갱신 + ABSENT(`pres.text` 가 절 머리에 없음).
2. 지적 20 — `server.rs api_status` = `Json(json!({…}))`, `json_escape` · 시험 삭제(CLI 는 serde 로 읽음 — 필드 이름 · 값 그대로). `tray.rs user_guide_url` 시험 삭제.
   `relay.rs block_relay` 의 `peer` 인자 삭제(호출부 함께). `check_version` 결과를 실행 파일 경로별로 한 번만(프로세스 수명 `OnceLock<Mutex<HashMap>>` — 실패는 캐시하지 않음).
3. 문서: [사용자 가이드](../../manual/user/user-guide.md)(3 에이전트 등록 — 원격 질문도 이 PC 의 도구 · MCP 를 쓰고 폴더 밖 파일도 읽을 수 있음(편집만 막힘) · 7 접속 상태 — 개요 절 머리는 점만) ·
   [web-ui](../../manual/dev/web-ui.md)(102 · 288) · [an_agent](../../manual/dev/an_agent.md)(실행 인자 · 상대별 상한) · [an_app](../../manual/dev/an_app.md)(필요 시) · PRODUCT · README 1.4.5 줄.
4. 버전 1.4.5(크레이트 넷 · Cargo.lock).
5. 원장(종결 때): `#1z-20 R0-1` 폴더 밖 읽기(셸 끔 파일 도구 — 신원 키 · 대화 DB · `~/.claude` · `~/.ssh`) · `R0-4` 사용자 허용 MCP(쓰기 포함) · `R0-5` Codex 환경 변수 — 셋 다 `수용`(결정 6 — 사용자 감수).

## 평가 항목

- [ ] E1: A1 — 안내문에 사실 아닌 장치 주장("그 위는 읽을 수 없다") 없음
  CHECK: python -c "print('claim=%d'%open('crates/agent/src/runner.rs',encoding='utf-8').read().count('그 위는 읽을 수 없다'))"
  EXPECT: claim=0
  EVIDENCE: pending
- [ ] E2: A1 — 실행 인자 회귀 없음: 편집 도구만 막고 읽기 · 웹 도구 미리 허용, MCP · 경로 금지 규칙 없음
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E3: A1 — 원장 `수용` 행 3개(#1z-20 R0-1 · R0-4 · R0-5)
  CHECK: python -c "import re;t=open('docs/overview/ledger.md',encoding='utf-8').read();print('rows=%d'%len(re.findall(r'^\| #1z-20 R0-[145] \|.*\| 수용 \|',t,re.M)))"
  EXPECT: rows=3
  EVIDENCE: pending
- [ ] E4: A2 — 실행 대기 중 멈춤 → 재개 → 실행(앱 재시작 없이)
  CHECK: cargo test -p an-agent --test relay
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A3 — 받기 DB 오류 = retry · 상대별 실행 상한(지적 3 · 6)
  CHECK: cargo test -p an-core --test store open_refused && cargo test -p an-agent --test capacity
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A3 — DB 사본 임시 이름 · 옛 폴더 반쯤 옮김 · 스키마 30 인덱스(지적 10 · 11 · 12)
  CHECK: cargo test -p an-core
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A3 — 공유 `exists` · 초대 쓰기 실패(지적 13 · 18)
  CHECK: cargo test -p an-agent --test share && cargo test -p an-net
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A3 — 알림 상태 · 검증 실패 건너뛰기 · CLI history 들여 씀 · status JSON(지적 14 · 15 · 17 · 20)
  CHECK: cargo test -p coworkagent-app --test update && cargo test -p coworkagent-app --lib
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E9: A3 — 감시가 업데이트 확인 중일 때 두 번째 실행이 오류 창 없이 넘겨준다(지적 9) — 재현(느린 릴리스 주소) 또는 시험, 평가자 대조
  EVIDENCE: pending
- [ ] E10: A3·A4 — 웹 로직: 번호 목록 시작 번호 · `*` 오판 없음 · 개요 절 머리 점만
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E11: A4 — 화면 literal: 개요 절 머리에 접속 글 없음 · 점 유지
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E12: A4 — 실제 화면(디버그 빌드 · 시험 데이터): 개요 미리보기 절 머리 = 이름 · 점 · 대화 수, "…통신" 없음 · 레일 · 대화 머리글 접속 글은 그대로 — 평가자 대조
  EVIDENCE: pending
- [ ] E13: A3 — 지울 코드: `json_escape` · `user_guide_url` · `let _ = peer` 없음
  CHECK: python -c "import pathlib as P;s=lambda f:P.Path(f).read_text(encoding='utf-8');print('left=%d'%(s('crates/app/src/server.rs').count('json_escape')+s('crates/app/src/tray.rs').count('fn user_guide_url')+s('crates/agent/src/chat/relay.rs').count('let _ = peer')))"
  EXPECT: left=0
  EVIDENCE: pending
- [ ] E14: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E15: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E16: A5 — 버전 1.4.5(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v145=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.5' for t in v)))"
  EXPECT: crates=4 v145=4
  EVIDENCE: pending
- [ ] E17: A5 — 문서 규약 · README · 코드 정리 · 크레이트 경계
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_code_tidy.py && python tools/check_crates.py
  EXPECT: /violations=0/
  EVIDENCE: pending
