# eval.md — 1l.ui-icons 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: e1db5f853b30f94376c3ebc7c0f2fa6cda7401a6

- [x] E1: A6·A7 — 워크스페이스 테스트 전부 통과(타일 순서 설정 기본 · 저장 · 틀린 값 400)
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A1·A4·A5·A6·A8 — 화면 로직 node 테스트 100건 이상 · 실패 0(메뉴 · 트리 아이콘 · 트리에 내 정보 없음 · 프로필 "사용자 정보" · 새로 만들기에 자동 왕복 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (1[0-9]{2})\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 112.4291

- [x] E4: A1·A2·A3·A4·A5 — 웹UI 문구 · 자리(아이콘 · `data-tone` · '?' · `me_dialog` · 한 열 · 등록 칸, 없어진 것 0 — `agent_q` · "새 에이전트 등록" · 설정 내 정보 카드)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E5: A6 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E6: A6 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E7: A6 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E8: A1·A2·A3·A4·A5 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력 · 설정 값은 바꾸지 않음): 메뉴 · 제목 · 트리 · ⋯ 아이콘 · 말풍선 색(내 글 강조색 · 상대 타일
  EVIDENCE: 2026-10-16 headless Edge · playwright(scratchpad `l1/e8l.mjs`, 이 PC 실제 앱 — 릴리스 `task/1l-ui-icons` 빌드(`tools/rebuild_run.py`), 비파괴 — 화면 · 읽기만, 등록 칸 · 팝업은 열었다 닫음, 타일 순서는 그룹 → 사용자로 바꿨다가 원래 값 `peers_first` 로 되돌림) · 15/15 PASS(라운드 2 수정 `6e9fdac` 뒤 재빌드 · 재측정) — 사이드바 · 화면 제목 · 설정 트리 · 대화 ⋯ · 타일 ⋯(상대 7 항목 · 그룹 — 타일 편집 · 위로 · 아래로 · 그룹 삭제 휴지통, 예외 없음) 아이콘 모두 16×16 · 말풍선 내 글 rgb(78,85,217)(= `--accent`) · 상대 글 `data-tone` peer-2 · 바탕 `--surface` 위 타일 색 층 `::before` rgb(47,158,122) 불투명도 0.12 · 띠 3px · 새로 만들기 4 항목(새 대화 · 초대 생성 · 초대 수락 · 에이전트 등록, 자동 왕복 없음) · 트리에 내 정보 없음 · 카드 7개 한 열(conn · peers · agents · run · screen · startup · notify, 왼쪽 같음) · 늘 보이는 카드 안내 0 · '?' 올리면 뜸 · 검색칸 없음 · 등록 칸 보이기(초점 `agent_name`) · 숨기기 · 타일 순서 그룹 → 사용자 = 레일 DOM · Tab 순서 그룹 먼저 · 새로 고침 뒤 유지 · 되돌린 뒤 원래 순서 · 프로필 "사용자 정보" 팝업(지문 표시) · Esc 닫힘 · JS 오류 0


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-16 · `/codex:adversarial-review --base 473441a --scope branch` · 세션 codex-review · HEAD `73ac7f3`(요청과 일치) · job `review-mva6zfgo-uele5k` —
  Verdict needs-attention(high 1).
- **[high] 그룹 타일 ⋯ 메뉴가 생성 중 예외로 중단됨**(`app.js` 778-785) — 판단: **사실**. `item(icon, text, onclick, cls)` 로 바꾸며 "그룹 삭제" 호출만 옛 3인자로
  남아 `onclick` 자리에 `"danger-text"` 가 들어가 `addEventListener` 가 `TypeError` — 그룹 타일 메뉴 전체가 안 열렸다. mock · E8 이 그룹 타일 메뉴를 열지 않아
  놓쳤다. **고침** `f312f9a` — `item("trash", "그룹 삭제", …, "danger-text")` · `.ico-trash` · node `icons_defined` 에 "`item(` 첫 인자는 아이콘 이름" 검사(고치기 전
  코드로 돌리면 실패 확인 — 99/100). 권고의 DOM 회귀 시험은 node 시험에 DOM 이 없어 정적 검사로 대신하고, E8 에 그룹 타일 ⋯ 열기를 더했다(15/15).

## 적대적 평가 — 라운드 2/2

- 2026-10-16 · `/codex:adversarial-review --base 473441a --scope branch` · 세션 codex-review(라운드 1 과 같음) · HEAD `303db62`(요청과 일치) · job
  `review-mva7cxtl-7bv84c` — Verdict needs-attention(medium 1).
- **[medium] Safari 15에서 상대 말풍선의 색상 배경이 적용되지 않음**(`app.css` 669) — 판단: **사실**. 지원 대상 macOS 12 의 기본 Safari 15 에는 `color-mix()` 가 없어
  (Safari 16.2 부터) 상대 말풍선 바탕 · 테두리 선언이 버려지고 띠만 남는다. **고침** `6e9fdac` — `color-mix()` 를 없애고 바탕 `--surface` 위에 타일 색 층
  (`.bubble::before`, `opacity: 0.12`, `z-index: -1` + `isolation: isolate` — 글 뒤, 띠는 덮지 않음)을 깔았다(Safari 15 가 아는 속성만). `ux_literals` 가
  `app.css` 에 `color-mix(` 없음 · 새 규칙을 확인. mock 다크 · 라이트 화면 같은 모양 · E8 재측정 15/15(층 색 · 불투명도 0.12 확인).
- 상한 도달 — 재리뷰하지 않는다.

## 미조치 지적

| 지적 원문(요약) | 판단 | 미조치 사유 |
|---|---|---|
| 라운드 2 권고 — Safari 15/16.1 에서 상대 말풍선 배경 · 띠를 확인하는 호환성 검증 추가(원장 `#1l R2-SAF`) | 사실(권고) | 이 PC(Windows)에 macOS Safari 15 가 없어 실기 확인 불가(확인 필요). 고친 규칙은 Safari 15 지원 속성(`::before` · `opacity` · `inset` 14.1+ · `isolation`)만 씀 — 19 인수 TC(macOS 12) 때 확인. 라운드 상한 |
