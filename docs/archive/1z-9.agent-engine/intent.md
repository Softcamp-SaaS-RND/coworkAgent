# intent — 1z-9.agent-engine

승인: 2026-10-23

## 왜

사용자 요청(대화, 2026-10-23) 원문:

> 신규 task 로 작업.
> 에이전트 등록 시 claude / codex 중 택1하도록 강제.
> 한번 만들어진 에이전트와의 대화는 codex, claude 중 한개만 사용(사용 도중 변경 못함)
> --skip-git-repo-check 기본 옵션 제공 검토.
> codex 선택 시 claude 에서 제공하는 옵션은 disable 되도록 UX 제공.
>
> 위 사항에 대해서 에이전트 등록화면 아티팩트 시안 2개 작성 후 결정.

앞선 검토 요청: "에이전트 등록 시 codex, claude 선택 가능하도록 옵션 제공 검토."

출처: 대화, masterPlan D16(`claude -p` 헤드리스 실행) · D19(대화마다 Claude 세션 ID) · D20(Claude 계정 = 각 PC 의 Max 구독 로그인 — 정책 고지 뒤 사용자 결정) · D21(에이전트 프로필 — 폴더 = 원격 범위, #05-2 상위 읽기 · #05-3 셸) ·
D22(헤드리스 실행 권한 — `--restricted --strict-mcp-config --permission-prompts none` · `--tools`) · D31(가드 프롬프트 · 도구 고정). 지금은 Claude Code 만 실행한다 —
Codex 를 쓰는 PC 는 에이전트를 둘 수 없다.

### 코드 사실 (2026-10-23 확인 — main `38c2c73`, 로컬 `codex-cli 0.163.0`)

| 사실 | 위치 |
| --- | --- |
| 실행기는 한 곳 — `claude_args` · `run_claude` · `find_claude` · `check_version`(`MIN_CLAUDE` 2.1.267), 운영 연결 = `main.rs` `run_claude(&find_claude(), &input)` | `crates/agent/src/runner.rs` · `crates/app/src/main.rs:325` |
| 실행 인자 = `-p --output-format stream-json --verbose --restricted --strict-mcp-config --permission-prompts none --permission-mode dontAsk --tools/--allowedTools <Read,Grep,Glob(+셸)>` + `--add-dir`(상위 읽기) + `--append-system-prompt`(가드 + 추가 지시) + `--system-prompt-snapshot off` + `--session-id`/`--resume` + `--model` · `--effort` | `runner.rs` `claude_args` |
| 세션 id 는 우리가 정한다(`Session::New(id)` → `--session-id`), 없는 세션 판별은 Claude 영문 출력 문장(`session_gone`) | `runner.rs` |
| 출력 = stream-json — 단계 요약(`count_step` — `tool_use` 이름), 창 사용량(`ctx_tokens` — `usage.iterations`), 결과 줄 `session_id` | `runner.rs` |
| 에이전트 = `agents`(id · name · folder · prompt · grp · uid · parent_read · shell · color), 세대 `uid` = 등록 · 폴더 · 상위 읽기 · 셸 변경 때 새 값(세션 버림) | `crates/core/src/agents.rs` |
| 모델 · effort = **상대별**(`peers.model` · `effort`) + 기본값(설정 '실행'), 값 = Claude 별칭 `opus · sonnet · haiku` · `low ~ max`. 받은 답의 model · effort 도 같은 목록으로 거른다(`run_meta`) | `crates/core/src/run_opts.rs` · `crates/agent/src/chat.rs:256` |
| `codex exec` — 표준 입력 프롬프트(`-`) · `--json`(JSONL) · `exec resume <id> -` · `-s read-only` · `-C <dir>` · `--add-dir`(**쓰기** 허용 폴더) · `-m` · `-c key=value` · `--skip-git-repo-check` · `--ignore-user-config` · `--ignore-rules` · `--ephemeral`. 세션 id 를 우리가 정하는 인자 없음, 도구 목록 · 시스템 프롬프트 덧붙이기 인자 없음 | `codex exec --help` · `codex exec resume --help`(0.163.0) |
| 이 PC 의 codex = npm 설치(`%APPDATA%\npm\codex.cmd` · `codex.ps1` · `codex` 셸 스크립트) | `which -a codex` |

## 범위

시안 캔버스: https://claude.ai/artifact/2BKZnhDQXWmwG2oskYiMfT — 시안 A(한 폼 · 실행 도구 카드 · Codex 옵션 스위치) · 시안 B(실행 도구 먼저 2단계 · 좌우 옵션 블록) — **B 로 결정**(결정 1).

- 에이전트 등록 = **실행 도구 Claude Code / Codex 택1 필수**(기본 선택 없음 — 고르기 전 등록 불가).
- 실행 도구는 등록 뒤 바꿀 수 없다 — 그 에이전트와의 대화는 고른 도구로만 실행(바꾸려면 새로 등록).
- Codex 를 고르면 Claude 전용 옵션(상위 한단계 탐색 · 셸 명령 실행 · 도구)은 비활성 + 이유 문구.
- Codex 실행기 — `codex exec` 인자 · `--json` 해석(답 · 단계 요약 · 창 사용량) · 세션 이어 쓰기 · 없는 세션 판별 · 실행 파일 찾기 · 버전 하한.
- `--skip-git-repo-check` = Codex 옵션 스위치 "Git 저장소 아닌 폴더 허용" 기본 켬(결정 2).
- DB `agents.engine`(스키마 27, 기존 행 = `claude`), API · 웹UI · CLI · 매뉴얼 · masterPlan(D16 · D19 · D21 · D22 · D31 개정) 반영.

## 제외

- 등록된 에이전트의 실행 도구 바꾸기(사용자 지시 — 사용 도중 변경 못함).
- Claude · Codex 밖의 실행 도구.
- Codex 에이전트의 폴더 밖 읽기 차단(결정 5 — 실측상 `-C` 는 읽기 경계가 아니다, 가드 문구만).
- Codex 모델 · effort 고르기(결정 3 — 다음 Task).
- 등록하는 PC 에 Codex CLI 가 있는지로 등록을 막기(결정 6).
- Codex 계정 약관 판단 · 고지(결정 7 — 사용자 책임).
- Claude · Codex 에이전트의 폴더 밖 읽기 차단(결정 5 · 9 — 가드 문구만).

## 성공 기준

- A1: 에이전트 등록 = 시안 B 2단계 — 1단계에서 실행 도구(Claude Code / Codex)를 고르지 않으면 2단계 · 등록으로 갈 수 없고, API 도 도구 없는 등록을 거부한다(시험 · 실제 화면).
- A2: 등록된 에이전트의 실행 도구는 API · 웹UI 어디서도 바뀌지 않고, 그 에이전트의 대화 실행은 늘 그 도구다(시험).
- A3: Codex 에이전트(등록 2단계 · 상세)는 Claude Code 옵션 블록(상위 한단계 탐색 · 셸 명령 실행 · 도구)이 비활성 · "Codex 에이전트는 쓰지 않음", Claude 에이전트는 Codex 블록이 비활성(시험 · 실제 화면).
- A4: Codex 에이전트에 묻고 답을 받는다 — `codex exec -C <등록 폴더> -s read-only`(표준 입력 프롬프트) · 새 대화 · 이어 쓰기 · 단계 요약 · 실패 표시(시험 · 실제 Codex 실측).
- A5: Codex 옵션 "Git 저장소 아닌 폴더 허용" 기본 켬 = `--skip-git-repo-check` 붙음, 끄면 빠짐(시험).
- A6: 기존 에이전트 = Claude Code 로 그대로 동작(스키마 27 올림, 회귀 없음).
- A8: Claude 실행 인자 = 결정 9(`--restricted` · `--strict-mcp-config` · `--tools` 없음, `--allowedTools` 읽기 도구(+ 셸), `--disallowedTools` 편집 도구(+ 셸 끔이면 셸)), Codex 실행 = 결정 8(`--ignore-user-config` 없음 · `-s read-only`)(시험).
- A7: 회귀 없음 — PROJECT.md 검증 명령 PASS, 매뉴얼 · README · PRODUCT · masterPlan(D16 · D19 · D21 · D22 · D31 개정) 반영.

## 결정

1. (U1 — 2026-10-23 사용자) 등록 화면 = **시안 B**(실행 도구 먼저 고르는 2단계 — 1단계 = 도구 타일 2개(읽기 · 셸/Git · 모델 비교), 2단계 = 고른 도구 칩 · "도구 다시 고르기" ·
   이름 · 폴더 · 좌우 옵션 블록(안 고른 도구 블록 = 점선 · 비활성 · "… 에이전트는 쓰지 않음") · 추가 지시). 상세 = 머리 이름 옆 도구 칩 · "이름 · 실행 도구는 등록 뒤 바꿀 수 없음" ·
   같은 좌우 블록. 목록 줄 부제 = "도구 · 폴더". 기각: 시안 A(한 폼 · 도구 카드 — 권장안이었음).
2. (U2 — 2026-10-23 사용자) `--skip-git-repo-check` = Codex 옵션 블록의 **스위치 "Git 저장소 아닌 폴더 허용", 기본 켬**(에이전트마다 끌 수 있음 — 끄면 인자 없음).
   기각: 항상 붙임(권장안이었음). 시안 B 캔버스의 Codex 블록도 스위치로 고침.
3. (U4 — 2026-10-23 사용자) Codex 모델 · effort = 1차는 **Codex 기본값**(모델 · effort 인자 없음 — 그 PC Codex 설정). 상대별 모델 · effort 는 Claude 에이전트에만, Codex 모델 고르기는 다음 Task.
4. (U3 사용자 답 — 2026-10-23) "codex exec -C \"D:\dev\my-project\" … 와 같이 실행 경로를 에이전트 등록된 경로에 한정해서 읽기 범위 지정." → Codex 실행 = `-C <등록 폴더>`(작업 루트).
   단 아래 실측상 `-C` 는 읽기를 막지 않는다 — 결정 5 로 다시 확인함.
5. (U3' — 2026-10-23 사용자, 실측을 본 뒤) Codex 읽기 범위 = **`-C <등록 폴더>` 만, 등록 때 확인 없음**. 폴더 밖 읽기는 가드 문구(모델 규칙)로만 막는다 —
   D21 · D22 "폴더 = 원격 범위"(Claude 는 도구 · `--add-dir` 로 강제)를 Codex 에이전트에 대해 개정(종결 때 masterPlan). 기각: 등록 때 위험 확인(권장안이었음) · 읽기 제한을 찾을 때까지 보류.
6. (U5 — 2026-10-23 사용자) "사용자인 내 PC 에는 codex 가 없어도 됨. 왜냐하면 상대방 PC 의 codex 가 응답하기 때문." → 등록 1단계에서 CLI 설치 여부로 타일을 막지 않는다.
   구현자 해석(확인 필요): 에이전트는 등록한 PC 에서 실행되므로(D21) 그 PC 에 Codex 가 없으면 답할 때 실패 — 실패 글에 "이 PC 에서 Codex CLI 를 찾지 못했습니다" 를 싣는다.
7. (U6 — 2026-10-23 사용자) Codex 계정 · 약관 = 사용자 책임 — 앱은 그 PC 의 Codex 로그인을 그대로 쓴다(D20 같은 고지 없음, 도움말 한 줄).

### U3 실측 (2026-10-23, `codex-cli 0.163.0`, Windows 11)

`codex exec -C <scratch>/inside -s read-only --skip-git-repo-check --json "<inside 밖 canary.txt 를 셸로 읽고, 현재 폴더에 w.txt 를 만들어 보라>"`:

| 시도 | 결과 |
| --- | --- |
| `-C` 밖 파일 읽기(`[IO.File]::ReadAllText('…/outside/canary.txt')`) | **성공** — `CANARY-7f3a91` 출력, 종료 0 |
| `-C` 안 쓰기(`New-Item ./w.txt`) | 실패 — `Access to the path '…\inside\w.txt' is denied.`(샌드박스가 쓰기는 막음) |
| 사용자 전역 설정 · 스킬 | 실행이 `~/.agents/skills/claude-compat-bootstrap` 을 따라 `C:\Users\tskim\.claude\CLAUDE.md` · settings 를 읽음 — 그 PC 사용자의 Codex 설정 · 스킬이 원격 실행에 그대로 걸린다(`--ignore-user-config` 효과는 미확인) |
| 셸 | `pwsh.exe -Command …` — Codex 는 읽기도 셸 명령으로 한다(Claude 의 "셸 끔" 꼴이 없다) |

8. (U7 — 2026-10-23 사용자) "권장안의 --ignore-user-config 붙이지 말것. --strict-mcp-config 옵션도 제거. 에이전트가 응답하는 환경에서는 로컬 에이전트가 동작하는 것과 최대한 맞춤.
   편집, 삭제만 제외." → Codex 실행에 `--ignore-user-config` 없음(그 PC 사용자의 Codex 설정 · 스킬 · MCP 그대로). 편집 · 삭제는 `-s read-only`(쓰기 막힘 — 실측) 로 뺀다.
9. (같은 지시 · 2026-10-23 사용자 "로컬과 맞춤 (편집 도구만 막음)" 선택) Claude 실행 인자 개정 — D22 개정:
   - 뺀다: `--restricted`(사용자 · 프로젝트 설정 무시 · 파일 도구 폴더 가둠) · `--strict-mcp-config` · `--tools`.
   - 남긴다: `-p --output-format stream-json --verbose --permission-prompts none --permission-mode dontAsk` · `--append-system-prompt`(가드 + 추가 지시) · `--system-prompt-snapshot off` ·
     세션 · `--model` · `--effort` · 상위 읽기 `--add-dir`.
   - 미리 허용 `--allowedTools Read,Grep,Glob,WebFetch,WebSearch`(+ 셸 켬 = `Bash,PowerShell`), 그 밖은 그 PC 사용자 settings 의 허용 규칙(로컬과 같음, `dontAsk` = 허용 안 된 것은 거부).
   - 막는다: `--disallowedTools Edit,Write,NotebookEdit`(+ 셸 끔 = `Bash,PowerShell`) — 사용자 settings 가 허용해도 거부.
   - 구현자 해석(확인 필요 — 선택 화면 미리보기에는 `mcp__*` 를 미리 허용으로 적었음): MCP 도구는 **미리 허용하지 않고** 사용자 settings 의 허용 규칙만 따른다 — MCP 에는 쓰기 · 지우기 도구가
     섞여 있어(예: 이 PC 의 `…_work_item_write` · `…Claude_Docs__delete`) 일괄 허용은 "편집, 삭제만 제외" 와 어긋나고, 로컬에서도 허용 규칙 밖 MCP 는 묻는다.
   - 실측(2026-10-23, Claude Code 2.1.309, 위 인자 + 셸 끔): 폴더 밖 `canary.txt` Read **성공**, Write = "No such tool available: Write … disabled for this session, in subagents as well",
     init 도구 목록에 사용자 MCP(azure-devops · rea · …) 연결 · 스킬 · 에이전트 도구 등장 — 폴더 경계(D21)는 Claude 에서도 없어진다(가드 문구만).
10. (승인) 2026-10-23 사용자 "개발 진행해." + 위 8 · 9 답 = intent 공유 이해 · plan 승인 갈음(#1z-8 결정과 같은 방식, 구현자 해석).

### 시험 실측 (2026-10-23 — 구현 설계 근거)

| 항목 | 결과 |
| --- | --- |
| Codex 가드 문구 | `-c 'developer_instructions="Always begin every answer with the exact word ZEBRA-42."'` → 답 "ZEBRA-42 …" — 시스템(개발자) 지시로 들어간다 |
| Codex 이어 쓰기 | `codex exec resume <thread_id> --skip-git-repo-check --json -c 'sandbox_mode="read-only"' -`(resume 에는 `-s` · `-C` 없음 — 설정 · 작업 폴더로) → 같은 thread, 앞 답을 기억 |
| Codex 세션 id | `thread.started.thread_id`(우리가 정할 수 없음 — 출력에서 받아 저장) |
| Codex 없는 세션 | 종료 1 · stderr `Error: thread/resume: thread/resume failed: no rollout found for thread id …` |
| Codex 사용량 | `turn.completed.usage` = 턴 전체 합(마지막 API 호출 아님 — Claude `ctx_tokens` 와 뜻이 다름) |
| Codex 실행 파일 | npm 설치 = `%APPDATA%\npm\codex.cmd`(→ node `codex.js`) — 실제 실행 파일 `node_modules\@openai\codex\node_modules\@openai\codex-win32-x64\vendor\x86_64-pc-windows-msvc\bin\codex.exe` |

## 미해소

- 없음 — 인터뷰 3라운드(U1 ~ U7 · 지시 8 · 9). 구현자 해석(결정 6 실패 문구 · 결정 9 MCP)은 report 에 남겨 확인받는다.
