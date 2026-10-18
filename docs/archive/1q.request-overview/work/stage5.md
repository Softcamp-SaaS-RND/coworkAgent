# stage5 — CLI 도움말 `coworkAgent /?`(plan 개정 1)

## 목표

intent 개정(사용자 지시 "`coworkAgent.exe /?` … cli 명령어 출력 기능 병행 개발. readme 에도 …") — A9 · A10, 결정 15. Stage 2 와 3 사이에 병행.

## 산출 파일

- `crates/app/src/cli.rs` — `is_help`(`/?` · `-h` · `--help`, 첫 인자만) · `HELP_HEAD` · `HELP_TAIL` · `help()`(= 머리 + `USAGE` + 출력 형식 · 종료 코드 · 연동 예) ·
  `run` 첫머리에서 표준 출력 · 0(인스턴스 파일 · 앱 불필요).
- `crates/app/tests/cli.rs` — `help_exe_prints_commands`(실제 실행 파일 · 파이프).
- `README.md` — 기능 표 "AI 에이전트용 CLI" 에 `/?` · 새 절 "다른 AI 도구와 연동"(명령 예 · AI 도구 지시 예 · `| more`).
- [사용자 가이드](../../../manual/user/user-guide.md) §14 — `coworkAgent /?` 줄 · 설명.

## 설계 결정

- 첫 인자만 도움말 — `coworkAgent ask user-b "/?"` 는 그대로 묻는다.
- Windows 릴리스는 콘솔 없는 앱이라 표준 출력은 파이프 · 리디렉션일 때만 보인다 — AI 도구는 파이프로 읽으므로 그대로, 사람에게는 `| more` 안내.
  콘솔 붙이기(AttachConsole)는 기존 CLI 전체 동작을 바꿔 이 개정 밖(사용자 확인 후보). 릴리스 빌드 콘솔 직접 실행은 확인하지 않았다(이론 근거).

## 테스트 결과

- RED: `cargo test -p coworkagent-app --test cli help_exe` → `assertion left == right failed: /?` `left: Some(2)`(모르는 명령 = 2).
- GREEN: 같은 명령 → `test result: ok. 1 passed`. 디버그 실행 파일 `coworkAgent.exe /?` → 머리 · 사용법 · 출력 · 종료 코드 · 연동 예 · 도움말, `rc=0`.
- `python tools/check_readme.py` → `readme ok` · README `coworkAgent /?` 5곳.

## 다음 단계

stage3 — 주고받은 메시지.
