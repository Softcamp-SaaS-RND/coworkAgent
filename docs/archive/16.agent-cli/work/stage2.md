# stage2 — CLI(`peers` · `ask` · `say` · `history`) · 릴리스 파이프

## 목표

[plan](../plan.md) Stage 2 — 같은 실행 파일의 CLI 하위 명령이 떠 있는 주 인스턴스의 로컬 API 를 부른다. 대상 gate: E6 · E7.

## 산출 파일

- `crates/app/src/cli.rs`(신규) — 인자 해석 · `<상대>` 해석(이름 · 지문 앞자리) · 4개 명령 · 답 대기 · 종료 코드 상수
  (`EXIT_USAGE` 2 · `EXIT_NOT_RUNNING` 3 · `EXIT_API` 4 · `EXIT_REPLY_FAILED` 5 · `EXIT_TIMEOUT` 6) · `run`(테스트 진입) · `main`(실행 파일 진입)
- `crates/app/src/handoff.rs` — `local_request(info, method, path, body, deadline) -> (상태, 본문)` 로 일반화, `request_open_within` 은 그 위에
- `crates/app/src/main.rs` — `main()` 맨 앞(`Tray::new()` 전)에서 첫 인자가 하위 명령이면 `cli::main` 종료 코드로 `exit`
- `crates/app/src/lib.rs` — `pub mod cli`
- 테스트: `crates/app/tests/cli.rs`(13건 — plan E6 목록 그대로) · `cli.rs` 단위 2건(`utc_time_known_dates` · `fingerprint_query_needs_hex_only`)

## 설계 결정

- CLI 분기는 트레이 · 이전 데이터 이어받기 · 로거 · 잠금보다 앞 — 실행 중이 아니어도 주 인스턴스가 되지 않는다(intent 결정 "오류로 끝낸다").
- 재시도 없음 — `instance.json` 이 없거나 형식 오류 · 연결 거부(`ConnectionRefused` · `Reset` · `Aborted` · `AddrNotAvailable`)면 바로 3.
  그 밖 요청 오류(시간 초과 등)는 4.
- HTTP 클라이언트 의존을 더하지 않았다 — `handoff` 의 원시 HTTP/1.1(`Connection: close`, 끝까지 읽기, 전체 `deadline`)을 넓혔다.
  기존 핸드오프 테스트(`handoff` 2 · `handoff_deadline` 1 · `handoff_failure` 1) 그대로 통과.
- `<상대>`: 페어링된 피어 중 이름 완전 일치(대소문자 무시) 또는 지문 hex 앞자리(4자 이상, 공백 · `-` · `:` 무시, hex 아닌 글자가 섞이면
  지문으로 보지 않음). 이름 · 지문이 서로 다른 피어에 걸리면 모호(2) — 후보를 `이름 (지문 8자)` 로 보여 준다.
- `ask --conv` 는 내가 연 대화(`out`)만 — 상대가 연 대화는 내 에이전트에게 묻는 대화라 2. `--agent` 가 대화 에이전트와 다르면 2.
- 답 대기: 1초 간격(남은 한도보다 길게 자지 않는다) — `reply_to == 질문 로컬 id` 인 에이전트 답 `done` 이면 성공, `error` 면 5(답 본문을
  사유에), 질문이 `error` · `undelivered` 면 5, 한도(30분)를 넘으면 6 + `coworkAgent history --conv {id}` 안내.
- `say` 는 대화 목록(최근 활동 순)의 그 상대 첫 대화 — plan 편차 3.
- `history --conv` 의 발화자는 웹UI `authorLabel` 과 같은 규칙(보는 쪽 기준), 시각은 `YYYY-MM-DD HH:MM UTC`(시간대 의존 없음,
  날짜 계산은 단위 테스트로 고정). 목록 제목의 탭 · 줄바꿈은 공백으로.
- `--json` 은 API JSON 그대로(`ask` = `{conversation, asked, reply}`, `--no-wait` 면 `reply` 없음).

## 테스트 결과

