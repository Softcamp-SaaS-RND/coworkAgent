# plan — 16.agent-cli

> 근거: [intent](intent.md)(승인 2026-10-12 — 공유 이해 도달). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `1bf681b`(intent 승인). plan 승인: 2026-10-12(사용자 "plan 승인").

## 목표

사용자의 AI 에이전트가 파이프로 부르는 CLI(`peers` · `ask` · `say` · `history`) — 이미 떠 있는 주 인스턴스의 로컬 API 를 쓰고,
CLI 로 나간 글은 양쪽 PC 이력 · 대화 화면 · 타임라인에 "AI 에이전트(CLI)" 로 구분돼 남는다.

### 성공 기준(intent 사본)

- A1: `peers` 로 피어 목록을 조회하면 등록된 사용자 · 연결 상태가 텍스트로, `--json` 이면 JSON 으로 나온다.
- A2: `ask` 로 상대 에이전트에게 질문하면 답이 올 때까지 기다려 stdout 으로 돌려주고, 대화 ID 를 주면 같은 대화를 이어 쓴다.
  즉시 반환 옵션이면 대화 ID 만 돌려준다. 실패 · 시간 초과는 0 이 아닌 종료 코드와 사유를 낸다.
- A3: `say` 로 상대 사람에게 실행 없는 글을 남길 수 있다.
- A4: `history` 로 대화 목록과 한 대화의 이력(타임라인)을 조회한다.
- A5: CLI 로 나간 글은 내 PC · 상대 PC 이력에 발화자 "내 AI 에이전트(CLI)" 로 저장되고 웹UI 대화 화면 · 타임라인에서 사람 글과 구분돼 보인다.
- A6: 파이프로 CLI 를 부르는 AI 에이전트(릴리스 빌드, 콘솔 없는 Windows 서브시스템)가 stdout · 종료 코드를 받는다.
- A7: coworkAgent 가 실행 중이 아니면 CLI 는 "coworkAgent 가 실행 중이 아니다" 사유와 0 이 아닌 종료 코드로 끝난다.
- A8: 로컬 API 는 잠금 파일 bearer 토큰 없이는 CLI 요청을 거부한다(웹UI 와 같은 신뢰 경계).

### intent 결정 · 제외(사본)

- 명령 literal(바꾸지 않는다):

  ```text
  coworkAgent peers [--json]
  coworkAgent ask <상대> "<질문>" [--agent <이름>] [--conv <대화ID>] [--no-wait] [--json]
  coworkAgent say <상대> "<글>" [--conv <대화ID>] [--json]
  coworkAgent history [<상대>] [--conv <대화ID>] [--json]
  ```

  `<상대>` = 피어 표시 이름 또는 지문 앞자리, 겹치면 오류.
- 답까지 대기(실행 시간 제한 30분, D24) · 텍스트 기본 · `--json` · `--no-wait`. 앱이 안 떠 있으면 오류로 끝(앱을 띄우지 않는다).
- 같은 OS 사용자의 로컬 프로세스 = 웹UI 와 같은 신뢰 경계(수용한 위험), CLI 호출은 전부 이력에 남는다.
- 제외: `coworkAgent chat [--peer]`(S8 · Phase 3) · 웹UI 디자인 개편(15) · 원격 권한 확대(D31 그대로) · 앱 없이 단독 동작.

## intent 대비 해석 · 편차 (plan 승인 대상)

코드 사실로 intent 문구와 어긋나거나 literal 이 비워 둔 자리 — 이 plan 승인으로 확정한다.

1. **발화자 구분 수단 — `author` 값 추가 대신 별도 표시 `via`**(intent `## 결정` "피어 프로토콜 `author` 값 추가" 와 다름).
   이유: `author` 는 동작을 가른다 — `fill_mine_exec` 의 `exec = (author = 'human')`, 대기열 · 제목 조회의 `author = 'human'` 조건
   (`crates/core/src/chat.rs`), 받은 쪽 `inbound_post` 의 `match author` 에서 **모르는 값은 `bad_request` 로 거절**한다
   (`crates/agent/src/chat.rs`). 새 `author` 값이면 이전 버전 상대에게 CLI 글이 거절된다. `via` 는 `#[serde(default)]` 선택 필드라
   (`proto.rs` 에 `deny_unknown_fields` 없음) 이전 버전 상대는 무시하고 사람 글로 보이며, 새 버전은 `via` 없는 글을 사람 글로 본다.
   CLI 글의 동작(실행 요청 · 메모 · 자동 왕복 멈춤)은 사람 글과 같다.
