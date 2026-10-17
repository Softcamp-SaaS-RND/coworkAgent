# stage3 — 시안 A 먹선 · 토큰 · AI 티 규칙 · 글꼴 · 재발 막는 시험

> 근거: [plan](../plan.md) Stage 3(해석 10 · 11 · 12 · 18 ~ 22) · 성공 기준 A4 · A5 · A11. 시안 승인 2026-10-17(사용자 "시안은 A안으로 채택. 나머지는
> 권장 방안 수용").

## 목표

시안 A(먹선) 토큰을 `app.css` 첫 `:root` 에 넣고, 그 밖 리터럴을 0 으로, 고정폭 글꼴을 허용 목록으로, 시안의 AI 티 규칙(말풍선 옆 띠 · 선택 안쪽 띠 ·
점선 상자 · 숫자 테두리 상자 · 절 제목 자간 · 단추의 글자 기호 아이콘)을 0 으로 만들고 IBM Plex Sans KR 을 실행 파일에 넣는다. 다시 들이면 시험이 막는다.
대상 gate: E4 · E5 · E10 · E11 · E13 · E14.

## 산출 파일

- [design/tokens.json](../design/tokens.json) = `variants/a.tokens.json` + `--agent-tile` · `--backdrop`(손으로 찾은 리터럴 두 곳 — 에이전트 줄 타일
  `#0A7562` · 대화 상자 뒤 `rgb(0 0 0 / 0.35)`) → 토큰 92 · AI 티 규칙 7.
- `crates/app/web/app.css`
  - 첫 `:root` = 시안 A 값 + 해석 10 · 12 · 19 · 22 토큰(`--fw-*` · `--lh-*` · `--sp-1 ~ 7` · `--radius-bubble` · `--radius-pill` · `--on-danger` ·
    `--bubble-*-alpha` · `--agent-tile` · `--backdrop`), 다크 대입에 `--on-danger`, `@font-face` 3.
  - 리터럴 296곳 → 토큰(스크립트 — 간격 · 둥글기 · 굵기 · 줄높이 · 글꼴 크기 표), 남은 7곳은 손으로(검색 입력 여백 · 배지 줄높이 · 대화 상자 뒤 2 ·
    말풍선 오른쪽 여백 · 관리 줄 여백 · 에이전트 타일).
  - AI 티: 말풍선 옆 띠 삭제 · 층 불투명도 = 토큰 · 선택 안쪽 띠 → 바탕색 · 점선 → 실선(레일 + · 그룹 + · 컨텍스트 알약 — 끌어 놓기 점선은 기능이라
    그대로) · 숫자 테두리 상자 삭제 · 절 제목 자간 삭제 · `.ico-more`(새 선 아이콘).
  - mono 7 선택자(`.side-list .sub` · `.agent-name` · `.chip` · `.msg-meta .run` · `.steps .meta` · `#invite` · `.agent-chip`) → 본문 글꼴(숫자는
    `tabular-nums`). `.stat`(한글이 든 숫자 표시). 연결 끊김 띠 글자 `--on-danger`.
- `index.html` — ✎ ⋯ 단추 4 → `ico-pencil` · `ico-more`, 입력창 안내 `<kbd>Enter</kbd> 보내기 · <kbd>Shift+Enter</kbd> 줄바꿈`(`mono` 클래스 뺌).
- `app.js` — 행 ⋯ 단추 → `ico("more")`, 개요 요약 "안 읽음 n" · "대기 요청 n" `.count` → `.stat`.
- `crates/app/web/fonts/` — `IBMPlexSansKR-{Regular,Medium,SemiBold}.woff2`(IBM 배포 그대로, sha256 = npm 레지스트리 `@ibm/plex-sans-kr@1.1.0` 값) ·
  `LICENSE.txt`(SIL OFL 1.1 원문).
- `crates/app/src/server.rs` — `FONTS`(`include_bytes`) · `GET /fonts/{file}`(`font/woff2` · `Cache-Control: public, max-age=31536000, immutable`) ·
  `is_public`(목록 이름만 토큰 없이).
- 시험 — `tests/css_tokens.rs`(새 · 시험 3: 리터럴 0 + 검사 선언 300 이상 · `var(--…)` 이름 선언됨 · 판정 규칙 자체), `tests/web_auth.rs`
  `fonts_public_and_cached`, `tests/ux_literals.rs`(잠금 6행 갱신 · 없음 3행 더함 · `bubble_contrast` 가 토큰 불투명도를 읽음 · 표 3개 `#[rustfmt::skip]`).
- [design/gates.py](../design/gates.py) `contrast`(E14) · [design/mock.py](../design/mock.py)(1o mock8 을 리포로 + `/fonts/` · `/favicon.ico`) ·
  [design/shots.mjs](../design/shots.mjs)(눈 확인 세트 · 문서 캡처).
- [web-ui.md](../../../manual/dev/web-ui.md) `## 디자인 토큰`(옛 `## 색 · 글꼴 · 크기` 를 바꿈) · 말풍선 문장. plan 해석 18 ~ 22 · E14 · intent A11 · 시안 선택.

## 설계 결정

- **간격 = 가까운 4pt 토큰, 같은 거리면 큰 쪽**(2 → 4 · 5 → 4 · 6 → 8 · 10 → 12 · 14 → 16 · 18 → 16 · 20 → 24 · 30 → 32). 토큰에 없는 큰 값은 `calc` 로
  토큰을 더한다(64 = sp-7 + sp-5 · 72 = sp-7 + sp-6 · 48 = sp-7 + sp-2) — 토큰을 늘리지 않는다. 여백이 조금 넓어진다(시안 A 의 차분한 방향과 같음 ·
  mock 으로 확인).
- **둥글기** 2 · 3 · 4 → sm(4) · 6 · 7 → radius(6) · 8 · 10 → lg(8) · 9 · 12 · 18(알약) → pill · 말풍선 16 → bubble(12) · 꼬리 4 → sm. **굵기** 600 · 700 →
  bold(600 — 시안 값, 사용자 승인) · 500 → medium · 400 → regular. **줄높이** 1.45 이상 → body(1.6) · 나머지 → tight(1.3) · 배지 18px → `1` + flex 가운데.
- **글꼴은 IBM 배포 woff2 그대로**(해석 20) — 부분 집합은 OFL 개작이라 예약 이름 "Plex" 를 못 쓴다. 실행 파일 +1,317,296 B(woff2 1,312,840 + 라이선스
  4,456) = 지금 9,386,496 B 의 +14%. 시안 때 적은 "+1.09 MiB(부분 집합)" 보다 0.16 MiB 크다 — 편차, 위 이유.
- **공개 글꼴 경로는 이름 목록과 같을 때만**(`is_public`) — `/fonts/` 접두 전체를 열지 않는다. 목록 밖 이름은 토큰이 있어야 닿는다(시험).
- **"안 읽음 n" 처럼 한글이 든 숫자는 `.stat`** — mock 캡처에서 `.count`(고정폭)의 대체 글꼴로 한글 자간이 벌어진 것을 잡았다. 잠금 문구
  (`대기 요청 ${g.waiting}`)는 그대로.
- **다크 연결 끊김 띠** — 밝은 위험색 위 흰 글자 2.78:1 결함을 `--on-danger`(다크 `#170C0B`)로 고침.
- **`ux_literals` 표는 `#[rustfmt::skip]`** — 이 파일은 Stage 2 전까지 rustfmt 기준이었고, rustfmt 가 튜플을 여러 줄로 펴면 표가 읽히지 않는다. Stage 2
  에서 어긋난 것을 여기서 맞춤(다른 시험 · `server.rs` 도 rustfmt 기준 유지).
- **`css_tokens` 는 정규식 크레이트 없이 손 파서** — 의존을 늘리지 않는다. `every_var_is_declared` 가 토큰 철자 틀림도 막는다(`--side-w` 는 app.js 가 넣음).

## 테스트 결과

```text
cargo test -p coworkagent-app --test css_tokens        ← RED(바꾸기 전 CSS)
첫 :root 밖 리터럴 296 곳: … test result: FAILED. 2 passed; 1 failed
cargo test -p coworkagent-app --test css_tokens        ← GREEN(검사한 선언 948 — 색 435 · 간격 284 · 글꼴 크기 95 · 둥글기 78 · 굵기 37 · 줄높이 10 · 글꼴 9)
test result: ok. 3 passed; 0 failed
python docs/tasks/1p.project-tidy/design/gates.py design      ← 바꾸기 전 tokens=90 mismatch=79 slop_rules=7 slop=7
tokens=92 mismatch=0 slop_rules=7 slop=0
python docs/tasks/1p.project-tidy/design/gates.py mono
mono=.count|.fp|.kbd|.md code|.mono|.raw|.steps .at|kbd end
python docs/tasks/1p.project-tidy/design/gates.py contrast
pairs=122 fail=0
```

- mock 눈 확인(가상 데이터 · Edge headless · 다크 · 라이트 — 개요 · 대화 · 그룹 · 에이전트 · 설정): 옆 띠 없음 · 실선 · 선 아이콘 · Plex 글꼴 · 키만 고정폭 확인,
  "안 읽음 n" 고정폭 결함을 잡아 고침. mock 은 gate 증거가 아니다(해석 16).

## 다음 단계

- ▶ 다음: tasks/1p.project-tidy/work/stage4.md — plan-copy 182행 · 용어 표 · 가이드 · README · 캡처.
