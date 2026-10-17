# plan — 1p.project-tidy

> 근거: [intent](intent.md)(승인 2026-10-17 — 미해소 1 ~ 9 권장안). **구현자는 이 문서와 부속 [plan-docs](plan-docs.md)(Stage 1) ·
> [plan-copy](plan-copy.md)(Stage 4)만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md). 측정 도구는 [design/gates.py](design/gates.py).
> 기준 커밋: `ceaaa6f`(main — 문서 · 코드 측정 기준). plan 승인: 2026-10-17(사용자 "의견 수용하고 진행해" — 문구 표 전부 · T7 ⓐ ·
> 해석 12 (나) · 해석 7 폐기 2건 · plan-docs 9 유지 · 진행 방식 (b) Stage 1 ~ 4 연속). 개정 1(디자인 병행) · 개정 2(EXPECT 모집단 정확값) 승인 2026-10-17 —
> 아래 `## 개정 기록`.

## 목표

세션마다 읽는 문서를 줄이고 세부는 필요할 때 여는 문서로 옮긴다. 동작은 그대로 두고 죽은 코드 · 중복 · 미사용 의존 · 흩어진 문구 시험을 정리한다.
화면은 Nocturne 토큰을 강제해 글꼴 · 크기 · 굵기 · 간격 · 둥글기를 맞추고, 메뉴 이름 · 설명을 승인된 문구 표대로 고친다. 재발은 시험이 막는다.

### 성공 기준(intent 사본)

- A1: 매 세션 필독 문서(루트 `CLAUDE.md` · `STATE.md` · `PROJECT.md` · ddd-workflow) 합이 20KB 이하이고, `CLAUDE.md` 필독 목록이 정확히 `STATE.md` ·
  `PROJECT.md` · ddd-workflow 다. 나머지는 "필요할 때 여는 문서" 표에서 링크로 닿는다(현 필독 체인 60.1KB · 이 4문서 27.2KB).
- A2: 루트 `PRODUCT.md` 가 15KB 이하(현 31.7KB)이고, 옮긴 사실(코드 이름 · 단위 붙은 수)은 살아 있는 문서에 남는다.
- A3: 동작 변경 없는 코드 정리 — `notice_for` 삭제 · `db_err` 한 곳 · `an-agent` 의 `serde` 의존 빼기 · `ux_literals.rs` 를 표로(시험 6개 이하, 잠근 문구 수 보존).
- A4: `app.css` 의 색 · 글꼴 크기 · 굵기 · 줄높이 · 둥글기 · 간격 리터럴이 첫 `:root` 밖에 0 — 시험이 막는다(다시 들이면 red).
- A5: `var(--mono)` 는 허용 목록(코드 · 지문 · 키 · 원문 보기 · 숫자)에만 — 한글이 들어가는 칩 · 부제 · 안내 · 에이전트 이름은 본문 글꼴.
- A6: [plan-copy](plan-copy.md) 의 승인된 행이 화면에 글자 그대로 반영되고, 사용자 가이드 · README 에 옛 문구가 남지 않는다. 표 밖 문구는 그대로다.
- A7: 한 개념 한 이름 — web-ui.md 용어 표의 "쓰지 않는 말" 이 화면 문구에 없다.
- A8: 화면 회귀 없음 — node 화면 시험 · 워크스페이스 시험 green + 다크 · 라이트 주요 화면 눈 비교(수동 — D28 로 19 인수).
- A9: `target-t12/` · 빈 워크트리 폴더 삭제, `STATE.md` 낡은 항목을 제자리로(원장 · 폐기 사유), README · 사용자 가이드 캡처를 새 화면으로(참조 없는 캡처 0),
  UI 용어 표(web-ui.md).
- A10: 검증 명령(`cargo test --workspace` · clippy · node 화면 시험 · `check_ddd_docs` · `check_readme` · `check_crates`) 통과.
- A11: (개정) 화면이 Claude Design 캔버스에서 고른 시안(A 먹선)대로다 — 시안 토큰(`design/tokens.json`)이 `app.css` 첫 `:root` 와 같고, 시안이
  정한 AI 티 규칙에 걸리는 곳이 0 이며, 글자/바탕 대비 4.5:1 미달이 0 이다.

