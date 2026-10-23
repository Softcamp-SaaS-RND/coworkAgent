# report — 1z-10.conv-info

종결: 2026-10-23 · 브랜치 `task/1z-10-conv-info` → main(squash — 사용자 승인 뒤)

## 결과

세부정보 '대화 정보' 를 쓸모 있는 줄로 바꾸고(시안 A − 숫자 칸 — 사용자 결정), 앱 전체의 아이콘 없는 라벨 · 절 제목 · 단추에 글자 앞 아이콘을 달고(아이콘 시안 · "의견 수용"),
설정 상세의 단추 줄을 내용 바로 아래로 옮겼다(진행 중 사용자 추가 지시 · 그림). 버전 1.3.5 그대로(사용자 "1.3.5 에 합침") · 스키마 28.

- 대화 정보: 받은 대화 = 기간(시작 · 마지막) · 답하는 도구 · 읽는 폴더 · 컨텍스트(압축 n회 · 마지막, 비우기) · **실행 명령**(+ 복사 · 안내), 보낸 대화 · 공유 사본 = 기간 · 컨텍스트.
  옛 상대 · 내 에이전트 · 실행 기본값 줄 없음. `GET /api/conversations/{id}/info` — 대화 목록 · 상대 프레임에는 명령 없음.
- 실행 명령 = 이 PC 가 그 대화에서 실제로 띄운 마지막 실행의 실행 파일 + 인자(`runner::launch_line` — 띄울 때 쓰는 `agent_program` · `agent_args` 그대로), 띄우기 직전 저장(세션 다시 실행 ·
  실패 실행 포함), 원격 본문 없음. 따옴표: Windows = PowerShell 7.3+(작은따옴표 · 둥근 따옴표 두 번 · 앞에 7.3 미만 · `Legacy` 인자 전달이면 멈추는 줄), 그 밖 POSIX.
- 아이콘: 새 30개(`app.css` mask SVG) · `.ico.lead`(글자색 16px) · 설정 화면 제목 = 메뉴 아이콘 + 강조색. 이미 아이콘 있는 메뉴는 그대로. "+ 새로 만들기" · 레일 "+ 사용자" · "+ 그룹" 의 `+` 글자 → 아이콘.
- 설정 상세(초대 생성 · 수락 · 연결 사용자 · 그룹 · 에이전트): 상자 높이 = 내용, 단추 줄 = 내용 바로 아래 · 상자 바닥, 칸보다 길면 본문만 스크롤, 빈 상세 안내는 칸 전체.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 대화 정보 줄(받은 / 보낸 · 공유) · 옛 줄 없음 | 충족 | E3(`conv_info`) · E5(`conv_info_api`) · E6(없어야 할 `info_peer` · `info_defaults`) · E8 |
| A2 실행 명령 = 띄운 그대로 · 다시 실행 · 실패 · 본문 없음 · OS 따옴표 | 충족 | E4(`launch_line_matches_spawn_args` · `conv_cmd` · `powershell_line_roundtrip` 실제 pwsh 7 / 5.1 / Legacy argv 왕복) |
| A3 명령은 그 대화 정보 응답에만 | 충족 | E5(목록 · 글 목록에 없음) · 구조(`CONV_COLS` · `Conversation` 밖 열) |
| A4 매핑 표 아이콘 · 기존 메뉴 그대로 · 설정 제목 강조 | 충족 | E6 · E3 · E8(절 제목 7 · 설정 제목 8 · 라벨 · 단추) |
| A6 단추 줄 = 내용 바로 아래 | 충족 | E6(CSS 규칙) · E8(1440 × 900 · × 2000 · × 560 측정) |
| A5 회귀 없음 · 스키마 28 · 문서 | 충족 | E1 · E2 · E7 · core `schema28_conv_info` |

- 적대적 평가 2 라운드(상한, Orca 가 라운드마다 새로 띄운 Codex): R1 = high 1(PowerShell 둥근 따옴표 인용 탈출 → 명령 주입) · medium 2(Windows PowerShell 5.1 인자 손실 · /info 실패 뒤 다시 읽기 없음)
  반영, R2 = medium 2(7.3+ `Legacy` 인자 전달 · A → B → A 늦은 응답 역전) 반영 — 재리뷰 없음. 미조치 1(R2-2 의 회귀 시험 — 원장 `#1z-10 R2-2` 열림).
