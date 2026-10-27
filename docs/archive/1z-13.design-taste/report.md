# report — 1z-13.design-taste

종결: 2026-10-26 · 브랜치 `task/1z-13-design-taste`(원 작업 폴더 — worktree 없음) → main(로컬 반영, push 는 사용자 확인 뒤).

## 결과

AI 디자인 느낌 제거(목적 1) — 앱 전체 시각 언어를 **Graphite**(흑연 무채색 + 차분한 청록 강조 하나)로 바꿨다. 큰 틀 · 기능 · 문구 그대로, 버전 1.3.7, DB 스키마 그대로(28).

- stage 1 — 토큰(라이트 · 다크): 주 단추 = 흑연(`--primary`), 강조 = 청록(채도 45 % 아래), 상태 색 차분하게. 공통 부품 — 글자 탭 · 테두리 없는 도구 단추 · 입력창 한 줄 ·
  사람 글 회색 면 / 에이전트 글 흰 면 + 선 · 세부정보 · 설정 타일 선 · 프로필 단추. 단추 자리 — '새로 만들기' 테두리 단추, 상세 단추 줄 오른쪽 한 묶음(P3).
- stage 2 — 사람 아이콘 아바타 → 이름 머리글자 네모(`initialOf` — 글자 묶음 `Intl.Segmenter`): 레일 · 줄 타일 · 프로필 · 말풍선 · 주고받은 메시지. 상대 · 그룹 타일 다섯 색 지움.
  에이전트 = 무채 네모 + 로봇(P1 — 사용자가 고른 아이콘 색 6 그대로).
