# report — 1b.inbound-capacity

종결: 2026-10-13 · 브랜치 `task/1b-inbound-capacity` → main

## 결과

받는 PC 의 `claude -p` 동시 실행을 **PC 전체 상한**(기본 2 · 1 ~ 8, 이 PC 설정)으로 묶었다. 넘는 요청은 **거절하지 않고** 실행 문(FIFO)에 닿은
순서로 기다리고, 차례가 온 **뒤의** 대화 · 에이전트 · 멈춤 상태로 실행한다(기다리는 동안 지운 에이전트는 실행하지 않음). 상대가 보낸 실행 요청이면
보낸 쪽에 새 프레임 `Progress` 로 차례를 알린다 — 웹UI 글 아래 "상대 PC 에 처리 중인 작업이 많아 응답을 기다립니다(앞에 k건)" → "처리 중", CLI `ask`
는 표준 오류에 진행 줄을 쓰고 대기 중에는 30분을 세지 않는다. 같은 대화의 앞 실행을 기다리는 후속 요청도 닿자마자 대기로 알리고 실행 목록에
넣는다. 받는 쪽은 실행 상세 맨 위 "이 PC 실행"(실행 중 · 대기 목록)과 설정 "동시 실행 상한" 을 본다. **포트**: 47100 을 남이 잡으면 임의 포트로
받고 잡은 프로세스(이름 · PID)를 트레이 툴팁 · 내 정보에 보인다 · 페어링된 상대는 `Ping{port}` 로 바뀐 포트 · IP 를 승인 초기화 없이 따라온다 ·
내 정보에 웹UI 포트 · 피어 포트(고정 | 임의) · 열린 피어 연결 수 · 강제 종료 뒤 5초 안에 두 포트가 풀리고 남은 `instance.json` 으로 다시 띄우면
47100 을 연다(실측) · 앱이 연 수신 소켓은 `claude -p` 자식에 물려가지 않는다.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 상한 2 에서 대화 5건 → 실행 2 · 대기 3 · 도착 순서 | 충족 | E4 gate 단위 5 · E5 `five_conversations_respect_cap_in_arrival_order`(최대 동시 2 · 기다린 셋 순서) |
| A2 30건 모두 대기 · 모두 답 · 거절 0 | 충족 | E5 `thirty_requests_all_answered_without_refusal`(진행 알림 폭주로 29/30 이던 결함을 상대별 알림 줄로 고침) |
| A3 보낸 쪽 "응답을 기다립니다(앞에 k건)" → "처리 중" | 충족 | E5 `waiting_request…` · `same_conversation_followup…` · E6 `message_json_carries_progress` · E3 `progressText` · `messagesSigOf` · E11 실화면 |
| A4 CLI 대기 · 처리 중 출력(`--json`) · 대기 때문에 포기하지 않음 | 충족(미조치 1) | E6 `cli_prints_progress_plain_and_json` · `cli_wait_clock_pauses_while_queued` — 상대가 대기 중 사라지면 무기한(미조치 · 사용자 결정) |
| A5 받는 쪽 실행 중 · 대기 목록 · 상한 1 ~ 8 저장 | 충족 | E6 `runs_lists_running_and_waiting` · `put_runs_bounds_and_saved` · E5 `cap_bounds_and_queue_snapshot` · E11 |
| A6 47100 잡은 프로세스 이름 · PID 표시 | 충족(웹UI) · 트레이는 19 TC | E7 `busy_port_falls_back_and_names_holder` · E10 `fixed=0 holder=python` · `tray::tip` 단위 시험 |
| A7 임의 포트 · 바뀐 IP 를 승인 초기화 없이 · 키 맞는 상대만 | 충족 | E7 `restarted_peer_on_new_port_is_followed` · `ping_from_other_ip_moves_paired_only` |
| A8 강제 종료 뒤 5초 안 포트 풀림 · 남은 instance.json 이 막지 않음 | 충족(강제 종료) · 정상 종료는 19 TC | E8 자식 소켓 · E9 `freed=1 relisten=1 instance_left=1` |
| A9 웹UI 포트 · 피어 포트 · 열린 연결 수 | 충족 | E7 `open_connections_counts_inbound` · E3 `portText` · E11 |
| A10 기존 테스트 · 웹UI 테스트 · 경고 0 | 충족 | E1 · E2 · E3(39) |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(같은 대화 후속 요청이 문 · 진행 · 목록에서 빠짐 — 사실 · 고침, plan 개정 1) ·
  medium 1(진행 알림 유실 · 피어 간 막힘 — 막힘만 고침, 유실은 명세), 라운드 2: medium 2(진행만 바뀌면 글을 다시 그리지 않음 — 고침 · CLI 무기한
  대기 — 명세, 사용자 결정). 미조치 10건은 [eval.md](eval.md) → 원장.
- 단계 리뷰 2회(서브에이전트, 읽기 전용) — Stage 1(M 2 · L 4): 문 통과 뒤 읽기 · `/api/runs` 400 · CLI 시계 판정 — 반영. Stage 2(M 3 · L 6):
  `AddrInUse` 만 잡은 쪽 찾기 · 툴팁 127 칸 · `port_check.py` 견고성 · macOS 시험 ignore — 반영.
- plan 개정 1건(위임): E5 5 → 6(같은 대화 후속 요청).
- masterPlan D24 개정(PC 전체 동시 실행 상한 · 거절 없는 대기열 · 진행 알림) · D7 보강(Ping 주소 따라감 · 잡은 프로세스) · D32 적용(`Progress` ·
  `Ping.port`).

## 산출

- `an-agent`: `gate::Gate` · `Ticket` · `chat` 의 `admit` · `max_runs` · `run_queue` · `set_max_runs` · `Progress` · `progress_of` · `send_progress`(상대별
  줄) · `tell_waiting` · `inbound_progress`.
- `an-net`: `Request::Progress` · `Ping{port}` · `bind_peer_port_on` · `Holder` · `port_holder` · `set_peer_holder` · `peer_port_note` ·
  `open_connections` · Ping 주소 갱신. `an-core`: `peers::set_addr`.
- `coworkagent-app`: `GET/PUT /api/runs` · 메시지 `progress` · `/api/me` 포트 필드 · CLI 진행 줄 · 기다림 멈춤 · `tray::tip` · 웹UI(메타 줄 · 이 PC
  실행 · 동시 실행 상한 · 내 정보 포트 3줄 · `progressText` · `runsSummary` · `portText` · `messagesSigOf`).
- 도구: `tools/port_check.py`(기본 · `--occupy`). 시험: `agent/tests/capacity.rs`(6) · `port_release.rs`(1) · `app/tests/capacity_api.rs`(5) ·
  `net/tests/ports.rs`(4) · `view.test.mjs` 4.
- 문서: 매뉴얼(an_agent · an_net · an_app · user-guide · developer-guide) · PRODUCT 도메인 규칙(실행 한도 · 피어 주소) · masterPlan.

## 남은 것

- **사용자 결정**: CLI 대기 중 상대가 사라졌을 때 — (a) 대기 포함 절대 상한 · (b) 상대 오프라인이면 다시 셈 · (c) 지금대로.
- 미조치 — [eval.md](eval.md) `## 미조치 지적` → 원장(진행 알림 재시도 · Ping 주소 정책 · "앞에 0건" 문구 · 상한 8 답 겹침 · Unix 미확인).
- 19 인수 TC — 두 PC 대기 안내 · 트레이 툴팁(피어 포트) · 정상 종료 뒤 포트 · 설정 상한.
- 다음 Task — **요청 프롬프트 처리경과 콜백**(사용자 지시) intent 인터뷰.