- `cargo test --workspace` — `passed=154 failed=0`(test result 줄 합산, Stage 1 뒤 139 + cli 13 + 단위 2).
- `cargo test -p coworkagent-app --test cli` — `13 passed`(E6).
- `cargo clippy --workspace --all-targets -- -D warnings` — 처음 `ptr_arg` 1건(테스트 `&PathBuf`) → `&Path`, 강제 재검사 `Finished`.
- `node --test …` — `# fail 0` · `cargo fmt --check` 통과.
- E7(개정 2 CHECK) — `rc=0 piped=True`: 콘솔 없는 릴리스 실행 파일을 파이프로 부르면 개발 PC 에서 실행 중인 트레이 앱의 `peers` 결과가
  stdout 으로 온다. `target/release/coworkAgent.exe` 는 그 트레이 앱이 잡고 있어 덮어쓰지 못했다(`os error 5`) — plan 개정 2.
- mutation 4종 모두 FAIL: 연결 거부를 4 로 → 1 failed · `say` 가 가장 오래된 대화 → 1 failed · `ask` 가 `via` 를 빼먹음 → 2 failed ·
  답 `error` 를 성공으로 → 1 failed.
- RED: `tests/cli.rs` 를 먼저 쓰고 `cli` 모듈 없이 시작(컴파일 실패 단계는 실행 기록을 남기지 않았다 — 위 mutation 으로 단정의 힘을 대신 확인).

## 리뷰

분리된 리뷰 에이전트(`a95ab51..d5a276a`) — high 0, medium 3, low 6. 코드로 확인 후:

| 지적 | 판단 | 조치 |
|---|---|---|
| `ask --conv` 테스트가 답 본문을 안 본다(앞선 답을 돌려줘도 통과) | 사실 | `reply.text == "답: 둘"` 단정 — `reply_to` 조건 제거 mutation 이 FAIL |
| 본문이 `--` 로 시작하면 옵션으로 해석 · `--` 구분자 없음 | 사실 | `--` 뒤는 전부 인자(사용법에 안내) + 테스트 |
| 전송 오류 502 에 대화 ID 가 없어 재시도하면 두 번 실행될 수 있다 | 사실 | 502 · POST 면 "전달됐을 수 있다 — history 로 확인" 안내. 대화 ID 를 오류에 싣는 것은 `Chat::send` 오류 계약 변경이라 하지 않음(원격 실행은 D31 읽기 전용) |
| `env::args()` 가 비-UTF-8 인자에서 패닉(트레이 시작까지) | 사실 | `args_os` + 손실 변환 |
| 남은 포트를 다른 프로그램이 쓰면 4 | 사실 | `local_request` 의 "HTTP 아님" 을 `InvalidData` 로, CLI 는 3 + 테스트(FIN 으로 닫는 가짜 서버). 처음엔 매치 가지 순서가 틀려 테스트가 잡았다 |
| 2xx 본문이 JSON 이 아니면 조용히 `Null` | 사실 | 4 + 사유 |
| 대기 중 폴링 실패 시 `history --conv` 안내 없음 | 사실 | 안내 추가 |
| 응답 읽는 중 `ConnectionReset` 이 3 으로 분류 | 사실(드묾) | 미조치 — 연결 직후 RST 와 구분할 근거가 없고, 앱이 끊은 것이면 "실행 중 아님" 판단도 틀리지 않다 |
| stdout 쓰기 실패가 4 | 사실 | 미조치 — 파이프가 끊기면 받을 사람이 없다. 종료 코드 표에 새 값을 두지 않는다 |
| `emit` 은 `line` 별칭(nit) | 사실 | 제거 |

반영 뒤: `passed=154 failed=0` · clippy `Finished`(처음 `needless_borrows` 3건) · `cargo fmt --check` · gate PASS(E8 ABANDON).

## 다음 단계

구현 완료 — 적대적 평가(`eval.md`): `python tools/check_gates.py docs/tasks/16.agent-cli` 로 평가 원장, E8(실PC · 실제 claude) 은 D28 에 따라
`ABANDON` + 19 TC 추가, 그 뒤 codex 적대적 평가(라운드 상한 2).
