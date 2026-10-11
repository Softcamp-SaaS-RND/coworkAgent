# plan — 01.pairing

> 근거: [intent](intent.md)(승인 2026-10-11, 자동 진행 위임). **구현자는 이 문서만 보고 일한다** — intent 의 성공 기준·결정·
> 제외를 아래에 옮겼다. 프로젝트 결정 번호(D·S)는 [masterPlan](../../masterPlan.md).

## 목표

설치별 ed25519 신원 · 내 정보 보기(S4) · 수동 주소 연결 · 양측 지문 비교 페어링 · 공개키 고정 TLS(D8). 02 의 메시지는 이
통로의 "페어링된 피어만" 관문 뒤로 붙는다.

### 성공 기준(intent 사본)

- A1: 첫 실행에 신원 키가 D15 경로에 생기고 재실행해도 지문이 같다. 키 파일이 손상되면 새로 만들지 않고 시작을 멈춘다.
- A2: 트레이 메뉴 "내 정보 보기" 가 웹UI 의 내 정보(이름 · PC 이름 · 지문 · 수신 주소)를 연다.
- A3: 피어 통신은 TLS 1.3 상호 raw public key 인증이다 — 고정한 키와 다른 상대 · 클라이언트 키를 내지 않는 상대는 연결되지 않는다.
- A4: 수동 주소 추가 뒤 **양쪽 사람이 모두 승인해야** 페어링이 끝난다 — 한쪽만 승인한 상태는 페어링이 아니다.
- A5: 페어링하지 않은(또는 삭제한) 피어의 페어링 외 요청은 거부되고, 페어링한 피어의 요청은 통과한다.
- A6: 원격 입력에 상한이 있다 — 큰 프레임 거부 · 이름 정제 · 내가 승인하지 않은 피어 수 상한.
- A7: Windows PC 2대가 실제 LAN 에서 서로 주소를 추가 · 지문 비교 · 양측 승인으로 페어링된다.
- A8: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 · 의존 경계 행이 통과한다.

### intent 결정 · 제외(사본)

