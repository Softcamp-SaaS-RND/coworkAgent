# stage1 — workspace + an-core + 의존 경계 검사

- **적대적 평가 기준선(BASE)**: `46c9c8d` — plan 승인 커밋(사용자 "승인", 2026-10-11).

## 목표

[plan](../plan.md) Stage 1 — Cargo workspace, `an-core`(데이터 경로 · SQLite · 파일 로그 회전 · 단일 인스턴스 잠금 ·
핸드오프 정보), `tools/check_crates.py`. 대상 gate: E5 · E6 · E7 · E12(E12 의 `crates=2 edges=1` 은 stage 2 에서 app 이
생겨야 맞는다).

## 산출 파일

- `Cargo.toml`(virtual workspace, `crates/*`, resolver 3) · `Cargo.lock`
- `crates/core/` — `Cargo.toml` · `src/{lib,paths,db,log_file,instance}.rs`
- `crates/core/tests/` — `paths.rs`(2) · `log_rotate.rs`(4) · `instance_lock.rs`(3)
- `tools/check_crates.py`

## 설계 결정

- 경로 API 는 루트를 인자로 받는다(`Paths::under`) — 운영 루트는 `Paths::default_root()` = `dirs::data_local_dir()/agentNetwork`.
  환경변수 스위치 없음.
- 로그 회전: `len > 0 && len + 이번 기록 > 1_048_576` 이면 `.1` 로 바꾸고 새로 쓴다. 빈 파일은 회전하지 않는다(한 번에 큰
  기록이 와도 빈 백업을 만들지 않게). 백업 1개(`ponytail:` 표시).
- 인스턴스 잠금: `File::try_lock` → `WouldBlock` 이면 Secondary. 잠금 파일은 열기만 하고 내용을 쓰지 않는다.
- 핸드오프 정보 `instance.json` 은 `{"port":N,"token":"hex"}` — serde 없이 우리가 쓴 형식만 읽는다(토큰 hex 검증).
  단위 테스트 2건은 `src/instance.rs` 안(개수 gate 파일 밖).
- `check_crates.py` 의 Layer: `an-core`=base · `agentnetwork-app`=app. 내부 간선은 `path` 의존만 센다.

## 테스트 결과

RED(구현 전, 스텁 `todo!()`): `cargo test -p an-core --no-fail-fast` → instance_lock 3 failed · log_rotate 4 failed · paths 2 failed.

GREEN:

```text
$ cargo test --workspace
test result: ok. 2 passed; 0 failed   (src/instance.rs 단위)
test result: ok. 3 passed; 0 failed   (instance_lock)
test result: ok. 4 passed; 0 failed   (log_rotate)
test result: ok. 2 passed; 0 failed   (paths)
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
$ python tools/check_crates.py
crates=1 edges=0 violations=0
```

경계 보호 변이: 회전 조건 `>` → `>=` 로 바꾸면 `limit_exact_kept` FAILED(3 passed; 1 failed) — 원복 확인.

## 다음 단계

[plan](../plan.md) Stage 2 — `crates/app` 로컬 웹UI 서버(토큰 · Host · Origin) + 상태 페이지 + 테스트 local_ui(6) · handoff(2).