### intent 결정(사본)

1. 한 Task, stage = 문서 → 코드 · 시험 → 화면 토큰 → 문구 · 동기화. 2. "자동 로드" = 리포 안 필독 체인(하네스 · 플러그인 제외). 3. 뼈대
(`CLAUDE.md` · `docs/etc/`) 개정 허용 — 프로젝트 무관성 유지 · 같은 커밋에 체커 개정. 4. 필독 ≤ 20KB · PRODUCT ≤ 15KB · masterPlan 은 Task 개설 · 종결
때만(D 본문 그대로). 5. 이전 결정 literal(원문 포함)도 고칠 수 있다 — 전 · 후 · 이유 표를 행별 승인. 6. 문구 규칙 R1 ~ R7([plan-copy](plan-copy.md)).
7. Nocturne 강제 · 새 시안 없음 · mono 는 코드 · 숫자만. 8. 동작 변경 0 · 큰 파일 쪼개기 제외 · `cfg(unix)` 불가침. 9. 추가 ⓐ ~ ⓓ 전부.

## 사실(실측, 기준 `ceaaa6f`)

- **문서**: 필독 4문서 27,173 바이트(LF) · `PRODUCT.md` 31,673 — `## 도메인 규칙` 36 블록 26.1KB(화면 9 · 흐름 14 · 불변 규칙 13). ddd-workflow 의
  문서 체계 · PROJECT/PRODUCT 계약 · 원장 계약 = 5.4KB. 표 · 처리는 [plan-docs](plan-docs.md).
- **코드**: `pub fn notice_for`(`app/src/notify.rs` 439) 참조 0 — 호출부는 모두 `notice_with`. `fn db_err` 는 같은 세 줄(`io::Error::other`)이
  `agent/src/chat.rs` 202 · `app/src/notify.rs` 125 · `net/src/node.rs` 286 세 곳, 호출 268곳. `an_core::db::Error` = `rusqlite::Error` 재수출(남의
  타입이라 `From` 구현 불가). `an-agent` 는 `serde_json` 만 쓰고 `serde`(derive) 참조 0. `ux_literals.rs` = `#[test]` 24개(문구 literal 22 · 계산 2 —
  초대 글 · 말풍선 대비).
- **화면**: `gates.py mono` = `#invite|.agent-chip|.agent-name|.chip|.count|.fp|.kbd|.md code|.mono|.msg-meta .run|.raw|.side-list .sub|.steps .at|.steps .meta|kbd`.
  입력창 안내는 `<span class="hint mono kbd-hint">Enter 보내기 · Shift+Enter 줄바꿈</span>`(한글이 mono). `:root` 밖 리터럴 — 글꼴 크기 10 · 11px,
  색 `#e5484d` · `#0A7562` · `rgb(0 0 0 / 0.35)` ×2, 굵기 600 ×6, 줄높이 8가지, 둥글기 9값 + 말풍선 조합 2, gap 10가지, padding 조합 40가지.
- **문구**: 화면 한글 문구 719개(index.html 213 · app.js 332 · view.js 174). 바꿀 후보 182행 + 용어 결정 7 — `gates.py copy before` = `rows=182 applied=182 stale=0 doc_stale=36`
  (현재 문구가 모두 실재 · 사용자 가이드 · README 가 그중 36행의 옛 문구를 인용 — Stage 4 에서 함께 고친다).
- **로컬 · 캡처**: `target-t12/` 9.0GB · 빈 `.claude/worktrees/agent-abd1086a96c70114e/` — `.cargo/config.toml` · `tools/` 에 참조 없음. README · 사용자
  가이드 캡처 5장(agents · pairing · workspace · group · home), 어디서도 참조 없는 캡처 4장(activity · new-conversation · relay · search).
