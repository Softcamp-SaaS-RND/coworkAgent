# report — 1z.chat-look

종결: 2026-10-21 · 브랜치 `task/1z-chat-look` → main(squash `0e9f95e` 로 GitHub 반영 뒤 종결 문서)

## 결과

사람 ↔ 에이전트 구분(말풍선 A · 에이전트 로봇 아바타 양쪽 · 대화 목록 카드 · 발화자 아이콘), 입력창 여백 `max(48px, 8%)` · 스크롤 영역 밖 24px · 글 칸 44 → 200px,
요청 "피드백" · "메모" → "의견"(여러 줄 200자 · Enter 확인 · Shift+Enter 줄바꿈 · 보낼 때 U+2028 — 이전 버전 호환), 레일 폭 손잡이 끌기로 타일 크기(놓으면 72 · 168px),
레일 사용자 · 그룹 타일 순서 포인터 끌기, 설정 "연결 목록" → "그룹설정"(시안 A 목록 + 상세). 시안은 Design 캔버스 "coworkAgent UI 시안"(사용자 확정).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 사람 · 에이전트 말풍선이 다르다 | 충족 | E7 · E11(실 화면 대비 라이트 17.47 · 다크 14.45 / 6.25 · 8.58) |
| A2 에이전트 말풍선 로봇 아바타 — 내 쪽 포함 | 충족 | E7 · E11 |
| A3 대화 목록 카드 · 발화자 아이콘 | 충족 | E6 · E8 |
| A4 메시지 줄 사람 / 에이전트 아이콘 | 충족 | E6 |
| A5 입력창 여백 · 44 → 200px · 스크롤 밖 24px | 충족 | E7 · E11(여백 79.375px · 간격 24px) |
| A6 "피드백" · 요청 "메모" 0건 → "의견" | 충족 | E9 |
| A7 의견 칸 3줄 · 200 · 글자 수 · 44px · Enter / Shift+Enter | 충족 | E6 · E8 |
| A8 의견 줄바꿈 저장 · 전달 · 이전 버전 호환 · 거부 | 충족 | E4(`note_lines_200` · `note_wire_old_peer`) · E5(`note_lines_reach_requester`) |
| A9 레일 손잡이 끌기 · 키 → 타일 크기 저장(경계 119 · 120 · 121) | 충족 | E6 · E7 · E11 |
| A10 회귀 없음 | 충족 | E1 · E2 · E3(node 158) · E10 |
| A11 타일 순서 포인터 끌기 5px · HTML 끌기 없음 | 충족 | E12 · E13 · E11(순서 바꾼 뒤 되돌림) |
| A12 "연결 목록" 0건 → "그룹설정" | 충족 | E14 |
| A13 그룹설정 목록 + 상세 · 검색 · 고른 항목 대체 | 충족 | E12 · E14 · E11 |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex — `codex-peer-review`) — 라운드 1 medium 3(그룹 이름 검색 상세 · 폭 손잡이 취소 · 끄는 중 다시 그리기),
  라운드 2 medium 2(그림 타일 기본 끌기 · 그룹설정 접속 점) 모두 반영. 미조치 2행(DOM 이벤트 경로 자동 시험 없음)은 사용자 결정 "기록만하고 종료"(2026-10-21) — 원장 `수용`([eval](eval.md)).
- plan 개정: E7 · E9 · E10 · E13 · E14 CHECK 보정(EXPECT 그대로), A11 ~ A13 · E12 ~ E14 는 사용자 추가 요청으로 intent 개정과 함께.
- plan 밖 변경: 리뷰 경로 개정(사용자 지시 — 기존 `review*` 터미널 배정 폐지 → Orca 새 Codex, `PROJECT.md` · 사용자 전역 지침 · 스킬), squash 뒤 브랜치 삭제 절차(`PROJECT.md`),
  `arapaima/` 워크트리 삭제(사용자 지시 — `.graphifyignore` · `PROJECT.md` 언급 정리).
- 다른 저장소 반영: AiStorage `tools/cowrkAgent` 커밋 `095d9288`(ADO push — 1021-cowork 세션).

## 산출

- an-core: `clean_req_note`(여러 줄 200자) · `note_to_wire` · `note_from_wire`.
- an-agent: `set_req_state` 보낼 때 U+2028, `inbound_req_state` 받을 때 되돌림.
- 웹UI: 말풍선 A · `agent-av` · `agent-tag`, `speakerEl`, 의견 칸 `noteForm`, `fitComposer`, `#rail_resize` · `RAIL_W`, `tileGrab`(레일 타일 포인터 끌기), 그룹설정 두 칸
  (`groupSettingsList` · `groupSettingsPick` · `gsDot`), 토큰 `--chat-gutter` · `--rail-w-normal`.
- 시험: Rust core 2 · agent 1(+ 거부 목록), `ux_literals` PRESENT · ABSENT, node 새 7(158).
- 문서: masterPlan S4 · S1 · D8 · D35 · D37 · 개정 기록 · 종결 목록, PRODUCT, README, 사용자 가이드, web-ui · an_core · an_app · an_agent · an_net · 개발자 가이드, 원장 2행.

## 남은 것

- DOM 이벤트 경로(포인터 취소 · 캡처 상실 · 끄는 중 다시 그리기 · 접속 점 75초 경계) 자동 시험 — 원장 `#1z R1·R2-D1` · `#1z R2-D2`(수용).
- 버전은 1.1.1 그대로(릴리스 없음).
