# stage2 — 프로토콜 · Node 페어링

## 목표

[plan](../plan.md) Stage 2 — 프레임·메시지(`proto`) · `Node`(수신 인가 · hello/approve/remove/ping · 발신 add/approve/
remove/ping). 대상 gate: E5 · E6.

## 산출 파일

- `crates/net/src/{proto,node,lib}.rs` · `crates/core/src/db.rs`(`Connection` · `Error` 재노출 — an-net 이 rusqlite 를 직접
  의존하지 않게)
- `crates/net/tests/` — `pairing.rs`(5) · `limits.rs`(3) · `common/mod.rs`(실제 신원 키 · 실제 DB · 127.0.0.1 임의 포트 노드)

## 설계 결정

- 인가는 `Node::answer` 한 곳 — 상대 SPKI 로 피어 표 조회. hello 는 자기 자신 키 · 포트 0 이면 `bad_request`.
- `approve` 는 내 승인을 먼저 기록하고 통지한다 — 통지 실패 시 오류를 돌려주되 내 승인은 남는다(재시도 = 다시 승인).
- `remove` 는 내 기록을 먼저 지우고 통지는 best-effort.
- 수신 accept 오류 시 100 ms 쉬고 계속(파일 핸들 고갈 시 바쁜 루프 방지).
- `Node::send` 를 공개 — 테스트가 임의 요청(미페어링 ping · 기록 없는 approve · 큰 이름)을 실제 TLS 경로로 보낸다.

## 테스트 결과

- pairing 5 · limits 3 통과. 처음부터 통과해 mutation 으로 RED 를 확인했다:

| mutation | 잡은 테스트 |
|---|---|
| ping 이 무조건 pong | unpaired_ping_rejected · one_side_approval_is_not_paired · removed_peer_rejected_again |
| `paired` = my_ok ∨ their_ok | one_side_approval_is_not_paired |
| 프레임 상한 제거 | oversize_frame_rejected(2초 안에 끊기는지 단정 — 처음엔 10초 연결 상한 뒤 끊겨도 통과해 단정을 보강) |
| 미승인 상한 제거 | unapproved_peer_cap |
| 기록 없는 approve 허용 | unpaired_ping_rejected |

## 다음 단계

Stage 3 — app 배선(신원 · 노드 · 피어 포트) · `/api/me` · `/api/peers*` · 웹UI 섹션 · 트레이 "내 정보 보기".
