# stage4 — 승인 문구 반영 · 용어 표 · 가이드 · README · 캡처

> 근거: [plan](../plan.md) Stage 4(해석 11 · 15 · 21) · [plan-copy](../plan-copy.md)(182행 · T1 ~ T7 승인 2026-10-17 + Y 11행 추가 승인) · 성공 기준
> A6 · A7 · A9 · A10. 진행 지시 2026-10-17(사용자 "의견 수용. 권장 방안으로 진행" — Stage 4 뒤 평가 직전 멈춤), 멈춘 자리의 결정 2026-10-17(사용자
> "권장 방안으로 진행" — 표 밖 발견을 Y 행으로 더함 · 표 밖 손질 4건 받아들임).

## 목표

plan-copy 승인 행을 화면 원본에 넣고(옛 문구 0 · 새 문구 있음), 잠근 시험을 새 문구로 옮기고, web-ui.md 에 문구 규칙 · 용어 표를 두고, 사용자
가이드 · README · 개발 문서의 화면 문구 인용을 화면과 같게 하고, 문서 캡처를 새로 찍는다. 대상 gate: E6 · E7 · E9 · E10 · E11 · E12(E8 은 평가 때).

## 산출 파일

- `crates/app/web/index.html` · `app.js` · `view.js` — 182행(스크립트 159 = 문구 전체가 한 literal 인 곳 · 손 23 = `raw:` · `(없음)` · `(지움)`).
  `peerGroupText(peer, group, on)` — G16 · G17 로 안내에서 대화 수가 빠져 `n` 인자와 호출부 계산을 지움.
- Y 11행(Stage 4 발견 · 추가 승인) — view.js 컨텍스트 메뉴 "압축" · "비우기", 컨텍스트 구분선 `${name} · …` 5꼴, "요약 실행 실패 · 세션은 그대로",
  도구 안내 "… 항상 사용: Read · …", 연결 끊김 안내 두 문장.
- 화면 손질(표 밖 — 캡처에서 찾음 · 사용자 승인, 아래 설계 결정): `app.css` `.rail-btn.group-add` `word-break: keep-all` · `button.small`
  `white-space: nowrap` · `.stat` `font-weight: var(--fw-regular)`, 세부정보 "이 PC 실행" 절 제목 + `#detail_runs_n`(`.stat`).
- `crates/app/tests/ux_literals.rs` — PRESENT 38행(새 문구 36 — Y 의 메뉴 2 포함 · 겹쳐서 지움 2 — "북마크 항목" → 있는 "북마크" 행, "실행 상세" →
  있는 "세부정보" 행) · ABSENT 41행 더함(`#1p 문구` 34 · `#1p 문구 Y` 7 — 바꾼 문구의 옛 꼴) · #1j ABSENT 2행 좁힘 · 개수 규칙을 새 말로("받은
  메시지" = "주고받은 메시지" 안의 것만).
- `crates/app/web/view.test.mjs` — 실패 26 시험의 기대 문구 · 시험 이름의 화면 용어(T2 ~ T7) · `context_note`(Y).
- [web-ui.md](../../../manual/dev/web-ui.md) `## 문구 규칙`(R1 ~ R7 · 잠금) · `## 용어 표`(개념 7 · 쓰지 않는 말 16) · 화면 구조 절의 화면 용어 · 말풍선 층
  값(0.1 · 0).
- [사용자 가이드](../../../manual/user/user-guide.md) — 정확한 바꾸기 40곳(인용 화면 문구 · 말풍선 설명 · 낡은 사실 · Y 의 메뉴 · 연결 끊김) · 용어
  (T1 ~ T5) · T7("글" → "메시지", 조사 맞춤 · 머리글 · 한글 · 글자는 그대로) · "로그인할 때 자동 실행" · 페어링 카드 '?' 안내.
- `README.md`(목적 4문구 그대로) · [an_app](../../../manual/dev/an_app.md)(화면 인용 · 말풍선 문단) · [개발자 가이드](../../../manual/dev/developer-guide.md) 2곳 ·
  [an_core](../../../manual/dev/an_core.md) 1곳.
- `docs/images/` — home · workspace · group(창 전체 2880×1800) · pairing · agents(설정 카드만 — README 표 "연결 · 에이전트 등록" 이 옛 캡처와 같은
  틀) 새로, 참조 없는 4장(activity · new-conversation · relay · search) 지움.