- 편차(사용자 확인 필요): 결정 2 "실행 명령 그대로" — 복사 문자열 앞에 PowerShell 버전 · 인자 전달 확인 줄(`PS_MIN_GUARD`)이 붙는다(R1 · R2 — 5.1 과 7.3+ 를 함께 만족하는 한 줄이 없어
  조용히 다른 명령이 도는 대신 멈춤). 실행 파일 + 인자 부분은 그대로. **→ 2026-10-24 사용자 결정: 확인 줄 뺌**(종결 · v1.3.5 릴리스 뒤 — 복사 = `& '<실행 파일>' …` 만,
  5.1 · `Legacy` 인자 손실은 사용자 몫, 버전 그대로. v1.3.5 릴리스 실행 파일에는 확인 줄이 남아 있음). 라벨 "CLI"(시안 A) → "실행 명령"(아이콘 시안) — intent 결정 2 에 기록.
- 구현자 해석(사용자 미확인 — intent):
  - "의견 수용" = 시안 보고의 U1(매핑 표대로) · U2(한 Task) · U3(보낸 대화 보드 따로 없음) 답으로 읽음, intent · plan 승인 갈음.
  - 공유 사본(관람자) = 보낸 대화처럼 기간 · 컨텍스트만.
  - 실행 명령에 세는 실행 = 요청에 답한 실행(세션 다시 실행 포함), 압축 요약 실행은 세지 않음.
  - 명령에 작업 폴더(`cd`)를 넣지 않음(폴더는 '읽는 폴더' 줄).
  - 단추 줄 이동(결정 8)을 이 Task 에 넣음 · 연결 사용자 상세도 같은 규칙.
  - 매핑 표 밖 같은 꼴(요청 화면 목록 · 보드 · 상태, 연결 승인 · 다시 알리기)에도 아이콘.
- 보안 영향: 실행 명령에는 그 PC 의 가드 문구 · 에이전트 추가 지시 · 폴더 경로가 든다 — 이 PC 웹UI(토큰 보호 로컬 API)에만, 상대 · 관람자 · 검색 · 진단 정보로 가지 않는다.
  복사한 명령을 돌리면 그 대화 세션에 이어진다(안내 문구).

## 산출

- core: 스키마 28(`conversations.cmd` · `cmd_at`) · `chat::set_cmd` · `ConvInfo` · `info`.
- agent: `runner::agent_program` · `agent_args` · `launch_line` · `shell_line` · `PS_QUOTES` · `PS_MIN_GUARD`(`run_agent` · `run_claude` · `run_codex` 가 같은 함수로 띄움), `chat::execute` 의 `note_cmd`.
- app: `chat_api` `conversation_info`(`/api/conversations/{id}/info`).
- web: `index.html` `#info_rows` · 글자 앞 아이콘, `app.css` 새 아이콘 30 · `.ico.lead` · 설정 제목 강조 · `.info-cmd` · 단추 줄 규칙, `app.js` `lead` · `button`/`field`/`convAct` 아이콘 인자 ·
  `renderTree` 제목 아이콘 · `renderInfo`(세대) · `drawInfo` · `copyLine`, `view.js` `convInfoRows` · `INFO_CMD_HINT`.
- 시험: core `schema28_conv_info` · agent `runner`(19 — `launch_line_matches_spawn_args` · `powershell_line_roundtrip`) · 새 `conv_cmd` · app `conv_info_api` · `view.test.mjs` `conv_info` ·
  `ux_literals` #1z-10 행 · 아이콘 든 기대값 갱신.
- 문서: web-ui · an_core · an_app · developer-guide · user-guide · PRODUCT · ledger · masterPlan D26 · S4(#1r 아래 · 레일 글자) · 종결 목록 · 개정 기록.
  시안: 대화 정보 https://claude.ai/artifact/Rw2MwLyD5LxopWRHQSUopW · 아이콘 https://claude.ai/artifact/BPAKeSm7tfHwhhH3eqCoxL.

## 남은 것

- main squash 반영 · push · 1.3.5 서명 릴리스(#1z-9 와 함께) — 사용자 결정.
- 원장 `#1z-10 R2-2` — 대화 정보 늦은 응답 역전 회귀 시험(`renderInfo` 를 `view.js` 로 빼는 구조 변경) — 사용자 결정.
- macOS(POSIX 따옴표 실제 셸 왕복) · 실제 두 PC 연동 — 19 acceptance · Mac 실기 조건 대기(D28).