- **기준선**(`gates.py`, 지금 — 전부 미충족이어야 맞다): `size` → `bytes=27173 under20k=False must_read_ok=False product=31673 under15k=False` ·
  `code` → `notice_for=1 db_err_defs=3 agent_serde=True ux_tests=24` · `extra` → `t12=True wt=True state_stale=5 shots=5 stale_shots=5 orphans=4`.

## 해석(plan 승인 대상)

1. **필독 4문서** = 루트 `CLAUDE.md` · `STATE.md` · `PROJECT.md` · ddd-workflow. 크기는 git 의 LF 바이트 합(CR 제외). `CLAUDE.md` `## 매 세션 필독` 절이 고르는
   문서 집합을 E1 이 본다.
2. **ddd-workflow 쪼개기** — 문서 체계 · PROJECT/PRODUCT 계약 · 원장 계약은 doc-conventions 로(문서를 만들거나 루트 문서를 고칠 때 연다). `STATE.md — 필수 2절`
   은 단계 전환마다 쓰므로 핵심에 남긴다. 절 이름을 거는 바깥 참조 · `check_ddd_docs.py` 문구는 같은 커밋에서 고친다.
3. **PRODUCT 도메인 규칙** = 지켜야 하는 규칙 한 줄 + 정본 절 링크. 흐름은 개발자 가이드 절, 화면은 web-ui.md. **옮김 = 대상에 없는 사실만 옮기고
   지운다** — E2 가 지운 줄의 코드 이름 · 단위 붙은 수를 살아 있는 문서에서 찾는다.
4. **web-ui.md**(`docs/manual/dev/`) — 새 lazy 문서(화면 구조 · 알림 · 디자인 토큰 · 문구 규칙 · 용어 표), 모듈 매뉴얼 형식의 예외(서두 명시) ·
   manual/README 등록.
5. **masterPlan** 은 Stage 1 에서 손대지 않는다(D 본문 = 정본 계약). 종결 환류에서 S4 · D26 · D35 의 바뀐 문구 · 1p 줄만 고친다.
6. **보관 · 삭제 후보**(bootstrap-interview · security-review · ui-mockups · feedback) = 유지 권장 — 필독이 아니고 archive 가 링크로 건다(plan-docs 9).
7. **STATE 낡은 항목** — 지우지 않고 제자리로: 원장에 있는 것은 STATE 에서만 지움, 1f 관찰은 원장 `#1f` 행, "UI 시안 차이 8건" · "미구현 코드 제안
   7건" 은 원본 목록이 없어 **폐기 + 사유**(stage1.md — 사용자 확인 대상).
8. **`db_err`** 는 `an_core::db::db_err`(pub fn) 한 곳 — 호출 268곳은 이름 그대로 두고 `use` 만 바꾼다(가장 짧은 diff).
9. **`ux_literals.rs` 표** — 있음 · 없음 · 개수 세 표를 도는 시험 + 계산 시험(초대 글 · 말풍선 대비)은 따로. 바꾸기 전후 잠근 literal 수를 stage2.md 에
   적고 같아야 한다.
10. **토큰 집합** — 굵기 `--fw-regular/medium/bold`(400 · 500 · 700), 줄높이 `--lh-body` 1.55 · `--lh-tight` 1.3, 간격 `--sp-1 ~ 7`(4 · 8 · 12 · 16 · 24 ·
    32 · 40), 둥글기 기존 `--radius-sm/--radius/--radius-lg`(6 · 8 · 10) + `--radius-pill`. 예외(시험이 허용): `0` · `auto` · `1px`(테두리) · `-1px`(시각적
    숨김) · `50%`(원) · `inherit` · `line-height: 1`(아이콘 상자). 600 → 700(제목 · 이름) 또는 500(본문 강조), 10 · 11px → `--fs-xs`, 4pt 밖 간격은 가까운 토큰.
11. **mono 허용 목록** = `.count` · `.fp` · `.kbd` · `.md code` · `.mono` · `.raw` · `.steps .at` · `kbd`. 칩 · 부제 · 에이전트 이름 · 실행 메타 · 초대 글은 본문
    글꼴(숫자는 `font-variant-numeric: tabular-nums`). 입력창 안내는 `<kbd>Enter</kbd> 보내기 · <kbd>Shift+Enter</kbd> 줄바꿈`.
