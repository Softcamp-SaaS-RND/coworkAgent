# eval.md — 01.pairing 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 48c1264a016eaf8c5b5b8e5992f9241851b478d0

- [x] E1: A8 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A8 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.21s

- [x] E3: A1 — 신원 3건(created_then_same_fingerprint · corrupt_key_is_error_not_replaced · fingerprint_is_8_groups_of_4_hex)
  CHECK: cargo test -p an-net --test identity
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E4: A3 — TLS 3건(pinned_key_connects_and_sees_client_key · pinned_mismatch_fails · client_without_key_rejected)
  CHECK: cargo test -p an-net --test tls
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E5: A4·A5 — 페어링 5건(add_leaves_both_sides_pending · one_side_approval_is_not_paired · both_approvals_pair · unpaired_ping_rejected · removed_peer_rejected_again)
  CHECK: cargo test -p an-net --test pairing
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

- [x] E6: A6 — 원격 입력 상한 3건(oversize_frame_rejected · remote_name_sanitized · unapproved_peer_cap)
  CHECK: cargo test -p an-net --test limits
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

- [x] E7: A2 — 웹UI API 2건(me_shows_fingerprint_and_port · add_then_list_peer)
  CHECK: cargo test -p agentnetwork-app --test peers_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E8: A8 — 의존 경계: 크레이트 3개 · 내부 간선 3개(app → core · app → net · net → core) · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: crates=3 edges=3 violations=0

- [x] E12: A4·A6 — 페어링 받기 창 · 재페어링 수렴 4건(unknown_hello_closed_without_window · window_accepts_one_then_closes · window_expires · lost_remove_readd_converges)
  CHECK: cargo test -p an-net --test pairing_guard
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.14s

- [x] E13: A3·A4·A6 — 초대 코드 · 세대 · 인증 전 연결 3건(wrong_code_does_not_consume_window · stale_approve_rejected · slow_clients_do_not_block_others)
  CHECK: cargo test -p an-net --test hardening
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.02s

- [x] E14: A3·A4 — 초대 텍스트 3건(invite_roundtrip_through_messenger_text · invite_parse_rejects_malformed · invite_add_pins_fingerprint)
  CHECK: cargo test -p an-net --test invite
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E9: A1 — 스키마 2 이전·재열기(core paths 2건 유지)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [ ] E10: A2 — Windows 실동작(수동): 트레이 "내 정보 보기" → 브라우저에 내 정보(이름 · PC 이름 · 지문 · 수신 주소) · 재실행 후 지문 동일
  EVIDENCE: pending

- [ ] E11: A7 — Windows PC 2대(수동): 같은 LAN 에서 B 가 "페어링 받기" → A 가 B 주소 + 초대 코드로 추가 → 양쪽 화면 지문이 상대 내 정보와 일치 → 양쪽 승인 → 양쪽 "페어링됨" · ping 성공 · 한쪽 삭제 뒤 ping 거부(누가·언제 / 두 PC 이름·IP / 관측 지문)
  EVIDENCE: pending