2. **라벨 관점** — 저장 값은 양쪽 `via = 'cli'` 하나, 라벨은 보는 쪽 기준: 내 PC `내 AI 에이전트(CLI)`, 상대 PC
   `{상대 이름}/AI 에이전트(CLI)`(기존 `{peer}/{에이전트}` 표기와 같은 꼴). A5 의 "내 AI 에이전트(CLI)" 는 내 PC 라벨이다.
3. **`say` 에 `--conv` 가 없을 때** — 그 상대와의 **가장 최근 대화**(보낸 · 받은 무관)에 쓴다, 대화가 없으면 오류(종료 코드 2).
   이유: 상대 PC 는 처음 보는 대화 uuid 의 `post` 를 `unknown` 으로 거절한다 — 대화는 에이전트 실행 요청(`message`)으로만 열린다.
   기각안: `--conv` 필수(literal 의 `[ ]` 위반) · 프로토콜 변경(범위 밖).
4. **`ask` 에 `--agent` 가 없을 때** — 새 대화면 상대 에이전트가 정확히 1개일 때 그것, 아니면 이름 목록과 함께 오류(종료 코드 2).
   `--conv` 가 있으면 그 대화의 에이전트(`--agent` 와 다르면 오류).
5. **`peers` 의 "연결 상태"** = 페어링 상태(`paired` · `my_ok` · `their_ok`). 온라인 여부(상대마다 ping)는 넣지 않는다 — 상대 수만큼
   지연이 생기고 literal 에 없다.
6. **대화형 콘솔** — 릴리스는 `windows_subsystem = "windows"`. 실측(2026-10-12, 같은 서브시스템의 시험 exe): 파이프로 부르면
   stdout · stderr · 종료 코드가 온다(`'hello-out\n' 'hello-err\n' 3`). 사람이 콘솔에서 직접 치면 셸이 기다리지 않고 출력도 보장되지
   않는다 — 이 Task 는 파이프(AI 에이전트) 경로만 보장한다. 사람용은 `coworkAgent peers | more` 처럼 파이프로 감싸 쓴다.

## 설계

### 저장(스키마 8, `an-core`)

```sql
ALTER TABLE messages ADD COLUMN via TEXT;   -- 'cli' = 사용자 AI 에이전트가 CLI 로 쓴 글, NULL = 웹UI(사람)
```

- `Message.via: Option<String>` · `MSG_COLS` 에 `via`. 쓰기: `add_human(..., via)` · 받은 글 저장 경로에 `via` 전달.
- 이전(v < 8) — 기존 규칙(한 트랜잭션 · 있는 열 건너뛰기 · `{db}.v7.bak` 사본). `paths` 테스트에 7 → 8 이전 1건 추가.

### 프로토콜(`an-net`)

- `Request::Message` · `Request::Post` 에 `#[serde(default)] via: Option<String>`. 받은 쪽은 `Some("cli")` 만 저장, 그 밖 값은 무시(NULL).

### 조율(`an-agent`)

- `Chat::send(peer_id, agent, conversation, text, via)` · `Chat::post(conversation_id, text, to_agent, via)` — `via` 를 내 이력과
  `message` · `post` 요청에 싣는다. 자동 왕복 시작(`start_relay`)은 CLI 범위 밖이라 그대로.

### 로컬 API(`an-app`)

- `POST /api/conversations` · `POST /api/conversations/{id}/messages` 본문에 `via`(선택, `"cli"` 만 허용 — 그 밖 400).
- `message_json` 에 `uuid` · `via` 추가(대기 판정이 `reply_to` = uuid 를 대조한다).
- 인증은 기존 `guard` 그대로 — `Authorization: Bearer {instance.json token}` · `Host: 127.0.0.1:{port}`. 토큰이 틀리면 401(A8).

### CLI(`an-app` `cli.rs` + `main.rs`)

- `main()` 맨 앞(`Tray::new()` 전)에서 첫 인자가 `peers` · `ask` · `say` · `history` 면 `cli::main(args)` 결과 코드로 `exit`.
  잠금 · 로거 · 이전 데이터 이어받기 · 트레이를 거치지 않는다(주 인스턴스가 되지 않는다).
