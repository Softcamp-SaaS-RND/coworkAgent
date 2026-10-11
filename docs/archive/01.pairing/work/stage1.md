# stage1 — core 스키마 2 · an-net 신원 · TLS

- **적대적 평가 기준선(BASE)**: `b6d6647` — plan 커밋(자동 진행 위임, 2026-10-11).

## 목표

[plan](../plan.md) Stage 1 — `an-core` 스키마 2(`peers` 표 · 저장 함수 · `identity_file`), 신규 `an-net` 의 신원 키 · 지문 ·
TLS 설정·검증기. 대상 gate: E3 · E4 · E9.

## 산출 파일

- `crates/core/src/{db,peers,paths,lib}.rs` · `crates/core/tests/paths.rs`(이름 변경 `db_schema_version_kept_on_reopen`)
- `crates/net/` — `Cargo.toml` · `src/{lib,identity,tls}.rs` · `tests/{identity,tls}.rs`
- `tools/check_crates.py` — `"an-net": "comm"`

## 설계 결정

- 스키마: `user_version` 0·1 → `peers` 표 생성 + 2. 더 새 스키마(> 2)는 `db::open` 오류 — 옛 실행 파일이 새 DB 를 망치지 않게.
- `peers::upsert` 는 `ON CONFLICT(spki)` 로 이름 · 호스트 · 주소만 갱신, 승인 플래그는 건드리지 않는다.
- 신원 키는 `any_eddsa_type` 으로만 연다(RSA·ECDSA PKCS#8 이면 오류). SPKI 는 rustls 서명 키의 `public_key()`.
- TLS 검증기는 키 보유 증명(`verify_tls13_signature_with_raw_key`)만, 스킴은 ED25519 하나. 서버 쪽은 클라이언트 키 필수 ·
  아무 키나, 클라이언트 쪽은 고정 키가 있으면 바이트 비교.
- `rustls` 는 `default-features = false` — TLS 1.2 코드가 빌드에 없다.

## 테스트 결과

- identity 3 · tls 3 · core paths 2 통과. 지문 기대값은 Python `hashlib` 로 따로 구했다(처음 적은 추정값이 틀려 RED 확인).
- mutation: 고정 키 비교를 끄면 `pinned_mismatch_fails` FAIL(확인). `client_auth_mandatory = false` 로 바꿔도
  `client_without_key_rejected` 는 통과 — 키 없는 클라이언트는 RPK 클라이언트 인증서 유형 협상에서 이미 끊긴다. 테스트는
  "거부된다" 는 결과를 단정하고 어느 층이 막는지는 단정하지 않는다.

## 다음 단계

Stage 2 — 프레임·메시지 · `Node`(수신 · hello/approve/remove/ping) · pairing 5 · limits 3.
