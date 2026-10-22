# plan — 1z-7.conv-type-look

> 근거: [intent](intent.md)(승인 2026-10-23 — 결정 1 ~ 9). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D21 에이전트 · D35 화면 · S4).
> 브랜치 `task/1z-7-conv-type-look`(작업 폴더 `D:\dev\git\coworkAgent-1z-7`). 기준 커밋: `fbd7bad`(origin/main). plan 승인: 2026-10-23(인터뷰 답 U1 ~ U10 으로 갈음 — intent 결정 9).
> 시안: https://claude.ai/artifact/A3m1CVHdpgjYemN29c8qKW (시안 A · AgentEdit 제안).

## 목표

대화 유형(내 에이전트 응답 `in` · 내가 문의 `out` · 공유 `shared`)을 대화 머리 · 사이드바 줄 · 개요(미리보기 · 목록)에서 같은 색 · 아이콘으로 가르고,
설정 '내 에이전트' 상세를 한 격자 · 아이콘 색 · 닫기로 정리한다. 버전 1.3.3.

### 성공 기준(intent 사본)

- A1: 대화 화면 머리 제목 앞에 `in` = 로봇 · `out` = 사람 아이콘(유형 색 칩), `shared` = 없음, 머리 아래 선 = 유형 색(시험 · 실제 화면).
- A2: 사이드바 대화 줄(대화 · 즐겨찾기 · 에이전트 응답) · 개요 미리보기 카드 · 개요 목록 줄의 테두리 색 · 아이콘이 A1 과 같은 유형 규칙 — `in` 청록 · `out` 파랑 · `shared` 회색 점선, 고른 줄 · 안 읽음 표시가 유형 테두리를 덮지 않는다(시험 · 실제 화면, 라이트 · 다크).
- A3: 설정 '내 에이전트' 상세 — 이름 · 아이콘 색 · 폴더 · 상위 탐색 · 셸 · 도구 · 추가 지시가 한 격자(값 열 한 줄 맞춤), 단추 저장 · 삭제 · 닫기, 닫기 = 상세 접힘 · 초점이 연필 단추로(실제 화면).
- A4: 아이콘 색 6개(`agent-tile` · `peer-1` ~ `peer-5`) 중 고르고 저장하면 다시 열어도 남고, 설정 머리 · '등록된 에이전트' 줄의 로봇 타일이 그 색 — 목록 밖 값 400, 색만 바꾸면 세대(`uid`) · 세션 그대로, 스키마 26(시험 · 실제 화면).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 버전 1.3.3, 매뉴얼 · masterPlan 반영.

## 설계

- core `agents`: `ICON_COLORS` `["agent-tile", "peer-1" … "peer-5"]`(첫 값 기본) · `Agent.color` · `set_color(conn, id, color)`(색만 — `uid` · 세션 그대로).
  `db` 스키마 26: `agents.color TEXT NOT NULL DEFAULT 'agent-tile'`(`has_column` 가드, 한 트랜잭션).
- app `chat_api`: `AgentPatch.color: Option<String>` — 목록 밖 400("color 는 …", 다른 칸을 쓰기 전에 검사), `agents::update` 뒤 `set_color`. `agent_json` 에 `color`.
  상대 PC 에 보내는 에이전트 목록(`names`) · CLI 출력에는 넣지 않는다.
- web `view.js`: `AGENT_COLORS` · `agentColor`(모르는 값 = 기본) · `convKindIcon`(`in` bot · `out` user · 그 밖 null) · `KIND_NAMES` · `PARENT_HINT` · `SHELL_HINT`,
  `agentPatch` 는 색이 바뀐 경우만 `color`.
- web `app.js`: `kindChip(c, cls)`(유형 칩 — 공유 사본은 빈 값) · `agentTile(color, cls)`(로봇 타일). `renderConvHead` — `#conv_kind`(아이콘 · 이름 · 숨김) · `.conv-head[data-kind]`.
  `convItem` · 개요 목록 줄 · `previewCardEl` 에 `data-kind = c.dir` + 칩. `renderAgentList` — 한 격자(이름 · 아이콘 색 `swatches` · 폴더 · 체크박스 `check-line` · 도구 · 추가 지시),
  표(`kvTable`) 제거, 단추 저장 · 삭제 · 닫기(연필 `click()` + `focus()`). `renderAgentsView` 내 에이전트 줄 = `agentTile(a.color, "small")`.
- web `index.html`: 제목 앞 `<span id="conv_kind" class="conv-kind kind-chip" role="img" hidden>`.
- web `app.css`: `:root` `--conv-in`(→ `--agent`) · `--conv-out`(→ `--accent`) · `--conv-shared`(→ `--faint`). `[data-kind]` → `--kind` · `--kind-soft`. 칩 = `--kind` 면 + `--surface` 아이콘.
  머리 아래 2px · 사이드바 줄 테두리 · 고른 줄 옅은 바탕 + 안쪽 테두리 · 카드 1.5px(안 읽음 `.pv-card.unseen` = 바깥 `outline`, `:hover` = 그림자 — 강조 테두리를 바꿈) ·
  목록 줄 테두리 · 공유 점선. 상세: `.check-line` · `.swatches` · `.swatch[aria-pressed]` · `.agent-pa`.