12. **말풍선 둥글기**(결정 7 의 "6 · 8 · 10 + 원 · 알약" 밖) — (가) 10px 로 맞춘다 · (나) `--radius-bubble: 16px` 토큰을 더해 지금 모양을 지킨다. 권장 (나).
13. **문구 표 위치** — 라운드 1 권장안은 "이 intent 에" 였으나 표는 방법(어떻게)이라 plan 부속 [plan-copy](plan-copy.md) 에 둔다(**결정 5 편차** —
    intent 결정 5 에 적음). 승인된 행만 바꾼다.
14. **T7 글 / 메시지** — plan-copy 용어 결정 T7 은 다른 행과 따로 답한다. ⓐ 가 아니면 `[T7]` 행을 그 답대로 고친다.
15. **캡처** — 가상 데이터 mock(지어낸 인물 · PC · 대화) + `playwright-core` msedge headless(1o 방식). 이번에는 mock · 찍기 스크립트를 `{TASK}/design/` 에
    커밋한다(다음 Task 재사용). README · 사용자 가이드 5장 다시, 참조 없는 4장 지움.
16. **A8 눈 비교**는 수동 gate — D28 로 19 인수에서(ABANDON 으로 드러낸다). mock 캡처는 gate 증거가 아니다.
17. **로컬 삭제**(9ⓐ)는 Stage 1 첫 단계 — 리포 밖이라 커밋은 없고 stage1.md 에 적는다.
18. (개정 1) **디자인 시안** — Claude Design 캔버스(Artifact Design 형, 비공개)에 지금 화면과 시안 셋을 같은 가상 데이터로 나란히 그린다.
    시안마다 토큰 표(색 · 글꼴 · 크기 · 간격 · 둥글기)와 "걷어낸 AI 티" 목록을 붙인다. 사용자가 고르면 그 토큰을 `{TASK}/design/tokens.json` 에 적고
    Stage 3 이 `:root` 를 그 값으로 바꾼다(리터럴 0 · mono 규칙은 그대로). 구조(5영역 · 말풍선 · 메뉴 범위) · 기능은 바꾸지 않는다. 글꼴을 실행 파일에
    넣는 시안이면(예: Pretendard, SIL OFL) 크기 증가를 시안에 적는다.
    낸 시안(2026-10-17): 캔버스 https://claude.ai/artifact/8wiNsMseNNNiVoJ1SW15wA(비공개) — 지금 + A 먹선(라이트 우선 · 쪽빛 · IBM Plex Sans KR 포함) ·
    B 흑연(다크 우선 · 호박색 · OS 글꼴) · C 청자(고밀도 · 청자색 · Noto Sans KR 포함), 같은 화면 원본에 토큰만 갈아 끼운 다크 · 라이트. 토큰 =
    [design/variants](design/variants/)`{a,b,c}.tokens.json`(토큰 90 — 지금 이름 + 해석 10 · 12 + `--on-danger` · `--bubble-mine-alpha` ·
    `--bubble-theirs-alpha`, AI 티 규칙 7 — 지금 걸리는 곳 css 7 · html 4 · js 1). 고른 파일을 `design/tokens.json` 으로 복사한다. 세 시안 모두 글자/바탕
    짝 37 × 2 테마 · 타일 흰 글자 · 말풍선 36 경우 4.5:1 이상(지금은 미달 12 — 타일 흰 글자 10 · 다크 faint/surface-2 · 다크 연결 끊김 띠 2.78).
    **고른 시안 = A 먹선**(2026-10-17) — `design/tokens.json` = `variants/a.tokens.json`. 토큰 값은 시안 값이다(해석 10 은 이름 · 개수만 정하고, 값은
    시안 따름 — 굵게 600 · 줄높이 1.6 · 둥글기 4 · 6 · 8 · 말풍선 12). 기본 테마(다크)는 그대로.