- `cli::run(args, instance_file, out, err, wait_limit) -> i32` — 테스트가 임시 경로 · 짧은 대기 한도로 부른다. `cli::main` 은
  `Paths::default_root()` 의 `instance_file()` · 30분으로 `run` 을 부른다.
- 요청: `handoff.rs` 의 원시 HTTP 를 일반화 — `local_request(info, method, path, body) -> (status, body)`(`Connection: close`, 끝까지
  읽기). `request_open` 은 이 위에 다시 얹는다. 재시도 없음 — `instance.json` 이 없거나 연결 거부면 바로 "실행 중이 아니다".
- `<상대>` 해석: 페어링된 피어 중 이름 완전 일치(대소문자 무시) 또는 지문 hex 앞자리(4자 이상, 공백 · `-` 무시) 일치가 정확히 1개.
- `ask` 대기: 1초마다 `GET /api/conversations/{id}/messages` — 끝 = `reply_to == 보낸 글 uuid` 인 에이전트 답이 `done`(성공) ·
  `error`(실패), 또는 보낸 글이 `error` · `undelivered`(실패). 한도(30분) 넘으면 시간 초과.
- 출력(텍스트): `peers` 한 줄에 하나 `{이름}\t{호스트}\t{지문 앞 8자}\t{페어링됨|대기}` · `ask` 답 본문(`--no-wait` 면 대화 ID) ·
  `say` 대화 ID · `history` 목록 `{대화ID}\t{상대}\t{에이전트}\t{제목}\t{마지막 상태}`, `--conv` 면 `[{시각}] {발화자}: {본문}`.
  `--json` 은 같은 내용을 API JSON 그대로(`ask` = `{conversation, asked, reply}`). 오류 사유는 stderr.
- 종료 코드:

  | 코드 | 뜻 |
  |---|---|
  | 0 | 성공 |
  | 2 | 사용법 · 인자 오류(모르는 명령 · 상대 없음 · 상대 · 에이전트 모호 · `say` 할 대화 없음) |
  | 3 | coworkAgent 가 실행 중이 아니다(`instance.json` 없음 · 형식 오류 · 연결 거부) |
  | 4 | 로컬 API 거부(401 토큰 · 4xx · 5xx — 페어링 안 됨 · 상대 거절 포함) |
  | 5 | 답 실패(에이전트 실행 오류 · 상대에 못 보냄) |
  | 6 | 대기 시간 초과 |

### 웹UI

- `view.js` `authorLabel` — `m.via === "cli"` 면 내 글 `내 AI 에이전트(CLI)`, 상대 글 `{peer}/AI 에이전트(CLI)`. 대화 화면 `kind` 칸
  `AI 에이전트(CLI) → 에이전트` · `AI 에이전트(CLI)`, 타임라인(C) 같은 라벨. node 테스트 추가.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 스키마 8 · 프로토콜 `via` · `Chat` · API 필드 · 웹UI 라벨 | 테스트(paths · cli_origin · view.test) | E3 · E4 · E5 |
| 2 | CLI(`cli.rs` · `local_request` · `main` 분기) · 릴리스 파이프 | 테스트(cli) · 릴리스 빌드 | E6 · E7 |
| 공통 | 검증 명령 | — | E1 · E2 · E8 |

## 평가 항목

- [ ] E1: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A5 — 화면 로직 node 테스트(CLI 라벨: 내 글 · 상대 글 · 타임라인)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A5 — 스키마 8 이전(paths 9 → 10건: 7 → 8 이전 · via 열)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 10 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A5 — 양쪽 이력 via(cli_origin 3건: ask_via_cli_marks_both_sides · say_via_cli_marks_both_sides · payload_without_via_is_human)
  CHECK: cargo test -p an-agent --test cli_origin
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A1·A2·A3·A4·A7·A8 — CLI(cli 16건: peers_lists_paired_with_json · peer_name_ambiguous_errors · ask_new_conversation_waits_for_reply · ask_conv_continues_same_conversation · ask_no_wait_prints_conversation_id · ask_agent_ambiguous_errors · ask_remote_failure_exit_code · ask_timeout_exit_code · say_defaults_to_latest_conversation · say_without_conversation_errors · history_lists_and_shows_timeline · not_running_exits_with_code · bad_token_rejected · unknown_command_exe_exits_usage · old_primary_refused_before_send · post_lost_response_is_not_not_running)
  CHECK: cargo test -p coworkagent-app --test cli
  EXPECT: test result: ok. 16 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A6 — 릴리스 빌드(콘솔 없는 서브시스템)를 파이프로 부르면 출력과 종료 코드가 온다(인스턴스 유무와 무관 — 실행 중이면 rc 0, 이전 버전이 실행 중이면 rc 4, 아니면 rc 3 — 0 이 아니면 stderr 사유)
  CHECK: cargo build --release -p coworkagent-app --target-dir target/gate && python -c "import subprocess,os;r=subprocess.run([os.path.abspath('target/gate/release/coworkAgent.exe'),'peers','--json'],capture_output=True);print(f'rc={r.returncode} piped={bool(r.stdout.strip()) if r.returncode==0 else bool(r.stderr.strip())}')"
  EXPECT: /rc=(0|3|4) piped=True/
  EVIDENCE: pending