- [design/gates.py](../design/gates.py) — JS 문자열 경계가 정규식 literal 에서 어긋나던 결함 고침(`REGEX_BEFORE` · `regex_end`).
  [design/shots.mjs](../design/shots.mjs) — 문서 캡처의 연결 · 에이전트 등록을 카드 자르기로.

## 설계 결정

- **gates.py 토크나이저 결함** — view.js 의 markdown 정규식 3개(`` /`([^`]+)`|…/gu `` · `` /^\s*```/ `` 2)에 든 백틱이 문자열 경계를 어긋내, 표를
  쓸 때(`0c5d81e`) view.js 화면 문구 17개가 목록에서 빠졌다. 정규식 literal 을 건너뛰게 고친 뒤에도 `copy` 182/0(빠졌던 것은 표 행과 겹치지 않음).
  빠졌던 문구 중 규칙 위반 10개는 승인 행만 바꾼다는 규칙대로 멈춰 묻고, 사용자 승인으로 plan-copy **Y1 ~ Y11** 로 더해 반영 — Y5 는 같은
  구분선 묶음(`${name} — …`)의 오류 줄로, 한글이 없는 literal 이라 목록에 없었지만 묶음을 맞추려 함께(보고에 따로 밝힘).
  편차: plan 48행의 "화면 한글 문구 719개(… view.js 174)" 는 결함 있는 토크나이저로 센 값이다 — 평가 때 고친 `gates.py` 로 다시 센다.