- P1 `identity.key`(ed25519 PKCS#8, 임시 파일 → 이름 바꾸기, 손상 시 시작 실패) · P2 지문 SHA-256(SPKI) 앞 16바이트 대문자
  hex 4자리 8묶음 · P3 `0.0.0.0:47100`, 사용 중이면 임의 포트 · P4 hello → 각자 승인 → approve 통지 → 양쪽 승인 = 페어링,
  삭제 = 내 기록 삭제 + remove 통지 · P5 미페어링에 허용 = hello · approve · remove · P6 프레임 64 KiB · 이름 64자 ·
  연결 10초 · 동시 32 · 미승인 피어 32 · P7 이름 = OS 사용자, PC = 호스트 이름, 트레이 "내 정보 보기" = `/#me`.
- 제외: 에이전트 · 메시지 · 이력(02) / mDNS · S5 목록 · 오프라인 재전송(Phase 2) / 슬랙형 UI(Phase 1) / 이름 편집 · 차단 ·
  키 교체 / macOS 실동작(D25).

## 전제

- 00.skeleton 위에 쌓는다(브랜치 `task/01-pairing` 은 `task/00-skeleton` 에서 갈라졌고, 00 의 리뷰 반영은 merge 로 받는다).
- 검증 환경: Windows 11 · rustc 1.95.0 `x86_64-pc-windows-msvc` · Python 3.12. 의존 첫 다운로드는
  [환경 노트](../../tech/windows-crates-io-tls-revocation.md).

## 설계

### 크레이트

| 크레이트 | Layer | 이번에 담는 것 | 내부 의존 |
|---|---|---|---|
| `an-core` | base | `Paths::identity_file` · DB 스키마 2(`peers` 표) · `peers` 저장 함수 | 없음 |
| `an-net`(`crates/net`, 신규) | comm | 신원 키 · 지문 · TLS 설정·검증기 · 프레임·메시지 · `Node`(수신·페어링 동작) | `an-core` |
| `agentnetwork-app` | app | 트레이 "내 정보 보기" · `/api/me` · `/api/peers*` · 웹UI 섹션 · 시작 배선 | `an-core` · `an-net` |

- `tools/check_crates.py` LAYER 에 `"an-net": "comm"`. 기대 `crates=3 edges=3 violations=0`.
- 암호(ring · rustls)는 `an-net` 에만. `an-core` 는 SPKI 를 BLOB 으로만 안다.

### 외부 의존(2026-10-11 crates.io 최신 안정판)

| crate | 버전 | 용도 |
|---|---|---|
| rustls | 0.23.45 (`default-features = false`, `ring` · `std` · `logging`) | TLS 1.3 · raw public key · 검증기 trait. TLS 1.2 를 빌드에 넣지 않는다 |
| tokio-rustls | 0.26.6 (`ring` · `logging`) | 비동기 TLS 스트림 |
| ring | 0.17.14 | ed25519 키 생성 · SHA-256(지문). aws-lc 는 cmake·nasm 이 필요해 기각 |
| serde · serde_json | 1.0.229 · 1.0.152 | 원격 프레임 JSON — 신뢰하지 않는 입력을 손 파서로 읽지 않는다 |

### 신원(P1 · P2 · A1)

- `an_net::identity::Identity::load_or_create(path)`: 파일이 없으면 `Ed25519KeyPair::generate_pkcs8` → `identity.key.tmp` 쓰기
  → `identity.key` 로 이름 바꾸기. 있으면 읽어 `rustls::crypto::ring::sign::any_eddsa_type` 로 연다(ed25519 가 아니면 오류).
- SPKI 는 서명 키의 `public_key()`. 지문 `fingerprint(spki)` = SHA-256 앞 16바이트 → `7F3A 19C2 …`(39자).

### TLS(A3)

- 양쪽 모두 `builder_with_provider(ring)` + `with_protocol_versions(&[&TLS13])`. 키는 `AlwaysResolves{Server,Client}RawPublicKeys`.
- 검증기는 **키 보유 증명만** 한다: `verify_tls13_signature_with_raw_key`, `requires_raw_public_keys() = true`,
  `supported_verify_schemes() = [ED25519]`, TLS 1.2 서명 검증은 오류.
  - 서버 측(클라이언트 검증): 클라이언트 키 필수, 아무 키나 받는다(누구인지는 연결 뒤 판단).
  - 클라이언트 측(서버 검증): 고정 키가 있으면 SPKI 바이트가 같아야 한다(다르면 핸드셰이크 실패). 첫 연결(주소 추가)만 고정
    없이 받고 그 키를 기록한다 — 사람이 지문을 비교해 승인한다.
- **인가는 핸드셰이크 뒤**: `peer_certificates()[0]` = 상대 SPKI → `peers` 표 조회.

### 프로토콜(P4 · P5 · P6)

- 연결 1개 = 요청 1개 · 응답 1개. 프레임 = 4바이트 big-endian 길이 + JSON, 64 KiB 초과는 읽지 않고 끊는다.
- 요청 `{"type": "hello", "name", "host", "port"}` · `approve` · `remove` · `ping`. 응답 `hello{name, host}` · `ok` · `pong` ·
  `error{code}`(`not_paired` · `unknown` · `busy` · `bad_request`).
- 수신 처리:
  - `hello`: 이름·호스트 정제(제어 문자 제거 · trim · 64자 · 빈 값은 `?`) · 처음 보는 키인데 내가 승인하지 않은 피어가 32 이상이면
    `busy` · 주소는 접속한 IP + `port` · 기존 기록이면 이름·주소만 갱신(승인 상태 유지) → `hello{나}`.
  - `approve`: 기록이 있으면 `their_ok` → `ok`, 없으면 `unknown`. `remove`: 그 키의 기록 삭제 → `ok`.
  - `ping`: 페어링(my_ok ∧ their_ok)이면 `pong`, 아니면 `not_paired`.
- 수신 연결: 동시 32(세마포어) · 연결당 10초(TLS 포함 전체). 발신: 10초.
- 발신 동작(`Node`): `add(addr)`(고정 없이 hello · 자기 자신이면 거부) · `approve(id)`(내 승인 기록 → 고정 키로 approve 통지,
  실패하면 오류를 돌려주고 내 승인은 유지 — 다시 누르면 재통지) · `remove(id)`(내 기록 삭제 → remove 통지는 실패해도 무시)
  · `ping(id)`(고정 키).
- "페어링 받기" 창(P8, 개정 1): `Node::open_window(d)` → 처음 보는 키의 hello 1건을 `d`(운영 `PAIRING_WINDOW` = 300초) 안에 받고
  닫는다. 창 밖 · 소비 뒤 · 마감 뒤는 `closed`(상한 `busy` 판정이 먼저). 아는 키의 hello 는 창과 무관, `peers::upsert` 가
  이름·주소와 함께 `my_ok` · `their_ok` 를 0 으로 되돌린다(발신 쪽 `add` 도 같은 함수). `remove(id)` 는 통지 성공 여부(bool)를
  돌려준다 — API `DELETE` 는 `{"notified": bool}`, 웹UI 는 실패를 표시한다. API `POST /api/pairing-window` → `{"seconds": 300}`.

### 저장(SQLite 스키마 2)

```sql
CREATE TABLE peers (
  id INTEGER PRIMARY KEY, spki BLOB NOT NULL UNIQUE, name TEXT NOT NULL, host TEXT NOT NULL, addr TEXT NOT NULL,
  my_ok INTEGER NOT NULL DEFAULT 0, their_ok INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL);
```

- `db::open`: `user_version` 0·1 → 표 생성 후 2. 실행 파일보다 새 스키마(> 2)면 열지 않는다(오류).
- 00 의 `db_user_version_1_kept_on_reopen` 은 스키마 상수를 단정하는 `db_schema_version_kept_on_reopen` 으로 이름을 바꾼다
  (00 E5 의 개수 2 는 그대로).

### 웹UI · 트레이(A2)

- 트레이 메뉴 "열기" · "내 정보 보기" · "종료". "내 정보 보기" = `{진입 URL}#me`(진입 리다이렉트 뒤에도 조각이 남는다).
- `GET /api/me` → `{name, host, fingerprint, port, addrs}`(addrs = 기본 경로 IPv4 + 포트). `GET /api/peers` →
  `[{id, name, host, addr, fingerprint, my_ok, their_ok, paired}]`. `POST /api/peers {addr}` · `POST /api/peers/{id}/approve` ·
  `DELETE /api/peers/{id}` · `POST /api/peers/{id}/ping`. 모두 00 의 guard 뒤. JSON 요청은 `Content-Type: application/json` 필수.
- 원격에서 온 이름은 웹UI 에서 `textContent` 로만 그린다.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | core 스키마 2 · `an-net` 신원 · TLS | `crates/net` · 테스트(identity · tls) | E3 · E4 · E9 |
| 2 | 프로토콜 · `Node` 페어링 | 테스트(pairing · limits) | E5 · E6 |
| 3 | app 배선 · API · 웹UI · 트레이 | 테스트(peers_api) · 릴리스 빌드 | E1 · E2 · E7 · E10 · E11 |

- 각 stage 는 TDD · `work/stage{M}.md` · 자기 검증. 개수 단정 gate 의 테스트 이름은 아래 그대로.

## 평가 항목

- [ ] E1: A8 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A8 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — 신원 3건(created_then_same_fingerprint · corrupt_key_is_error_not_replaced · fingerprint_is_8_groups_of_4_hex)
  CHECK: cargo test -p an-net --test identity
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A3 — TLS 3건(pinned_key_connects_and_sees_client_key · pinned_mismatch_fails · client_without_key_rejected)
  CHECK: cargo test -p an-net --test tls
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A4·A5 — 페어링 5건(add_leaves_both_sides_pending · one_side_approval_is_not_paired · both_approvals_pair · unpaired_ping_rejected · removed_peer_rejected_again)
  CHECK: cargo test -p an-net --test pairing
  EXPECT: test result: ok. 5 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A6 — 원격 입력 상한 3건(oversize_frame_rejected · remote_name_sanitized · unapproved_peer_cap)
  CHECK: cargo test -p an-net --test limits
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A2 — 웹UI API 2건(me_shows_fingerprint_and_port · add_then_list_peer)
  CHECK: cargo test -p agentnetwork-app --test peers_api
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A8 — 의존 경계: 크레이트 3개 · 내부 간선 3개(app → core · app → net · net → core) · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=3 edges=3 violations=0
  EVIDENCE: pending
- [ ] E12: A4·A6 — 페어링 받기 창 · 재페어링 수렴 4건(unknown_hello_closed_without_window · window_accepts_one_then_closes · window_expires · lost_remove_readd_converges)
  CHECK: cargo test -p an-net --test pairing_guard
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E13: A3·A4·A6 — 초대 코드 · 세대 · 인증 전 연결 3건(wrong_code_does_not_consume_window · stale_approve_rejected · slow_clients_do_not_block_others)
  CHECK: cargo test -p an-net --test hardening
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E14: A3·A4 — 초대 텍스트 3건(invite_roundtrip_through_messenger_text · invite_parse_rejects_malformed · invite_add_pins_fingerprint)
  CHECK: cargo test -p an-net --test invite
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A1 — 스키마 2 이전·재열기(core paths 2건 유지)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E10: A2 — Windows 실동작(수동): 트레이 "내 정보 보기" → 브라우저에 내 정보(이름 · PC 이름 · 지문 · 수신 주소) · 재실행 후 지문 동일
  EVIDENCE: pending
- [ ] E11: A7 — Windows PC 2대(수동): 같은 LAN 에서 B 가 "페어링 받기" → A 가 B 주소 + 초대 코드로 추가 → 양쪽 화면 지문이 상대 내 정보와 일치 → 양쪽 승인 → 양쪽 "페어링됨" · ping 성공 · 한쪽 삭제 뒤 ping 거부(누가·언제 / 두 PC 이름·IP / 관측 지문)
  EVIDENCE: pending

## 종결 시 할 일

- 루트 `PRODUCT.md`: 핵심 의존 · 모듈 Layer 표에 `an-net` · 도메인 규칙(신원 · 페어링).
- `docs/manual/dev/` 에 `an-net` 매뉴얼.
- masterPlan `## 개정 기록` 에 `#01` 한 줄(P1~P7 중 프로젝트 수준 결정 승격 여부 포함).
- `eval.md`: gate 원장 → 적대적 평가(BASE = plan 커밋, 최대 2 라운드) → `## 미조치 지적` → 원장.

## 개정 기록

3. 2026-10-11 · intent P10 · 설계 "웹UI" · 평가 항목 E14 추가 · E10 · E11 ABANDON — 초대 텍스트 복사/붙여넣기(`an_net::invite`,
   `tls::Pin::Fingerprint`, `Node::add_invite` · `open_invite`, API `POST /api/peers {invite}` · `POST /api/pairing-window` →
   `{seconds, code, invite}`). 수동 gate E10 · E11 은 masterPlan D28(프로토타입 인수 Task 일괄)로 ABANDON. 사유: 사용자 지시
   2026-10-11(초대 UX · 원격 세션이라 수동 확인 불가 · 프로토타입까지 함수 테스트 · TDD 만). 승인: 사용자 지시 + 확인 답.
   적대적 평가는 라운드 상한(2)에 이미 도달 — 이 개정분은 적대적 평가를 거치지 않았다(report 에 명시).
2. 2026-10-11 · 설계 "프로토콜" · intent P8 개정 · P9 · 평가 항목 E13 추가 — 페어링 받기 창을 초대 코드에 묶음(API 추가 본문
   `code`, `POST /api/pairing-window` → `{seconds, code}`), hello · approve 에 세대 `round`(DB `peers.round`), 수신 연결 출처별 4 ·
   3초(`INBOUND_DEADLINE`) · 전체 초과 시 즉시 끊기. E11 절차: B "페어링 받기" → A 가 B 주소 + 초대 코드로 추가. 사유: 적대적 평가
   라운드 2(창 선점 · 인증 전 연결 포화 · 세대 없는 approve). 승인: 자동 진행 위임 + 사용자 "진행 / 의견 수용"(2026-10-11).
1. 2026-10-11 · 설계 "프로토콜" · intent 결정 P8 · 평가 항목 E12 추가 — 처음 보는 키의 hello 를 "페어링 받기" 창(5분 · 1건)
   뒤로, 아는 키의 hello 는 양쪽 승인 초기화, 삭제 통지 결과 노출. E11 수동 절차에 "B 가 페어링 받기 → A 가 주소 추가" 순서를
   더한다. 사유: 적대적 평가 라운드 1(미인증 hello 32회 슬롯 고갈 · 삭제 통지 유실 뒤 재페어링 불수렴). 승인: 자동 진행 위임.
