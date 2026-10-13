# stage2 — 피어 포트

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

47100 을 남이 잡았으면 그 프로세스를 트레이 · 웹UI 에 보이고(A6), 임의 포트 · 바뀐 IP 를 페어링된 상대가 승인 초기화 없이 따라오게 하며(A7), 설정
화면에 포트 상태를 보이고(A9), 자식 프로세스가 수신 소켓을 물고 가지 않음을 잰다(A8 일부). 대상 gate: E1 · E2 · E7 · E8.

## 산출 파일

- `crates/net/src/node.rs` — (Stage 1 커밋에 먼저 들어간) `bind_peer_port_on` · `Holder` · `port_holder` · `slots` · `open_connections` · Ping 주소 갱신
  + 이번: `Node.holder` · `set_peer_holder` · `peer_holder` · `peer_port_note()`("피어 포트 47100 사용 중 — {이름}(PID {n}) · 임의 포트 {p} 로 받는다").
- `crates/app/src/peers_api.rs` — `/api/me` 에 `ui_port` · `peer_port_fixed` · `peer_port_holder{name, pid}` · `peer_port_note` · `open_connections`.
- `crates/app/src/main.rs` — 잡은 쪽을 노드에 기록 · 툴팁 덧붙임(`App.note`). `crates/app/src/tray.rs` — `run(url, note, …)` · `pub fn tip(n, note)`.
- 웹UI — `view.js` `portText` / `app.js` 내 정보 3줄 · 설정 화면이면 새로 고침마다 다시 읽음(열린 연결 수) / `index.html` 행.
- 테스트 — `net/tests/ports.rs`(4) · `agent/tests/port_release.rs`(1) · `view.test.mjs` `portText`.

## 설계 결정

- **`ui_port` = 요청 Host 의 포트** — guard 가 `Host: 127.0.0.1:{웹UI 포트}` 만 들이므로 같은 값이다. 라우터 상태 · `Me` 구조체(테스트 생성자 다수)를
  넓히지 않았다.
- **`peer_port_fixed` = 지금 포트가 47100** — 잡은 쪽 기록 유무가 아니다(시험 노드는 임의 포트지만 잡은 쪽이 없다).
- **툴팁은 두 줄** — 첫 줄은 지금처럼("coworkAgent" · "coworkAgent — 새 요청 n건"), 둘째 줄이 피어 포트 문구. Windows 툴팁 128자를 넘으면 잘릴 수
  있다(잡은 프로세스 이름이 길 때) — 웹UI 내 정보에 전부 있다.
- **다른 출처 IP 시험은 127.0.0.2 · 127.0.0.3 에서 직접 TLS** — 노드의 보내기는 출처를 고르지 않는다. Windows 루프백은 127/8 전체를 받는다.
- **자식 소켓 시험은 운영 바인드(`bind_peer_port_on(0)`, tokio) + `run_process`** — Windows `ping -n 4` · Unix `sleep 3`(Unix 는 이 PC 에서 미실행).

## 테스트 결과

```text
cargo test -p an-net --test ports                    → test result: ok. 4 passed; 0 failed (2회)
cargo test -p an-agent --test port_release           → test result: ok. 1 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 38 / # fail 0
cargo test --workspace --no-fail-fast                → FAILED · panicked 0
cargo clippy --workspace --all-targets -- -D warnings → Finished
```

- RED: `ports` `restarted_peer…` 는 첫 판에서 `ping()` 의 돌려주는 값(상대 집중 여부)을 성공으로 잘못 읽어 실패 — 시험을 고침.
- `port_holder` Unix(`lsof`) 경로 · Unix `port_release` 는 이 PC(Windows)에서 컴파일 · 실행 못 함 — 확인 필요(Mac 실측 몫).

## 단계 리뷰 (2026-10-13, 분리된 리뷰 에이전트 · diff `a0e3d8e..e695947` + Stage 1 의 net 몫 · `tools/port_check.py`)

High 0 · Medium 3 · Low 6. Ping 주소 갱신(TLS 인증 키의 자기 행 · TCP 출처 IP · port 0 무시 — 다른 호스트로 반사 불가) · netstat 파싱(dual-stack 도
`-p TCP` 표에 0.0.0.0 줄) · `ui_port`(guard 가 Host 정확 일치) · 포트 줄 `textContent` · `open_connections` permit 수명은 결함 없음.

| 지적 | 판단 | 처리 |
|---|---|---|
| M · Ping 이 저장 주소를 출처 IP 로 확인 없이 덮는다 — 다중 NIC · VPN · NAT 면 닿지 않는 주소가 될 수 있다 | 추정 — 명세(해석 8 "출처 IP, port")가 정한 동작 | 미조치 · eval 에 올림. 다음 Ping(30초)이 다시 맞추고, 상대 쪽에서 오는 연결은 그대로 받는다 |
| M · `port_check.py` 가 1초 안 재시작이면 옛 `instance.json` 을 새 것으로 받는다 | 추정 | `start()` 가 **토큰이 바뀐** instance.json 을 기다린다(옛 파일은 남긴 채 띄움 — A8 조건 유지) |
| M · `port_check.py` 가 중간 예외면 앱을 복구하지 않는다 | 사실 | `main` `finally` — 47100 을 연 앱 하나가 아니면 끄고 정상으로 다시 띄움 |
| L · 바인드 실패 사유를 가리지 않아 예약 포트 · 권한도 "사용 중: 알 수 없음" | 사실 | `AddrInUse` 일 때만 잡은 쪽을 찾는다 — 그 밖은 로그 + 임의 포트(잡은 쪽 없음) |
| L · 툴팁 128 칸 넘으면 tray-icon 이 NUL 없이 복사 | 사실(tray-icon 0.26.1 소스) | `tray::tip` 이 UTF-16 127 칸 안으로 자르고 "…" · 단위 시험 `tip_lines_and_windows_limit` |
| L · `ports` 의 127.0.0.2 는 macOS 에서 못 연다 | 사실(Unix 미실행) | `ping_from_other_ip…` 를 Windows 밖에서 `ignore`(사유 표기) — Mac 에서 E7 은 3 passed · 1 ignored(D25 ABANDON 대상) |
| L · `port_check.py` netstat 출력이 비면 거짓 "풀림" | 사실 | 표 머리(` TCP `)가 없으면 중단 |
| L · `port_holder` 가 시작 중 동기 · 시간 제한 없음 | 사실(멈춤 빈도 추정) | 미조치 — 기록(평소 1초 미만, 47100 이 막힐 때만 부름) |
| L · tasklist 출력(OEM 코드페이지)을 UTF-8 로 읽어 한글 프로세스 이름이 깨질 수 있다 | 추정 | 미조치 — 기록(PID 는 정확, 표시 이름만) |

범위 밖 메모(리뷰어): Windows 는 다른 소켓이 `127.0.0.1:p` 를 잡고 있어도 `0.0.0.0:p` 바인드를 허용한다 — 같은 PC 프로세스가 특정 IP 로 47100 을
가로챌 수 있다(키 고정이라 피해는 연결 방해). `SO_EXCLUSIVEADDRUSE` 는 1a 범위 — 원장 후보.