19. (개정 1) **말풍선 규칙** — 옆 띠 3px 를 지우고(가는 선 1px 는 그대로) 층 불투명도를 `--bubble-mine-alpha` · `--bubble-theirs-alpha`
    토큰으로(`opacity: var(…)`, A = 0.1 · 0). 잠금 갱신: `ux_literals` PRESENT 4행(#1l · #1o 말풍선 규칙) · `bubble_contrast` 는 불투명도를 토큰에서
    읽는다 · web-ui.md · 사용자 가이드 · masterPlan S4(종결 환류).
20. (개정 1) **글꼴 포함** — IBM Plex Sans KR 400 · 500 · 600 을 실행 파일에 넣는다(`include_bytes` · `GET /fonts/{파일}` 토큰 없이 · `Cache-Control`
    1년 · `@font-face` `font-display: swap`). 파일은 IBM 배포 woff2 그대로(npm `@ibm/plex-sans-kr@1.1.0` `fonts/complete/woff2/hinted`, 완성형 11,172자)
    — 부분 집합으로 줄이면 OFL 개작이라 예약 이름 "Plex" 를 쓸 수 없다(시안 때 적은 1.09 MiB 대신 1,312,840 B · 1.25 MiB). 라이선스 원문을
    `web/fonts/LICENSE.txt` 로 함께 넣고 `/fonts/LICENSE.txt` 로 연다. `css_tokens` 예외 = `@font-face` 블록.
21. (개정 1) **글자 기호 아이콘** — 단추의 ✎ ⋯ 를 선 아이콘(`ico` 한 벌)으로. 문구 속 "⋯ 메뉴" 는 Stage 4 문구 표 행과 함께.
22. (개정 1) **대비 gate** — `gates.py contrast`: 첫 `:root` 토큰으로 글자/바탕 짝 37 × 2 테마 · 타일 흰 글자 6 × 2(상대 5색 · 에이전트 타일) ·
    말풍선 18 × 2 = 122 를 계산해 미달 0.
    다크 연결 끊김 띠는 `--on-danger`(새 토큰) 글자로.

## 설계

- **Stage 1 — 문서**: [plan-docs](plan-docs.md) 2 ~ 10 그대로. 순서 = 로컬 삭제 → `CLAUDE.md` → ddd-workflow 쪼개기 + doc-conventions + 바깥 참조 +
  체커 문구 → `PROJECT.md` → `PRODUCT.md` 도메인 규칙(개발자 가이드 · web-ui.md 로 옮김) → manual/README → `STATE.md` · 원장 `#1f`. 매 단계 `gates.py size` ·
  `moved ceaaa6f` · `check_ddd_docs`.
- **Stage 2 — 코드 · 시험**: `notice_for` 삭제 → `db_err` 를 `an_core::db` 로 · 세 크레이트 `use` → `an-agent` `serde` 빼고 빌드로 확인 → `ux_literals.rs` 표
  (literal 수 전후 같음). `cfg(unix)` 코드는 손대지 않는다.
- **Stage 3 — 화면 토큰**: RED 먼저 — `crates/app/tests/css_tokens.rs`(include_str app.css, 첫 `:root` 를 떼고 선언을 훑어 위반 0 + 검사한 선언 300개 이상)를
  지금 CSS 로 실패시켜 본다 → 토큰 추가(해석 10 · 12) → `:root` 밖 리터럴 바꾸기 → mono 범위(해석 11) · 입력창 안내 markup → 다크 · 라이트 mock 캡처로 눈
  확인(stage3.md). web-ui.md `## 디자인 토큰`.
- **Stage 4 — 문구 · 동기화**: plan-copy 승인 행 반영 → `ux_literals.rs`(새 문구 있음 · 옛 문구 없음) · `view.test.mjs` → web-ui.md `## 문구 규칙` ·
  `## 용어 표` → 사용자 가이드 · README(목적 4문구 그대로) · 개발자 가이드의 화면 문구 인용 → 캡처(해석 15) → `STATE.md` 의 캡처 항목 지움.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 필독 줄이기 · lazy 이관 · STATE 정리 · 로컬 캐시 삭제 | `CLAUDE.md` · ddd-workflow · doc-conventions · `PROJECT.md` · `PRODUCT.md` · 개발자 가이드 · web-ui.md(새) · manual/README · `STATE.md` · 원장 · `check_ddd_docs.py` | E1 · E2 · E12 |
| 2 | 죽은 코드 · 중복 도우미 · 미사용 의존 · 문구 시험 표 | `core/src/db.rs` · agent · app · net · `agent/Cargo.toml` · `ux_literals.rs` | E3 · E10 · E11 |
| 3 | (개정) 시안 A 토큰 · 리터럴 0 · mono 범위 · AI 티 규칙 0 · 글꼴 포함 · 재발 막는 시험 | Claude Design 캔버스 · `design/tokens.json` · `app.css` · `index.html` · `app.js`(클래스 · 아이콘) · `web/fonts/` · `src/server.rs`(글꼴 경로) · `tests/css_tokens.rs` · `ux_literals.rs` · web-ui.md | E4 · E5 · E10 · E11 · E13 · E14 |
| 4 | 승인 문구 반영 · 용어 표 · 가이드 · README · 캡처 | 웹 원본 · 시험 · 문서 · `docs/images` · `{TASK}/design/` | E6 · E7 · E8 · E9 · E10 · E11 · E12 |

## 평가 항목

- [ ] E1: A1·A2 — 필독 4문서 합 20KB 이하 · `CLAUDE.md` 필독 절이 정확히 STATE · PROJECT · ddd-workflow · `PRODUCT.md` 15KB 이하
  CHECK: python {TASK}/design/gates.py size
  EXPECT: /files=4 bytes=\d+ under20k=True must_read_ok=True product=\d+ under15k=True/
  EVIDENCE: pending
- [ ] E2: A1·A2 — 줄인 4문서(`CLAUDE.md` · `PROJECT.md` · `PRODUCT.md` · ddd-workflow)에서 지운 줄의 사실(ASCII 코드 이름 · 단위 붙은 수)이 살아 있는 문서에 모두 있다(기준 `ceaaa6f`)
  CHECK: python {TASK}/design/gates.py moved ceaaa6f
  EXPECT: targets=167 missing=0
  EVIDENCE: pending
- [ ] E3: A3 — `notice_for` 0 · `db_err` 정의 1(an_core) · `an-agent` `serde` 의존 없음 · `ux_literals.rs` 시험 6개 이하
  CHECK: python {TASK}/design/gates.py code
  EXPECT: /notice_for=0 db_err_defs=1 agent_serde=False ux_tests=[1-6] end/
  EVIDENCE: pending
- [ ] E4: A4 — `app.css` 첫 `:root` 밖 색 · 글꼴 크기 · 굵기 · 줄높이 · 둥글기 · 간격 리터럴 0(검사한 선언 300개 이상을 함께 단정)
  CHECK: cargo test -p coworkagent-app --test css_tokens
  EXPECT: /test result: ok\. [1-9]\d* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A5 — `var(--mono)` 를 쓰는 선택자가 허용 목록과 같다
  CHECK: python {TASK}/design/gates.py mono
  EXPECT: mono=.count|.fp|.kbd|.md code|.mono|.raw|.steps .at|kbd end
  EVIDENCE: pending
- [ ] E6: A6 — plan-copy 승인(✅) 행이 화면 원본에 모두 반영(현재 문구 사라짐 · 제안 문구 있음)되고 사용자 가이드 · README 에 옛 문구 0
  CHECK: python {TASK}/design/gates.py copy
  EXPECT: rows=193 applied=193 stale=0 doc_stale=0
  EVIDENCE: pending
- [ ] E7: A7·A9 — web-ui.md 용어 표가 있고(개념 1개 이상) 쓰지 않는 말이 화면 문구에 0
  CHECK: python {TASK}/design/gates.py terms
  EXPECT: terms=7 banned=16 banned_hits=0
  EVIDENCE: pending
- [ ] E8: A8 — 실제 앱(릴리스 빌드)에서 다크 · 라이트 주요 화면(개요 · 대화 · 세부정보 · 설정 · 새 대화)을 바꾸기 전 캡처와 눈으로 비교 — 글꼴 · 크기 · 간격이 어긋나거나 잘린 곳 없음(수동, D28 이면 19 인수)
  EVIDENCE: pending
- [ ] E9: A9 — `target-t12/` · 빈 워크트리 폴더 없음 · `STATE.md` 낡은 항목 0 · README · 사용자 가이드 캡처 전부 새로 · 참조 없는 캡처 0
  CHECK: python {TASK}/design/gates.py extra ceaaa6f
  EXPECT: t12=False wt=False state_stale=0 shots=5 stale_shots=0 orphans=0
  EVIDENCE: pending
- [ ] E10: A8·A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9]\d* passed; 0 failed/
  EVIDENCE: pending
