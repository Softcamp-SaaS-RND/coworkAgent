# plan — 1z-10.conv-info

> 근거: [intent](intent.md)(승인 2026-10-23 — 결정 1 ~ 8). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)(D26 · S4 #1r).
> 브랜치 `task/1z-10-conv-info`(작업 폴더 `D:\dev\git\coworkAgent-1z-10`). 기준 커밋: `6594e46`(main). plan 승인: 2026-10-23(결정 7 — "개발 진행해." · "의견 수용" 으로 갈음, 구현자 해석).
> 시안: 대화 정보 https://claude.ai/artifact/Rw2MwLyD5LxopWRHQSUopW (A − 숫자 칸) · 아이콘 https://claude.ai/artifact/BPAKeSm7tfHwhhH3eqCoxL (매핑 표).

## 목표

세부정보 '대화 정보' 를 쓸모 있는 줄(기간 · 답하는 도구 · 읽는 폴더 · 컨텍스트 · 실행 명령 복사)로 바꾸고, 앱 전체 아이콘 없는 항목에 아이콘을 단다. 설정 상세의 단추 줄은 내용 바로 아래로.
버전 1.3.5 그대로.

### 성공 기준(intent 사본)

- A1: 세부정보 '대화 정보' = 받은 대화 기간 · 답하는 도구 · 읽는 폴더 · 컨텍스트 · 실행 명령(+ 복사), 보낸 대화 · 공유 사본 = 기간 · 컨텍스트 — 상대 · 내 에이전트 · 실행 기본값 줄 없음(시험 · 실제 화면).
- A2: 실행 명령 = 그 대화에서 이 PC 가 실제로 띄운 마지막 실행의 실행 파일 + 인자 그대로(세션 다시 실행이면 그 다시 실행 것, 실패한 실행도), 복사 = 그 전체 문자열(OS 셸 따옴표), 원격 본문 없음(시험).
- A3: 실행 명령은 이 PC 웹UI 의 그 대화 정보 응답에만 — 대화 목록 · 상대 프레임 · 관람자 · 검색 · 진단 정보에 없음(시험).
- A4: 아이콘 매핑 표의 항목마다 아이콘(기존 재사용 · 신규), 이미 아이콘 있는 메뉴 그대로, 설정 화면 제목 = 메뉴와 같은 아이콘 · 강조색(시험 · 실제 화면).
- A6: 설정 상세의 단추 줄이 내용 바로 아래 — 상세 칸이 화면보다 짧은 내용이면 단추 아래가 비고, 긴 내용이면 내용과 함께 스크롤되지 않고 칸 안에 남는다(실제 화면 — 초대 생성 · 수락 · 에이전트 상세).
- A5: 회귀 없음 — PROJECT.md 검증 명령 PASS, 스키마 28 올림(기존 대화 = 실행 명령 없음), 매뉴얼 · PRODUCT · masterPlan 반영.

## 설계

### stage 1 — 저장 · 실행 명령 · API

- core `db`: 스키마 28 = `conversations.cmd TEXT`(NULL = 없음) · `conversations.cmd_at INTEGER`(ALTER — 기존 대화 NULL).
- core `chat`: `set_cmd(conn, conversation_id, cmd, at)` · `ConvInfo { created_at, last_at, compacts, last_compact_at, clears, cmd, cmd_at }` · `info(conn, id) -> Option<ConvInfo>`.
  `last_at` = 그 대화 글의 가장 늦은 `created_at`(없으면 `created_at`), `compacts` = `ctx_op IN ('compact','auto_compact') AND state = 'done'` 수 · 그중 마지막 `created_at`, `clears` = `ctx_op = 'clear' AND state = 'done'` 수.
  `CONV_COLS` · `Conversation` 은 그대로(목록 응답에 명령을 싣지 않는다 — A3).
- agent `runner`(한 곳에서 만든다 — 저장한 명령 = 띄운 명령):
  - `agent_program(agent) -> PathBuf`(Codex = `find_codex()`, 그 밖 `find_claude()`) · `agent_args(input) -> Vec<String>`(Codex = `codex_args(agent, session)`, 그 밖 `claude_args(agent, session, model, effort)`).
    `run_agent` = `agent_program` 으로 고른 실행 파일, `run_claude` · `run_codex` 는 인자를 `agent_args(input)` 로 만든다(같은 함수).
  - `command_line(program, args) -> String` — Windows = PowerShell(`& '<실행 파일>' <인자 …>`, 인자가 `[A-Za-z0-9_\-.,=:/\\@+]+` 이면 그대로, 아니면 `'…'` · 안의 `'` 는 `''`),
    그 밖 = POSIX 셸(`'<실행 파일>' …`, 안의 `'` 는 `'\''`). 빈 인자 = `''`. 줄바꿈 · 한글 · `"` 는 작은따옴표 안에 그대로.
  - `launch_line(input) -> String` = `command_line(agent_program, agent_args)`. 원격 본문(표준 입력)은 넣지 않는다.
- agent `chat` `execute`: 실행 직전(`spawn_blocking` 앞) `launch_line(&input)` 을 `chat::set_cmd` 로 저장, 세션 다시 실행이면 그 입력으로 다시 저장(A2 — 실패한 실행도 남는다). 저장 실패는 경고 로그만(실행은 그대로).
  압축 요약 실행은 저장하지 않는다(intent 구현자 해석).
- app `chat_api`: `GET /api/conversations/{id}/info` → `{created_at, last_at, compacts, last_compact_at, clears, agent, cmd}`.
  `agent` · `cmd` 는 받은 대화(`dir = in`)에서만 — `agent = {engine, folder}`(대화 에이전트 이름으로 찾음, 지웠으면 `null`), `cmd = {text, at}`(없으면 `null`). 그 밖 dir = 둘 다 `null`. 없는 대화 = 404.

### stage 2 — 웹UI · 아이콘 · 단추 줄 · 문서

- '대화 정보'(index.html): `<dl class="info">` 의 `#info_peer` · `#info_agent` · `#info_defaults` 를 지우고 `<dl id="info_rows" class="info">`. 절 제목에 아이콘.
- `view.js` `convInfoRows(info, dir, now)` → `[{icon, label, text, mono, copy}]`:
  - 기간(`clock`) = `"{시작 formatTime} 시작 · 마지막 {마지막 formatTime}"`.
  - 답하는 도구(`bot`) = `engineLabel(engine)`(지운 에이전트면 "지운 에이전트") — in 만.
  - 읽는 폴더(`folder`, mono) = 폴더 — in 만.
  - 컨텍스트(`compress`) = `"압축 {n}회 · {마지막 formatTime}"` / `"압축 없음"`, 비우기가 있으면 `" · 비우기 {n}회"`.
  - 실행 명령(`terminal`, mono, `copy` = 전체 명령) = 명령 앞부분(화면은 한 줄 말줄임) / 없으면 `"아직 실행 없음"`(copy 없음) — in 만.
  - 복사 안내 `INFO_CMD_HINT` = "이어 쓰기 세션 인자가 든 명령 그대로 — 터미널에서 돌리면 이 대화 세션에 이어지고, 프롬프트는 표준 입력으로 받습니다."
- `app.js` `renderDetail` → `renderInfo(conv)`: 대화 · 그 대화 글 수 · 마지막 글 상태가 바뀔 때만 `GET …/info` 를 다시 부른다(응답 오기 전 = "불러오는 중 …", 다른 대화 응답은 버림).
  복사 = `navigator.clipboard.writeText(cmd.text)` · 단추 옆 "복사했습니다"(말풍선 복사와 같은 꼴).
- 아이콘(결정 4 · 시안 매핑 표 — 이미 있는 아이콘은 손대지 않는다):
  - `app.css` 새 `.ico-*`(mask SVG, 시안과 같은 패스): `activity · info · clock · folder · terminal · copy · eye · code · help · send · list · grid · resize · user-plus · folder-plus · cpu ·
    sliders · layers · palette · sort · volume · refresh · file · filter · key · globe · save · folder-up · git · message`.
  - 위계: 라벨 · 타일 · 단추 아이콘 = 글자색 16px(`.ico` 그대로), 설정 화면 제목(`.set-head h2`) = 메뉴 아이콘 + `var(--accent)`(토큰).
  - 붙이는 자리(시안 매핑 표 그대로):
    - 세부정보 절 제목: 이 PC 실행 `monitor` · 진행 `activity` · 대화 정보 `info` · 자동 왕복 `swap` · 관람자 `eye` · 요청 `inbox` · 북마크 `bookmark`, 관람자 추가 `plus`.
    - 상단 · 사이드: 새로 만들기 `plus`(앞 글자 "+" 를 아이콘으로) · 홈 목록 `list` · 미리보기 `grid` · 미리보기 크기 `resize` · 레일 "+ 사용자" → `user-plus` + "사용자" · "+ 그룹" → `folder-plus` + "그룹"
      (S4 의 "+ 사용자" · "+ 그룹" 글자를 아이콘으로 — 종결 때 masterPlan 개정) · 새로 만들기 메뉴 "세션 생성" 은 이미 아이콘이면 그대로.
    - 대화 머리 · 입력: 대화 보기 `chat` · 원문 보기 `code` · 질의 `help` · 요청 `inbox` · 보내기 `send`.
    - 설정 화면 제목: `settingsTree()` 의 아이콘을 제목 앞에(JS — 한 곳).
    - 설정 타일 라벨: 모델 `cpu` · effort `sliders` · 동시 실행 상한 `layers` · 자동 압축 `compress` · 테마 `palette` · 타일 순서 `sort` · 타일 크기 `resize` · 로그인할 때 자동 실행 `power` ·
      알림 받기 `bell` · 페어링 `link` · 받은 요청 `arrow-down` · 효과음 `volume` · 받은 답 `chat` · 알림에 미리보기 보이기 `eye` · 자동 왕복 `swap` · 자동 왕복 기본 상한 `reset`.
    - 설정 단추: 에이전트 등록 `plus` · 진단 정보 복사 `copy` · 새로 고침 `refresh` · 로그 제목 `file` · 수준(보이는 라벨이 없으면 선택 칸 앞) `filter`.
    - 연결 · 그룹 · 에이전트 상세: 모델 `cpu` · effort `sliders` · 상태 `activity` · 지문 `key` · 주소 `globe` · 대화 N `chat` · 저장 `save` · 연결 확인 `refresh` · 내 에이전트 편집 `pencil` ·
      삭제 `trash` · 닫기 `x` · 이 그룹의 대화 `chat` · 넣을 수 있는 대화 `list` · 넣기 `plus` · 빼기 `minus` · 이름 바꾸기 `pencil` · 이름 `title` · 실행 도구 `bot` · 아이콘 색 `palette` ·
      폴더(절대 경로) `folder` · Claude Code 옵션 · Codex 옵션 `gear` · 상위 한단계 탐색 허용 `folder-up` · 셸 명령 실행 허용 `terminal` · Git 저장소 아닌 폴더 허용 `git` · 추가 지시 `message` ·
      수락 `check` · 등록 `check` · 취소 `x`.
  - JS 도우미: `button(label, onClick, cls, icon)` · `field(label, control, box, icon)` · `convAct(…, icon)` 에 선택 인자(없으면 지금과 같음). `mdFoot` 닫기 = `x`.
- 단추 줄(결정 8 — 사용자 그림 "적당한 위치": 상세 상자가 단추 줄에서 끝남): `.md-detail` = `align-self: start` · `max-height: 100%`(상자 높이 = 내용, 칸보다 길면 칸 높이까지),
  `.md-body` = `flex: 0 1 auto`(내용 높이, 넘치면 줄어들며 그 안만 스크롤) — `.md-foot` 이 내용 바로 아래 · 상자 바닥. 초대 생성 · 수락 · 연결 사용자 · 그룹 · 에이전트 등록(1 · 2단계) ·
  에이전트 상세 모두 같은 규칙.
- 문서: [web-ui](../../manual/dev/web-ui.md)(대화 정보 · 아이콘 · 단추 줄) · [an_app](../../manual/dev/an_app.md)(`/info`) · [an_core](../../manual/dev/an_core.md)(스키마 28 · `info`) ·
  [developer-guide](../../manual/dev/developer-guide.md)(실행 명령 저장 · 따옴표) · [사용자 가이드](../../manual/user/user-guide.md)(세부정보 대화 정보 · 명령 복사 주의) · PRODUCT · masterPlan D26 · S4 · 개정 기록(종결).

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`: 아이콘이 들어가는 줄의 기대 문자열을 새 마크업으로(매처는 그대로), '대화 정보' 의 `info_peer` · `info_defaults` 줄 → `info_rows`.
- 새 시험: core 스키마 28 · `info` 셈(압축 · 비우기 · 끝난 것만 · 마지막 시각) · `set_cmd`, agent `command_line`(작은따옴표 · 큰따옴표 · 줄바꿈 · 한글 · 빈 인자 · 그대로 인자) ·
  `agent_args` = `claude_args` / `codex_args`, 대화 실행이 명령을 남김(가짜 실행기 — 새 세션 `--session-id`, 세션 다시 실행 = 새 세션 명령, 실패한 실행도), app `/info`(in = agent · cmd, out · shared = null,
  404, 대화 목록 · 글 목록 응답에 `cmd` 없음), `view.test.mjs` `convInfoRows`(in · out · shared · 압축 없음 · 명령 없음).

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — 웹UI 로직 시험(대화 정보 줄 · 받은 / 보낸 / 공유)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A2 — 실행 명령(따옴표 · 한 곳에서 만든 인자 · 대화 실행이 남김 · 다시 실행 · 실패)
  CHECK: cargo test -p an-agent --test runner --test conv_cmd
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A1·A3 — 대화 정보 API(in · out · shared · 404 · 목록에 명령 없음)
  CHECK: cargo test -p coworkagent-app --test chat_api conv_info
  EXPECT: /test result: ok\. [1-9] passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A1·A4·A6 — 화면 원본 literal(대화 정보 · 아이콘 자리 · 설정 제목 강조 · 단추 줄 규칙) · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A5 — 문서 규약 · 크레이트 경계 · 스키마 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('schema=%d app=%d core=%d web=%d guide=%d'%(r('crates/core/src/db.rs').count('SCHEMA_VERSION: i64 = 28'),r('docs/manual/dev/an_app.md').count('/info'),r('docs/manual/dev/an_core.md').count('cmd_at'),r('docs/manual/dev/web-ui.md').count('info_rows'),r('docs/manual/user/user-guide.md').count('실행 명령')))"
  EXPECT: /schema=1 app=[1-9][0-9]* core=[1-9][0-9]* web=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: pending
- [x] E8: A1 · A4 · A6 — 실제 화면: 디버그 빌드 + 빈 데이터 · ego-browser 1440 × 900 — 받은 대화 세부정보 대화 정보 줄 · 실행 명령 복사(클립보드 = 저장값), 보낸 대화 = 기간 · 컨텍스트,
  매핑 표 항목 아이콘(설정 · 상세 · 대화 머리 · 입력 · 상단 · 레일) · 설정 제목 강조색, 초대 생성 · 수락 · 에이전트 상세 · 그룹 상세 단추 줄이 내용 바로 아래
  EVIDENCE: Claude(구현자) 2026-10-23 — 디버그 빌드 + scratchpad `ui1z8\data`(스키마 28), ego-browser DOM 판독(창 최소화 — 스크린샷 없음). 받은 대화 = 기간 · 답하는 도구 · 읽는 폴더 ·
  컨텍스트 "압축 1회 · 어제 22:22" · 실행 명령 + 안내, 복사 = 저장값 그대로("실행 명령을 복사했습니다"), 보낸 대화 = 기간 · 컨텍스트. 절 제목 7 · 설정 제목 8(강조색 rgb(36, 91, 171)) ·
  타일 라벨 · 상세 라벨 · 단추 · 대화 머리 · 입력 · 상단 · 레일 아이콘. 단추 줄 = 상자 바닥 = 내용 바로 아래(1440 × 900 · × 2000 — 초대 생성 462 · 수락 431 · 사용자 590 · 그룹 599 · 에이전트 814,
  칸 아래 858 / 1958), 1440 × 560 = 본문 스크롤 · 단추 줄 보임. 상세: [work/stage2.md](work/stage2.md) 테스트 결과.

## 개정 기록

- 2026-10-23(구현 중 — 설계만, 평가 항목 · EXPECT 그대로): ① 따옴표 없이 두는 글자에서 `,` · `@` · `+` 뺌(PowerShell 배열 · 스플랫). ② 아이콘 자리를 매핑 표 밖 같은 꼴(요청 화면 목록 · 보드 · 상태, 연결 승인 · 다시 알리기)까지.
