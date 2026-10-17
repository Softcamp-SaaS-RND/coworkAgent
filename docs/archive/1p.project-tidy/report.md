# report — 1p.project-tidy

종결: 2026-10-17 · 브랜치 `task/1p-project-tidy`(main 합치기 · 릴리스는 사용자 확인 뒤)

## 결과

매 세션 필독을 루트 `CLAUDE.md` · `STATE.md` · `PROJECT.md` · ddd-workflow 4문서 18,915 바이트로 줄이고(27,173 — masterPlan 까지 읽던 체인은 60.1KB),
나머지는 `CLAUDE.md` "필요할 때 여는 문서" 표로 돌렸다. ddd-workflow 의 문서 체계 · 루트 3문서 계약 · 원장 계약은 doc-conventions 로, `PRODUCT.md` 도메인
규칙의 흐름 · 화면은 개발자 가이드 절 · 새 web-ui.md 로 옮겼다(`PRODUCT.md` 31,673 → 15,253 바이트). 코드는 동작 그대로 죽은 함수 · 중복 도우미 · 미사용
의존을 지우고 문구 시험을 표로 바꿨다(24 → 5 시험, 잠근 단정 258 / 132 / 3 그대로). 화면은 Claude Design 캔버스의 시안 셋 중 **A 먹선**(쪽빛 강조 · 가는
선 · IBM Plex Sans KR 포함)을 골라 토큰 92 로 맞췄다 — 첫 `:root` 밖 리터럴 296곳(+ 라운드 1 이 찾은 3곳) → 0 · AI 티 규칙 7 → 0 · 대비 미달 12 → 0.
화면 문구 193행을 승인 표대로 고치고 용어 7개를 한 이름으로 맞췄다. 재발은 시험(`css_tokens` · `ux_literals` · `view.test.mjs`)과 측정 스크립트
([design/gates.py](design/gates.py))가 막는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 필독 ≤ 20KB · 필독 목록 셋 | 충족 | E1 `files=4 bytes=18915 under20k=True must_read_ok=True` · E2 `targets=167 missing=0` |
| A2 `PRODUCT.md` ≤ 15KB · 옮긴 사실 남음 | 충족 | E1 `product=15253 under15k=True` · E2 |
| A3 동작 변경 없는 코드 정리 | 충족 | E3 `notice_for=0 db_err_defs=1 agent_serde=False ux_tests=5` · stage2 `gates.py ux 303f65a` `present=258/258 absent=132/132 counts=3/3` |
| A4 첫 `:root` 밖 리터럴 0 | 충족 | E4 `css_tokens` 3 passed(검사 선언 948 — stage3) · 라운드 1 · 2 보강(`var()` 대체 값 · `1px` 테두리만 · 지역 사용자 속성) |
| A5 고정폭 허용 목록 | 충족 | E5 `mono=.count\|.fp\|.kbd\|.md code\|.mono\|.raw\|.steps .at\|kbd end` |
| A6 승인 문구 반영 · 가이드 · README 옛 문구 0 | 충족 | E6 `rows=193 applied=193 stale=0 doc_stale=0` |
| A7 한 개념 한 이름 | 충족 | E7 `terms=7 banned=16 banned_hits=0` |
| A8 화면 회귀 없음 | 충족(자동) · 눈 비교는 19 인수 | E10 · E11(node 화면 시험 112 · clippy 경고 0) — E8 ABANDON(D28, 원장 `#1p E8`) |
| A9 로컬 정리 · STATE · 캡처 · 용어 표 | 충족 | E9 `t12=False wt=False state_stale=0 shots=5 stale_shots=0 orphans=0` · E7 |
| A10 검증 명령 | 충족 | `check_gates` 14건 미충족 0 · ABANDON 1(E8) |
| A11 시안 A 토큰 · AI 티 0 · 대비 4.5:1 | 충족 | E13 `tokens=92 mismatch=0 slop_rules=7 slop=0` · E14 `pairs=122 fail=0` |

- 적대적 평가 2 라운드(상한 — 세션 codex-review, base `0c5d81e`, job `review-mvc86mwc-wbjrux` · `review-mvc98fao-y8qyc8`). 지적 6건 모두 사실 — 지적한
  경로는 모두 고쳤다(라운드 1: `var()` 대체 값 · 간격 1px → `app.css` 3곳 · gate 모집단 단정 · 중첩 템플릿 lexer, 라운드 2: 지역 사용자 속성 · 모집단 내용
  digest · 문장 자리 정규식). 권고 중 둘은 상한에서 남겼다 — R1(EXPECT 정확값)은 사용자 승인으로 plan 개정 2 → 닫힘, R2(실제 JS 파서)는 수용([eval](eval.md),
  원장 `#1p R1` · `R2`).