- [ ] E11: A8·A10 — 우리 코드 경고 0(clippy -D warnings) · node 화면 시험 실패 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings && node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass [1-9]\d*\s+# fail 0/
  EVIDENCE: pending
- [ ] E12: A10 — 문서 규약 · README · 의존 경계 체커 통과
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_crates.py
  EXPECT: /crates=4 edges=\d+ violations=0/
  EVIDENCE: pending
- [ ] E13: A11 — 고른 시안의 토큰(`design/tokens.json`)이 `app.css` 첫 `:root` 값과 모두 같고, 시안이 정한 AI 티 패턴이 화면 원본에 0
  CHECK: python {TASK}/design/gates.py design
  EXPECT: tokens=92 mismatch=0 slop_rules=7 slop=0
  EVIDENCE: pending
- [ ] E14: A11 — 첫 `:root` 토큰으로 계산한 글자/바탕 대비(짝 37 × 2 테마 · 타일 흰 글자 6 × 2 · 말풍선 18 × 2)가 모두 4.5:1 이상
  CHECK: python {TASK}/design/gates.py contrast
  EXPECT: pairs=122 fail=0
  EVIDENCE: pending

## 종결 시 할 일

- `report.md` · 매뉴얼 갱신(an_core `db_err` · an_app 화면 함수 · web-ui.md) · [ledger](../../overview/ledger.md) 미조치 · `PRODUCT.md` 모듈 표.
- masterPlan 환류 — S4 · D26 · D35 의 바뀐 화면 문구(승인 행대로), Phase 1 오버플로 "진행: 1p" → 종결 목록, 개정 기록 `Task #1p` 한 줄.
- `STATE.md` · `git mv docs/tasks/1p.project-tidy docs/archive/1p.project-tidy` · 인바운드 링크 · `/graphify --update`.