- **컨텍스트 메뉴 이름**(Y1 · Y2) — "압축" · "비우기" 만 남겨도 설명은 고른 뒤 확인 상자("대화를 요약해 새 세션으로 이어 갑니다. 답하는 PC 의 Claude
  사용량을 씁니다." · "새 세션으로 시작합니다. 에이전트는 앞 대화를 기억하지 않습니다.")에 있다(R3 · 사용량 보안 사실 그대로).
- **#1j 없음 2행 좁힘** — `"그룹 없음"`(1j 가 지운 그룹 칸 placeholder) · `"읽기 전용"`(1j 가 지운 내 정보 · 연결 상세 태그)을 통째로 막던 행이,
  승인 G12("그룹 없음" — 빈 그룹 목록) · C11("읽기 전용" — 공유한 대화 상태 줄)과 부딪혔다. 1j 가 지운 꼴(`placeholder: "그룹 없음"` ·
  `class: "ro", text: "읽기 전용"`)로 좁혀 1j 의도(그룹 칸 · 읽기 전용 태그 없음)는 그대로 잠근다 — 같은 기능의 다른 4행(`id="peer_group"` 등)도 남음.
- **"이 PC 실행" 절 제목** — P2 의 이유 칸이 "'이 PC 실행' 은 절 제목" 인데 app.js 는 요약 줄을 절 제목 자리에 썼다 → 문구만 바꾸면 절 이름이
  사라져(캡처에서 확인) 제목을 남기고 요약은 옆 `.stat`(다른 절의 "요청 n" · "북마크 n" 과 같은 꼴). HTML 구조(`id` 옮김)는 승인 문자열 밖 — 아래
  CSS 3건과 함께 **표 밖 손질 4건, 사용자 승인 2026-10-17**.
- **"+ 그룹"(R5) 줄바꿈** — 40px 레일 단추에서 "+ 그 / 룹" 으로 낱말 중간이 끊겼다 → `word-break: keep-all`("+ / 그룹").
- **작은 단추 줄바꿈** — 에이전트 화면 "새 대화" 가 "새 대 / 화" 로 끊겼다(줄의 `a` 가 `flex: 1 1 auto` 라 단추가 몇 px 줄면 한글이 음절에서 꺾임,
  Stage 3 의 여백 10 → 12px(`--sp-3`)로 드러난 것으로 보임 — Stage 3 전 화면에서는 확인 안 함, 확인 필요) → `button.small` `white-space: nowrap`(높이
  28px 고정이라 줄바꿈은 늘 깨진 모양).
- **`.stat` 굵기** — 절 제목(`h3` bold) 안에서 상속돼 제목보다 도드라졌다 → `.count` 처럼 regular.
- **T7 범위** — 화면 · 사용자 가이드 · README 는 "메시지", 개발 문서 · 코드 이름(`사람 글` · 주석의 "실행 상세" 같은 내부 이름)은 그대로(T7 ⓐ). 화면
  라벨을 따옴표로 인용한 주석 1곳(app.js "북마크 항목")만 고침. 용어 표의 쓰지 않는 말은 "글" 하나가 아니라 묶음 말(부분 문자열 검사라 "글꼴" ·
  "글자" 를 막지 않게).
- **CLI · 트레이 · 서버 문구는 그대로** — 표가 웹 원본만 다뤘다. 가이드의 CLI 진행 줄("보내는 중 — 상대에 아직 …") · 알림("요청 상태 — …") ·
  상대 반영 실패 사유("편집 — …", `chat.rs`) 인용은 그대로 두고, 개발자 가이드는 화면 · CLI 문구를 갈라 적음.
- **문서 낡은 사실** — 사용자 가이드 17장 "메시지 수정 · 삭제는 없다" 가 7.6 편집 모드(#1f)와 어긋나 "상대 쪽 사본은 편집 모드의 '상대 창에도'
  로만" 으로 고침. an_app.md 말풍선 문단(불투명도 0.12 · 3px 띠)이 Stage 3 뒤 낡아 토큰 층 · 띠 없음으로 고침(Stage 3 누락).
- **조사** — 받침이 바뀌는 바꾸기(대화방 → 대화 · 북마크 항목 → 북마크 · 글 → 메시지)는 바로 뒤 조사(은 · 을 · 이 · 과 · 으로)를 함께 바꾸고, 굵게
  표시 뒤 조사 1곳은 손으로(7.2 "…답)는 오른쪽, …메시지**는").

## 테스트 결과

```text
python docs/tasks/1p.project-tidy/design/gates.py copy          ← 손 행 전  rows=182 applied=159 stale=23 doc_stale=36
rows=182 applied=182 stale=0 doc_stale=0                          ← Y 11행 더한 뒤
rows=193 applied=193 stale=0 doc_stale=0
python docs/tasks/1p.project-tidy/design/gates.py terms
terms=7 banned=16 banned_hits=0
node --test --test-reporter=tap "crates/app/web/*.test.mjs"      ← 기대 문구 고치기 전 pass 86 fail 26
# tests 112  # pass 112  # fail 0
cargo test -p coworkagent-app --test ux_literals                ← 고치기 전 literals_present 34 없음 · literals_absent 2 있음
test result: ok. 5 passed; 0 failed
cargo test --workspace                                          ← 75 묶음 합계
passed=418 failed=0 ignored=3
cargo clippy --workspace --all-targets -- -D warnings
Finished `dev` profile … (경고 0)
python docs/tasks/1p.project-tidy/design/gates.py design · mono · contrast · code · size · moved ceaaa6f
tokens=92 mismatch=0 slop_rules=7 slop=0 · mono=.count|.fp|.kbd|.md code|.mono|.raw|.steps .at|kbd end · pairs=122 fail=0 ·
notice_for=0 db_err_defs=1 agent_serde=False ux_tests=5 end · files=4 bytes=19092 under20k=True must_read_ok=True product=15066 under15k=True ·
targets=167 missing=0
python docs/tasks/1p.project-tidy/design/gates.py extra ceaaa6f
t12=False wt=True state_stale=0 shots=5 stale_shots=0 orphans=0      ← wt = 빈 워크트리 폴더(다른 프로세스가 쥠, rmdir "Device or resource busy")
python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_crates.py
PASS — DDD 문서 규약 위반 없음 · readme ok · crates=4 edges=6 violations=0
rustfmt --edition 2024 --check crates/app/tests/ux_literals.rs  → 차이 없음
```

- 캡처 눈 확인(mock · 다크 · 문서 5장 + 눈 확인 세트): 새 문구 · "+ / 그룹" · 절 제목 "이 PC 실행" · "새 대화" 한 줄 확인. mock 은 gate 증거가 아니다(해석 16).
  Y 행은 컨텍스트 메뉴 · 구분선 · 연결 끊김 띠 · 에이전트 등록 칸 안내라 문서 캡처 5장 화면 밖(agents.png 의 도구 안내 한 줄만 — 다시 찍음).

## 다음 단계

- 결정 반영 끝(Y 11행 · 표 밖 손질 4건). 남은 블로커: 빈 워크트리 폴더(E9 `wt`) — 종결 전 다시 지움.
- ▶ 다음: 평가 — `check_gates` · E8 ABANDON(D28) · codex 적대적 리뷰(최대 2 라운드). 종결 때 masterPlan 반영(61행 "시작 — 자동 실행" · 161행 #1o
  "agent 대화" 원문 → T3 "에이전트 대화").