- plan 편차(모두 기록 · 승인): 개정 1 디자인 병행(A11 · E13 · E14 · 해석 18 ~ 22) · 개정 2 EXPECT 모집단 정확값 · 문구 표를 plan 부속 plan-copy 로(결정 5) ·
  표 밖 발견 Y 11행(토크나이저 결함으로 빠졌던 view.js 문구 — Y5 는 같은 묶음이라 함께) · 표 밖 화면 손질 4건(stage4) · #1j 없음 2행 좁힘 · 글꼴은 부분
  집합 대신 IBM 배포 그대로(OFL 예약 이름 — 실행 파일 +1,317,296 B, +14%) · 간격 1px → 0(라운드 1) · 1f 범위 밖 관찰은 원장 대신 masterPlan 후보로
  (plan-docs 8) · plan 48행 "화면 한글 문구 719개" 는 고친 lexer 로 737(eval "상한 뒤 사용자 결정").
- masterPlan 환류 범위 — plan 은 S4 · D26 · D35 였고, 같은 승인 문구가 걸린 D13("시작") · D38("에이전트 대화")도 고쳤다. 개발 문서의 내부 이름("사람 글" ·
  "글마다")과 명세 이름(D29 · S9 "인바운드 알림")은 T7 ⓐ 대로 그대로. E2 가 찾는 1o 말풍선 층 값(14% · 7%)은 이 Task 가 바꾼 값이라 S4 는 새 값으로
  고치고, 옛 값은 개정 기록 `#1p` 줄에 "→" 로 남겼다.

## 산출

- 문서 — 루트 `CLAUDE.md`(필독 셋 + 필요할 때 여는 문서 표) · ddd-workflow 핵심 · doc-conventions(문서 체계 · 루트 3문서 계약 · 원장 계약) · `PROJECT.md` ·
  `PRODUCT.md`(도메인 규칙 = 한 줄 + 정본 절 · 웹UI 글꼴) · 개발자 가이드(흐름 블록 · `/fonts/`) · **web-ui.md**(새 — 화면 구조 · 디자인 토큰 · 문구 규칙 ·
  용어 표) · manual/README · an_core(`db::db_err`) · an_app(`/fonts/{파일}` · 화면 인용) · 사용자 가이드 · README · `tools/check_ddd_docs.py` 문구.
- 코드 — `notify::notice_for` 삭제 · `an_core::db::db_err` 한 곳 · `an-agent` `serde` 의존 삭제 · `server.rs` `FONTS` · `GET /fonts/{file}`(목록 이름만 토큰
  없이 · 1년 캐시) · `web/fonts/`(woff2 3 · `LICENSE.txt`).
- 화면 — `app.css` 첫 `:root` = 시안 A 토큰 92(`--fw-*` · `--lh-*` · `--sp-1 ~ 7` · `--radius-bubble` · `--radius-pill` · `--on-danger` · `--bubble-*-alpha` 포함),
  리터럴 0 · 말풍선 옆 띠 · 선택 안쪽 띠 · 점선 상자 · 숫자 테두리 상자 · 절 제목 자간 · 글자 기호 아이콘 없음 · 고정폭 8 선택자 · `.stat` · 입력창 안내
  `<kbd>` · index.html · app.js · view.js 문구 193행(`peerGroupText(peer, group, on)` — 대화 수 인자 지움).
- 시험 — `tests/css_tokens.rs`(새 — 리터럴 0 · 변수 선언 · 판정 규칙 회귀) · `web_auth` `fonts_public_and_cached` · `ux_literals.rs` 표(24 → 5 시험 ·
  `#1p 문구` 없음 41행) · `view.test.mjs` 새 문구.
- 측정 — `design/` 의 `gates.py`(size · moved · code · ux · mono · copy · terms · design · contrast · extra + `tokenizer_selftest` · `pin_selftest`) · `mock.py` ·
  `shots.mjs` · `tokens.json` · `variants/`(시안 A · B · C) — 다음 Task 가 다시 쓴다.
- 캡처 — `docs/images` home · workspace · group · pairing · agents 새로(가상 데이터), 참조 없는 4장 지움.
- masterPlan D13 · D26 · D35 · D38 · S4 환류 · 원장 3행(`#1p R1` 닫힘 · `R2` 수용 · `E8` 조건 대기). `PRODUCT.md` 모듈 표 · layer 표는 바뀐 것 없음.

## 남은 것

- 19 인수 TC — 실제 앱(릴리스 빌드) 다크 · 라이트 주요 화면 눈 비교(원장 `#1p E8`).
- 화면 문구 추출의 휴리스틱 한계(원장 `#1p R2` — 수용).
- 에이전트 화면 "새 대화" 작은 단추가 음절에서 꺾이던 원인(Stage 3 여백 변경으로 드러난 것으로 보임 — 확인 필요, `white-space: nowrap` 으로 막음).
- main 합치기 · 릴리스(실행 파일 +1.25 MiB — 버전 판단) · GitHub 반영은 사용자 확인 뒤.