## 개정 기록

- 개정 1(2026-10-17 · 승인 2026-10-17 사용자 "의견 수용 진행. 시안은 A안으로 채택. 나머지는 권장 방안 수용"): 디자인 병행 — A11 · 해석 18 ~ 22 ·
  Stage 3 · E13 · E14 추가. 사유: 사용자 지시 "디자인도 최신 claude design skills 적용해서 ai slop 디자인이 아니게 개선도 병행".
- 개정 2(2026-10-17 · 승인 2026-10-17 사용자 "의견 수용. 권장 방안으로 진행"): E2 · E6 · E7 · E9 · E13 · E14 의 EXPECT 를 하한 정규식(`[1-9]\d*`)에서
  승인 모집단 정확값으로 — `targets=167` · `rows=193 applied=193` · `terms=7 banned=16` · `shots=5` · `tokens=92 … slop_rules=7` · `pairs=122`. 판정 방법
  (CHECK)은 그대로, gate 안 단정(`design/gates.py` `POP` — 개수 · 내용 digest · 열쇠 유일)도 그대로. 사유: 적대적 평가 라운드 1 · 2 지적(모집단을 줄여도
  통과 — [eval](eval.md) `## 미조치 지적` R1) · eval-protocol 규약 2 "대상 개수가 계약이다".