- stage 3 — 대화 종류 = 무채 선 아이콘(결정 15 → 16: 응답 = 로봇 · 문의 = 사람 · 공유 = 눈), 종류 색 면 · 테두리 · 점선 · 머리 선 · 대화 줄 카드 지움.
- stage 4 — 실제 화면 25곳 × 라이트 · 다크 사진을 사용자가 확인(https://claude.ai/artifact/Ndz7qpArMaEjgDwf845CfT), 답 반영: 문의 = 사람 아이콘 · 요청 보드 · 의견 상자 상태 색 띠 지움 ·
  메뉴 스위치 = 설정 화면 스위치 모양. stage2-2(타일 편집 이름 칸 넘침) · stage2-3(`conv_leave` 시험 경합 — 사용자 "이 Task 에서 시험만 고침") 고침. 문서 · 1.3.7.
- 평가 — codex 라운드 1/2 지적 1(R1-1 대비 짝 빈틈 — 사실, 고침) · 라운드 2/2 지적 0. 미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 AI 티 0 | 충족 | E5 `taste files=50 checks=9 violations=0`(옆 · 위 색 띠까지 넓힘 — 개정 ⑨), E7 사진 50장 판독(그라데이션 0 · 사람 SVG 0 · 아바타 사람 아이콘 0) · 사용자 확인 |
| A2 강조색 하나 | 충족 | E4 `token_contrast` 78 경우 · E7 채도 있는 색 = 토큰만(강조 · 상태 · 위험 · 에이전트 아이콘 색), 주 단추 14/14 = `--primary` |
| A3 동작 · 문구 그대로 | 충족 | E1 · E3 · E4 통과, E6 `labels n=280 missing=0 added=0`, 바뀐 시험 행은 plan '바꾸는 시험' · 개정 ① ~ ⑪ 에 행마다 근거 |
| A4 실제 화면 | 충족 | E7 — 25곳 × 두 테마, 콘솔 오류 0, 사용자 "확인 · 평가 진행" |
| A5 회귀 없음 | 충족 | E1 · E2 · E8 · E9, 매뉴얼(web-ui · 사용자 가이드) 반영 |

- 편차 · 한계:
  - plan "화면 23" → 목록대로 25곳(개정 ⑨). 오류 판독은 심은 감시만 — 브라우저 자원 실패(404 · CSP)는 세지 않았다.
  - R1-1 고침(`d3698d7` — 라이트 `--run` #8A6A2F → #80632B · 연결 끊김 머리글자 흐림 지움) 뒤 사진은 다시 찍지 않았다(대비는 토큰 짝 시험이 덮음).
  - plan 토큰 표의 `--run` 값과 E4 제목 '70 경우' 는 승인 때 값 — 개정 ⑪ 이 바꾼 값 · 수를 적었다(시험은 늘기만 함).
  - 대화 ⋯ 메뉴의 그룹 체크 상자(`.float`) 꺼짐 테두리 대비가 낮다(본 것, 이 Task 범위 밖 — stage4-2 질문 문구에 '대화 메뉴' 를 잘못 넣었고 실제로는 프로필 메뉴만 바꿨다).
  - R2 권고(결함 아님): `--faint`/`--accent-soft`(고른 줄 시각 — 4.55 충족) 짝 추가, 상태 점(글자 아님 — 3:1 충족)은 둠.
- 구현자 해석(사용자 미확인):
  - 결정 16 이 닿는 곳 = 유형 아이콘 전부(대화 줄 · 개요 목록 · 미리보기 · 검색 · 대화 머리) — 답은 "대화목록 섹션" 이었으나 한 함수 `convKindIcon` 이고 "사람은 사람 아이콘" 원칙이라 함께.
  - '상대 대화 목록' 화면 = 상대 타일 + 공유 대화를 연 화면(사이드바 '대화' · '공유 대화').
  - 띠에만 쓰이던 상태 class(`board-col` · `board-card` · `req-feedback` 의 `req-{상태}`)를 함께 지움.
  - R1-1 고침 방식(라이트 `--run` 어둡게 · 연결 끊김은 점 색으로만).

## 사용자 결정 필요

> **2026-10-27 사용자 결정 — 종결**: 이 절의 남은 항목 · 이 Task 의 구현자 해석은 모두 종결 · 폐기한다 — 사용자 "잔여 이슈는 종결/폐기 처리. 왜냐하면 이전 task에서 결정사항이여도 현재 UI/UX 및 기능을 현재 디자인에 맞춰 개발 완료함." 원장 행이 있으면 원장에서 `수용`.

| 인덱스 | 내용 | 곳 |
|---|---|---|
| report-1 | 결정 16 을 사이드바 대화 줄에만 둘지(지금 = 유형 아이콘 전부) | [intent](intent.md) 결정 16 |
| report-2 | stage2-1 접힘 저장 실패(`open:peer:{id}` 열쇠 거절, #1q 부터) — 별도 Task(masterPlan 후보에 적음) 착수 시점 | [work/stage2](work/stage2.md) · [masterPlan](../../masterPlan.md) 후보 |
| report-3 | 1.3.7 릴리스 — origin push · 서명 빌드 · gh release | — |

## 산출

- web: `crates/app/web/app.css`(토큰 · 공통 부품 · 머리글자 · 종류 표시 · 띠 지움 · 메뉴 스위치) · `index.html` · `js/`(`logic/peer.js` `initialOf` · `logic/conv.js` `convKindIcon` ·
  `core/widgets.js` `tileFace` · `rowTile` · `rail.js` · `tiles.js` · `peers.js` · `home.js` · `search.js` · `settings/` · `shell.js` · `conv.js` · `sidebar.js` · `requests.js`).
- 시험: `ux_literals.rs`(행 · `token_contrast` 78 경우) · `view.test.mjs`(`initial_of` · `conv_kind_icon`) · `crates/agent/tests/conv_leave.rs`(stage2-3).
- Task 도구: [design/taste.py](design/taste.py)(AI 티 검사) · [design/labels.py](design/labels.py)(문구 비교).
- 문서: [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-13 줄) · README 1.3.7 · masterPlan D26 · D35 · S4 개정 · 후보 1.
- 버전 1.3.7 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`.