- 시험: core `paths` `schema26_agent_color` · app `chat_api` `agents_crud_api`(색 PATCH · 400 · 목록) · `view.test.mjs` `agent_color` · `conv_kind_icon` · `ux_literals`(#1z-7 줄,
  #1z-2 로봇 줄 · #1k 타일 줄 개정, 이름 표 없음). 에이전트 시험의 `Agent { … }` 에 `color`.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [an_app](../../manual/dev/an_app.md) · [an_core](../../manual/dev/an_core.md) · [사용자 가이드](../../manual/user/user-guide.md) ·
  PRODUCT · README(1.3.3) · masterPlan D21 · D35 · 개정 기록(종결). 버전 네 크레이트 1.3.3.

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2·A4 — 웹UI 로직 시험(유형 아이콘 · 색 기본값 · 바뀐 색만 보냄)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A4 — 스키마 26 · 기본 색 · 색만 바꾸면 세대 · 세션 그대로 · API 색 저장 · 목록 밖 400
  CHECK: cargo test -p an-core --test paths schema26 && cargo test -p coworkagent-app --test chat_api agents_crud
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A2·A3 — 화면 원본 literal(제목 칩 · 유형 토큰 · 머리 선 · 줄 테두리 · 카드 윤곽선 · data-kind 셋 · 이름 · 아이콘 색 · 닫기 · 로봇 타일, 이름 표 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A5 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d webui=%d core=%d app=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.3\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/web-ui.md').count('**대화 유형 색**(#1z-7'),r('docs/manual/dev/an_core.md').count('스키마 26(#1z-7'),r('docs/manual/dev/an_app.md').count('color?}'),r('docs/manual/user/user-guide.md').count('**대화 유형 색**')))"
  EXPECT: ver=4 webui=1 core=1 app=1 guide=1
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A4 — 실제 화면: 디버그 빌드(시험 데이터 폴더 — `in` · `out` · `shared` 대화 시드)에서 대화 머리 아이콘 · 선, 사이드바 줄 · 개요 카드 · 목록 줄 색 · 아이콘,
  고른 줄 · 안 읽음, 라이트 · 다크, 내 에이전트 상세 한 격자 · 닫기(초점) · 아이콘 색 저장 → 다시 열어도 남음 · '등록된 에이전트' 타일 색
  EVIDENCE: Claude(구현자) 2026-10-23 — `target/debug/coworkAgent.exe start --parent <대기 프로세스>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z7\data`(사용자 앱 그대로, 상대 2 · 대화 5(in 2 · out 2 · shared 1) · 글 10 sqlite 시드, 에이전트 coder API 등록), ego-browser DOM 판독 · 스크린샷.
  관측: `#c/1`(in) 제목 칩 보임 · 이름 "내 에이전트 응답" · `ico-bot` · 면 rgb(12,108,104) · 머리 아래 solid rgb(12,108,104) → `#c/2`(out) "내가 문의" · `ico-user` · 면 · 선 rgb(36,91,171),
  고른 줄 바탕 rgb(232,241,254)(`--accent-soft`) → `#c/5`(shared) 칩 hidden · 선 dashed rgb(104,108,113). 사이드바(나) out 줄 테두리 rgb(36,91,171) · 사람 칩, 공유 dashed · 칩 없음,
  에이전트 응답 화면 in 줄 2 = rgb(12,108,104) · 로봇 칩. 개요 미리보기 카드 out/in/shared = 1.5px(화면 1.39px) solid 파랑 / solid 청록 / dashed 회색, 칩 사람 / 로봇 / 없음,
  안 읽음 카드(공유) = 테두리 그대로 dashed + 바깥 outline solid rgb(36,91,171) — 카드 간격에서 잘리지 않음(스크린샷). 다크(`data-theme=dark`) 카드 테두리 rgb(127,172,240) · rgb(113,194,188) ·
  rgb(143,147,151), 칩 아이콘 rgb(23,24,27). 개요 목록 줄 5 = in/out 칩 · 테두리, shared 처음 solid → 결함(같은 우선순위 규칙이 뒤에서 덮음) → 점선 규칙을 뒤로 옮김 → 재빌드 뒤 dashed(목록 · 사이드바 ·
  카드 모두). 설정 '내 에이전트' 상세: 격자 7줄(이름 · 아이콘 색 · 폴더 · 상위 · 셸 · 도구 · 추가 지시) 값 열 left 611px 모두 같음 · 단추 "저장" "삭제" "닫기" · 색 단추 6(처음 agent-tile pressed) →
  peer-3 → 저장 = 머리 타일 `pa agent-pa peer-3` rgb(123,89,139) → 새로 고침 뒤 그대로 → 닫기 = 상세 hidden · `aria-expanded` false · 초점 `agent1_manage` → '등록된 에이전트' 내 줄 = `pa agent-pa peer-3 small`.
  끝에 `coworkAgent stop`.

## 개정 기록

- 없음
