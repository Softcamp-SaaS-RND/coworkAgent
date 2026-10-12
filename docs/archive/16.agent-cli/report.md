# report — 16.agent-cli

종결: 2026-10-12 · 브랜치 `task/16-agent-cli` → main

## 결과

사용자의 AI 에이전트가 파이프로 부르는 CLI — `coworkAgent peers` · `ask` · `say` · `history`. 떠 있는 주 인스턴스의 로컬 API 를
`instance.json` bearer 토큰으로 부르고(웹UI 와 같은 신뢰 경계), 앱을 대신 띄우지 않는다. CLI 로 쓴 글은 `via = 'cli'` 로 양쪽 PC 이력에
남고 웹UI 대화 화면 · 타임라인에 `내 AI 에이전트(CLI)` · `{상대}/AI 에이전트(CLI)` 로 사람 글과 구분돼 보인다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 `peers` 텍스트 · `--json` | 충족 | E6 `peers_lists_paired_with_json` |
| A2 `ask` 답 대기 · 이어 쓰기 · `--no-wait` · 실패 · 시간 초과 코드 | 충족 · 실PC ABANDON | E6(ask 6건) · E8(D28) |
| A3 `say` 실행 없는 글 | 충족 · 실PC ABANDON | E6 `say_*` 2건 · E8 |
| A4 `history` 목록 · 타임라인 | 충족 | E6 `history_lists_and_shows_timeline` |
| A5 양쪽 이력 CLI 출처 · 화면 구분 | 충족 · 실화면 ABANDON | E3 · E4 · E5 · E8 |
| A6 콘솔 없는 릴리스를 파이프로 | 충족(거절 경로) · 성공 경로 19 이관 | E7 `rc=4 piped=True`(개정 3 ③) · 측정 `24c52b6` `rc=0 piped=True` |
| A7 실행 중이 아니면 3 | 충족 | E6 `not_running_exits_with_code`(파일 없음 · 닫힌 포트 · 비-HTTP 포트) |
| A8 토큰 없으면 거부 | 충족 | E6 `bad_token_rejected` |

- 적대적 평가 2 라운드(상한) — high 3 · medium 1. 반영: 모르는 명령은 트레이로 새지 않고 2 · 보내기 전 `/api/status` `cli_api`
  확인(이전 버전 주 인스턴스면 아무것도 보내지 않고 4)(라운드 1), 보낸 뒤 응답 유실은 3 이 아니라 4 + 재전송 금지 안내(라운드 2).
  미조치 3건(E8 · idempotency key · E7 성공 경로)은 [eval.md](eval.md) → 원장.
- stage 리뷰 2회(분리된 리뷰 에이전트) — `via` 기록 원자화 · 자동 왕복 `via` 거절 · `--` 구분자 · `args_os` 등 반영.
- plan 개정 3건: 1(설계 문구 — `send_via`/`post_via` · `reply_to` 는 로컬 id) · 2(E7 CHECK `--target-dir target/gate`) · 3(E6 13 → 16 ·
  E7 rc 4 허용 — 사용자 승인 2026-10-12).

## 산출

- `an-core`: 스키마 8(`messages.via`) · `Message.via` · `set_via` · `VIA_CLI`.
- `an-net`: `Request::Message` · `Post` 의 선택 필드 `via`(이전 버전 상대는 무시).
- `an-agent`: `Chat::send_via` · `post_via`(받은 글의 `via` 저장, 글 · 출처 · 순서 한 트랜잭션).
- `coworkagent-app`: `cli`(명령 4개 · 종료 코드 0/2/3/4/5/6) · `handoff::local_request` · `main` 의 CLI 분기(인자가 있으면 CLI) ·
  `/api/status` `cli_api` · API 본문 `via` · 웹UI 라벨(`authorLabel` · `kindText` · 타임라인).

## 남은 것

- 실PC · 실제 claude 확인과 릴리스 CLI 의 성공 경로 — `19.prototype-acceptance` TC-J(D28).
- 재전송 중복 방지 키(idempotency key) — #11 R1-F2k 와 함께 Phase 2 재전송에서.
- `coworkAgent chat [--peer]`(S8) — "인자가 있으면 CLI" 분기에 더한다.