- [ ] E8: A2·A3·A5 — 실PC 두 대 · 실제 claude(수동): AI 에이전트가 `ask` · `say` 를 파이프로 부르고 양쪽 대화 화면 · 타임라인에 CLI 라벨 확인
  EVIDENCE: pending

## 종결 시 할 일

- E8 은 D28 에 따라 `ABANDON`(사유: D28) + 19.prototype-acceptance 에 TC 추가.
- 매뉴얼 `an_core` · `an_net` · `an_agent` · `an_app`(CLI 사용법 · 종료 코드) · PRODUCT(CLI · 스키마 8) · masterPlan D14 에 CLI literal
  환류(D14 TODO 중 CLI `<상대>` 해석 닫기).

## 개정 기록

- 개정 1(2026-10-12, Stage 1 구현 — 설계 문구만, 평가 항목 · EXPECT 변경 없음): ①`Chat::send` · `post` 시그니처 대신
  `send_via` · `post_via` 추가(기존 호출 유지) ②`message_json` 에 `uuid` 를 넣지 않는다 — `reply_to` 는 uuid 가 아니라 질문의 로컬 id
  라서 대기 판정은 `reply_to == 보낸 글 id`. ③`add_human(..., via)` 대신 삽입 뒤 `set_via` — 같은 트랜잭션 안에서(stage 리뷰 반영).
  ④자동 왕복 시작(`relay_agent`)에 `via` 를 주면 400(조용히 버리지 않는다 — stage 리뷰 반영).
- 개정 2(2026-10-12, Stage 2 — E7 CHECK 보정, EXPECT 변경 없음): 릴리스 빌드를 `--target-dir target/gate` 로 — 개발 PC 에서
  `target/release/coworkAgent.exe` 가 트레이 앱으로 실행 중이면 덮어쓰지 못해(`os error 5`) 빌드가 실패한다. 같은 릴리스 프로필 ·
  같은 서브시스템이라 판정 대상은 같다.
- 개정 3(2026-10-12, 적대적 평가 라운드 1 반영 — **E6 EXPECT 변경 13 → 15, 사용자 승인 2026-10-12**): ①인자가 하나라도 있으면 CLI —
  모르는 명령도 트레이로 새지 않고 2(실행 파일로 단정 `unknown_command_exe_exits_usage`). D14 `chat`(S8)은 착수 때 이 분기에 더한다.
  ②보내기 전 `/api/status` 의 `cli_api`(= 1)로 주 인스턴스가 CLI 출처 표시를 아는지 확인 — 이전 버전 · 다른 프로그램이면 아무것도
  보내지 않고 4(`old_primary_refused_before_send`).
  ③**E7 EXPECT 변경 `/rc=(0|3) piped=True/` → `/rc=(0|3|4) piped=True/`(사용자 승인 2026-10-12)** — ②로 개발 PC 에 이전 버전 트레이 앱이 떠
  있으면 rc 4 + stderr 사유가 정답이다(재측정 실측 `rc=4 piped=True`). 판정 대상(파이프로 출력 · 종료 코드가 온다)은 그대로.
  ④**E6 EXPECT 변경 15 → 16(사용자 승인 2026-10-12)** — 적대적 평가 라운드 2 반영: 요청을 다 보낸 뒤 응답 전에 끊기면 `local_request` 는
  `UnexpectedEof`, CLI 는 POST 면 3 이 아니라 4 + "전달 여부를 모른다 — 다시 보내지 말고 history 로 확인"(`post_lost_response_is_not_not_running`).
