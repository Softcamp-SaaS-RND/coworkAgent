# report — 1k.ui-polish

종결: 2026-10-15 · 브랜치 `task/1k-ui-polish` → main

## 결과

1j 다듬기 — 에이전트 화면 · 주고받은 글의 머리 안내문을 없애고, 내 에이전트 줄 오른쪽 ✎(설정의 그 에이전트를 펼쳐 "관리" 에 초점) · 주고받은 글 줄 왼쪽 상대 타일 · 이름 옆
에이전트 칩(보낸 대화 = 상대 에이전트, 받은 대화 = "내 {에이전트}")을 넣었다. 대화 말풍선 아래 "다시 보내기" 를 없애고(입력창에서 다시 쓴다), 레일 '+' 는 설정 '연결' 처럼
페어링 카드를 맨 위로 · 초대 수락 칸을 펼쳐 초점. 설정은 디자인 캔버스 ④ 대로 두 열(페어링 · 연결 목록 | 내 정보 · 내 에이전트) · 페어링 큰 단추 둘 · 관리 줄(타일 · 이름 ·
부제 · "그룹 ▾" · "관리") · "에이전트 등록".

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 에이전트 안내문 없음 · 내 에이전트 연필 → 설정 그 에이전트 | 충족 | E4 `polish_literals` · E8(연필 4 · `agent{id}_manage` 펼침 초점) |
| A2 주고받은 글 안내문 없음 · 상대 타일 · 에이전트 칩 | 충족 | E3 `inbox_rows_agent` · E4 · E8(줄 6 = 타일 6 = 칩 6) |
| A3 다시 보내기 없음 | 충족 | E3 `message_actions_no_resend` · `message_actions` · E4 · E8(0) |
| A4 레일 + → 페어링 맨 위 · 수락 칸 | 충족 | E4 · E8(위 0px · 펼침 · 초점 `invite_text`) |
| A5 설정 디자인 캔버스 ④ 배치 | 충족 | E4 · E8(두 열 · 단추 둘 같은 class · 관리 · 줄 타일 · 에이전트 등록) · mock 1000px 한 열 |
| A6 검증 명령 | 충족 | E1 · E2 · E3 · E5 · E6 · E7 · `check_gates` 8/8 |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(stage 문서가 아직 커밋 안 된 eval 을 가리킴) · medium 1(여러 그룹의 같은 상대 관리 줄 id · 펼침
  겹침) — eval 커밋 · 열쇠 `peer{id}_{그룹|none}`. 라운드 2: medium 2(연결 목록 접속 상태 부제가 시간이 흘러도 그대로 · 남은 에이전트 검색이 연필 진입을 가림) —
  부제만 매 새로 고침에 고침(`peerSub` · `data-peer-sub`, 관리 폼 보존) · `editAgent` 가 검색 비움. 상한 도달 — 재리뷰 없음, 미조치 0([eval](eval.md)).
- plan 편차(stage1): 관리 = `aria-expanded` 토글 · "초대 생성" 안내(5분 · 1건 · 모르는 연결 삭제) 남김 · 에이전트 화면 상대 줄 부제 그대로.
- 이 PC 반영: 릴리스 빌드(라운드 1 이전 코드)로 화면 확인 10/10(비파괴). 종결 뒤 main 최종 빌드로 다시 띄워 재확인.
- masterPlan: S4 보강 · 개정 기록.

## 산출

- 웹UI: `view.inboxRows` `agent` · `messageActions`(resend 없음) · `refusedText` · `app.rowEls {tile}` · `editAgent` · `showAccept` · `manageItem` · `rowTile` ·
  `svgButton` · 설정 두 열 · 페어링 카드 · 관리 줄 · "에이전트 등록". `remembered` 없앰.
- 시험: `view.test.mjs` 96 → 98 · `ux_literals.rs` 17 → 18.
- 문서: 사용자 가이드 · an_app · PRODUCT · README · masterPlan · 캡처 2장(페어링 · 에이전트, 가상 데이터).

## 남은 것

- 19 인수 TC — 여러 상대에서 주고받은 글 타일 색 · 한 상대가 여러 그룹일 때 연결 목록 관리 줄.
- 캡처 중 주고받은 글 · 에이전트 화면은 아직 없다(다음 캡처 갱신 때).
- GitHub 반영(push) · 릴리스(1h · 1j · 1k 함께)는 사용자 확인 뒤.
