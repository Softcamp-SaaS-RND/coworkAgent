# stage 1 — 저장 · 실행 명령 · API

## 목표

plan "설계 · stage 1" — 대화의 마지막 실행 명령 저장(스키마 28), 대화 정보 셈(`chat::info`), 실행 명령을 띄울 때와 같은 함수로 만들기(`agent_args` · `launch_line`), `GET /api/conversations/{id}/info`.

## 산출 파일

- core `db.rs` — 스키마 28(`conversations.cmd` · `cmd_at`). `chat.rs` — `set_cmd` · `ConvInfo` · `info`.
- agent `runner.rs` — `agent_program` · `agent_args`(`run_claude` · `run_codex` 도 이것으로 인자를 만든다) · `launch_line` · `shell_line`. `chat.rs` — `execute` 가 띄우기 직전 `note_cmd`(세션 다시 실행이면 다시).
- app `chat_api.rs` — `conversation_info`(`/api/conversations/{id}/info`).
- 시험: core `paths.rs` `schema28_conv_info`(+ 스키마 번호 단정 28) · agent `runner.rs` `launch_line_matches_spawn_args` · 새 `conv_cmd.rs` `run_records_launch_line` · app `chat_api.rs` `conv_info_api`.

## 설계 결정

- 따옴표 없이 두는 인자 = 글자 · 숫자 · `_-./:=`(PowerShell 은 `\` 도). 쉼표는 PowerShell 에서 배열 연산자라 감싼다(`--allowedTools Read,Grep,…` → `'Read,Grep,…'`). POSIX 는 역슬래시가 이스케이프라 감싼다.
  plan 의 허용 글자(`,` · `@` · `+` 포함)에서 줄임 — 붙여 넣었을 때 뜻이 바뀌는 글자.
- 실행 파일 표기: Windows = `& '<경로>'`(경로에 공백이 흔함 — 호출 연산자), POSIX = `'<경로>'`.
- `info` 의 압축 · 비우기 셈은 `state = 'done'` 만(진행 중 · 실패한 압축은 세지 않음). 마지막 = 그 대화 글의 가장 늦은 시각(글이 없으면 시작).
- A3 — 명령 열은 `CONV_COLS` · `Conversation` 밖이라 대화 목록 · 상대 프레임 · 관람자 사본 · 검색 · 진단 정보 경로가 읽지 않는다(구조). 시험은 목록 · 글 목록 응답에 명령이 없음을 잰다.
- 명령 기록 실패는 경고 로그만 — 화면 표시용이라 실행을 막지 않는다.

## 테스트 결과

- `cargo test -p an-core --test paths` → `test result: ok. 18 passed; 0 failed`.
- `cargo test -p an-agent --test runner` → `test result: ok. 18 passed; 0 failed`.
- `cargo test -p an-agent --test conv_cmd --test resume_missing` → `ok. 1 passed` · `ok. 5 passed`.
- `cargo test -p coworkagent-app --test chat_api conv_info` → `test result: ok. 1 passed; 0 failed`.
- `cargo test --workspace --no-fail-fast` → `test result: FAILED` 0줄 · ok 83줄.
- `cargo clippy --workspace --all-targets -- -D warnings` → Finished.

## 다음 단계

stage 2 — 웹UI 대화 정보 · 아이콘 · 단추 줄 · 문서(plan "설계 · stage 2").