ABANDON: E10 원격 세션이라 화면 확인 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)
ABANDON: E11 원격 세션 · PC 2대 확인 불가 — 프로토타입(Phase 1 끝) 인수 Task 에서 수동 테스트 일괄(masterPlan D28, 사용자 지시 2026-10-11)

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base b6d6647 --scope branch` · 대상 HEAD `bb53f60` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 미인증 hello 32회로 신규 페어링을 지속 차단할 수 있음 (crates/net/src/node.rs:155-165)
  TLS 클라이언트 키는 임의 키를 허용하며, 처음 보는 키의 hello를 즉시 영구 DB에 저장합니다. 공격자가 ed25519 키 32개로 슬롯을 채우면 이후 정상 피어는 모두 busy를 받습니다. TTL, pairing window, source별 rate limit이 없고 limits.rs의 unapproved_peer_cap 테스트도 33번째 키가 거부됨을 확인합니다. 항목은 재시작 후에도 남고 삭제 즉시 다시 채울 수 있어 LAN의 미페어링 호스트 하나가 핵심 페어링 기능을 지속 방해할 수 있습니다.
  Recommendation: unknown hello 저장을 명시적·단기 pairing mode 또는 invitation nonce 뒤로 옮기십시오. 최소한 pending TTL, source별 rate limit, 안전한 eviction을 추가하고 포화 상태에서도 승인된 초대가 진행되는 회귀 테스트를 작성하십시오.
- [medium] remove 통지 실패 뒤 재페어링 상태가 정상 UI 흐름으로 수렴하지 않음 (crates/net/src/node.rs:265-275)
  로컬 기록을 먼저 삭제하고 remove 전송 실패를 로그만 남긴 채 성공 처리합니다. 이후 다시 add하면 상대는 기존 my_ok/their_ok를 보존하지만 삭제한 쪽은 false/false로 새 레코드를 만듭니다. 삭제한 쪽이 approve해도 상대가 자신의 승인을 다시 보내지 않아 해당 쪽은 my_ok=true, their_ok=false에 고착됩니다. 상대 UI는 이미 paired라 재통지 버튼도 노출하지 않습니다. 일시적 네트워크 장애만으로 양쪽 신뢰 상태가 달라지고 정상 UI에서 복구할 수 없습니다.
  Recommendation: remove를 tombstone/outbox로 저장해 재시도하거나 부분 실패를 API/UI에 표시하십시오. hello 응답에 승인 세대·상태를 포함해 재동기화하고, lost remove → re-add → 양측 paired 회귀 테스트를 추가하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 미인증 hello 슬롯 고갈 | 사실 — 창 없이 처음 보는 키를 영구 저장 | intent P8 · plan 개정 1: 처음 보는 키의 hello 는 "페어링 받기" 창(5분 · 1건) 안에서만, 그 밖은 `closed`. 회귀 `pairing_guard.rs`: 창 없이 40회 → 0건 저장 후 정상 초대 진행 · 1건 뒤 닫힘 · 마감 경과(E12). mutation 3종(창 검사 제거 · 소비 안 함 · 마감 무시) 모두 FAIL 확인 |
| 삭제 통지 유실 뒤 재페어링 불수렴 | 사실 — hello 가 상대 쪽 승인을 보존 | hello(주소 추가)는 양쪽 승인을 0 으로(`peers::upsert`). `remove` 가 통지 성공 여부를 돌려주고 API `{"notified"}` · 웹UI 실패 표시. 회귀 `lost_remove_readd_converges`(E12) — 승인 보존 mutation 에서 FAIL 확인. tombstone 재시도는 두지 않음(재추가가 수렴하므로) |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 · 같은 base `b6d6647` · 대상 HEAD `86e8f07` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 임의 키가 1회용 페어링 창을 선점해 정상 페어링을 계속 차단할 수 있음 (crates/net/src/node.rs:173-184)
  처음 보는 TLS 키라면 의도한 상대인지 확인하지 않고 `take_window()`로 창을 즉시 소비합니다. LAN 공격자가 유효한 ed25519 키로 hello를 반복하면 사용자가 창을 열 때마다 먼저 1건을 차지하고, 정상 PC는 `closed`를 받습니다. UI에는 창이 조기 소비됐다는 상태도 없어 실패 원인이 숨습니다. E12는 정상 상대가 먼저 도착하는 순차 경로만 검사해 이 경쟁을 재현하지 않습니다.
  Recommendation: 초대 nonce/코드 또는 예상 주소·지문에 창을 결합하고, 일치하는 hello만 창을 소비하게 하십시오. 공격 hello 선점 후에도 정상 초대가 성공하는 회귀 테스트와 출처별 rate limit을 추가하십시오.
- [high] 인증 전 연결 32개로 수신 서비스를 지속 포화시킬 수 있음 (crates/net/src/node.rs:123-147)
  세마포어 permit을 TLS 인증 전에 raw TCP 연결별로 점유하고 각 연결을 10초간 허용합니다. 한 호스트가 handshake나 frame을 완료하지 않는 소켓 32개를 유지하면 약 3.2 connection/s만으로 accept가 멈춰 정상 페어링·ping·remove를 차단할 수 있습니다. 페어링 창과 DB 상한은 모두 이 단계 뒤라 방어하지 못합니다.
  Recommendation: accept 후 전역·출처별 pre-auth quota를 적용하고 TLS handshake/frame-header에 더 짧은 별도 deadline과 rate limit을 두십시오. 32개 slow client 중 정상 연결이 진전되는 회귀 테스트를 추가하십시오.
- [medium] 세대 없는 Approve가 재페어링 reset 뒤 적용되어 양쪽 상태가 갈릴 수 있음 (crates/net/src/node.rs:191-196)
  `Approve`에는 pairing generation/nonce가 없고 수신 시 현재 레코드의 `their_ok`를 무조건 설정합니다. 재추가 hello가 승인을 초기화하는 동안 이전 승인 요청이 늦게 도착하면 새 라운드 승인으로 오인됩니다. 이후 한쪽만 `paired`가 되고 반대쪽은 pending인 상태가 가능하며, `lost_remove_readd_converges`는 hello와 approve를 직렬 실행해 이 ordering을 놓칩니다.
  Recommendation: hello마다 pairing generation/nonce를 발급해 DB에 저장하고 Approve가 같은 generation일 때만 상태를 변경하게 하십시오. hello/approve 순서를 교차시킨 회귀 테스트로 양쪽 상태 수렴을 검증하십시오.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 페어링 창 선점 | 사실 — 처음 보는 키면 무엇이든 창을 소비 | 창을 초대 코드(8자 · 31자 알파벳 ≈ 39.6 비트 · 상수 시간 비교)에 묶음. 틀린 코드는 `closed` 이고 창을 닫지 않는다. 회귀 `hardening.rs::wrong_code_does_not_consume_window`(E13) — "틀린 코드가 창 소비" mutation 에서 FAIL 확인 |
| 인증 전 연결 포화 | 사실 — permit 을 accept 전에 기다리고 10초 유지 | 출처 IP 당 동시 4 · 수신 연결 3초(`INBOUND_DEADLINE`) · 전체 32 초과는 accept 를 멈추지 않고 바로 끊기. 회귀 `slow_clients_do_not_block_others`(127.0.0.2 에서 40개 → 127.0.0.1 요청 1초 안 처리 · 느린 연결 3.5초 안 끊김, E13) — 출처별 몫 제거 · 상한 10초 mutation 에서 FAIL 확인 |
| 세대 없는 approve | 사실 | hello 마다 `round`(16바이트 난수)를 양쪽 `peers.round` 에 저장, approve 는 같은 세대에만(`stale`). 회귀 `stale_approve_rejected`(E13) — 세대 무시 mutation 에서 FAIL 확인 |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| E10 · E11 수동 gate 미충족(라운드 1·2) | 사실 | 프로토타입(Phase 1 끝) 인수 Task 로 이관(D28). ABANDON 기록 |
| 출처별 속도(rate) 제한(라운드 2 권고) | 사실 — 동시 수 제한만 넣음 | 동시 4 · 3초 상한으로 한 출처의 점유가 묶이고 초대 코드로 창 선점이 막혀 실익이 작다. 여러 IP 를 쓰는 LAN 공격자가 전체 32 자리를 3초씩 반복 점유하는 경우는 남는다 — Phase 2(mDNS · 연결 가능 목록)에서 수신 정책을 다시 볼 때 검토 |
| 삭제 통지 tombstone 재시도(라운드 1 권고) | 사실 — 넣지 않음 | 다시 추가가 양쪽 승인을 지워 수렴하고(`lost_remove_readd_converges`), 통지 실패는 API · 웹UI 에 드러난다 |
