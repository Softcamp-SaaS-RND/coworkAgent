# stage 1 — 1.1.1 서명 릴리스 · 업데이터 되살리기 · Authenticode 확인 · 1.2.0

## 목표

intent 결정 1 ~ 6 — 서명된 1.1.1 등록, #1y 업데이터를 배포처 `Softcamp-SaaS-RND/coworkAgent` 로 되살리고 받은 exe 의 회사 인증서 서명을 확인.

## 산출 파일

- 릴리스 `v1.1.1`(Latest) — 서명 exe 사본(SHA-256 `2798778c…43cffc`), `--target fe4e7d2a…`.
- `crates/app/src/update.rs`(되살림 + `RELEASES` · `SIGNER` · `check(…, trusted)` · `swap(exe)` · `authenticode`), `crates/app/tests/update.rs`(되살림 + 주입 · 새 시험 3).
- `crates/app/src/main.rs` · `lib.rs` · `crates/core/src/paths.rs`(`8610de5` 역적용 + `authenticode` 주입), `crates/app/Cargo.toml`(`ureq` · `windows-sys` 기능), 크레이트 넷 1.2.0 · `Cargo.lock`.
- 문서 — 개발자 가이드 · an_app · an_core · 사용자 가이드 · README · PRODUCT(`8610de5` 역적용 + 배포처 · 서명 확인 · v1.2.0).

## 설계 결정

- 되살리기 = 파일째(`git checkout 8610de5^ -- update.rs tests/update.rs`) + 작은 hunk 역적용(`git apply -R`) — 걷어낸 뒤 바뀐 곳이 없어 그대로 적용됐다.
- 서명 확인 자리 = `check` 안 `.new` 쓰기 뒤 · 이름 바꾸기 전(`WinVerifyTrust` 는 파일 경로를 받는다) — 실패면 `.new` 를 지워 다음 시작의 `clean` 을 기다리지 않는다.
  확인 함수는 주입(`trusted`) — 로컬 HTTP 시험의 가짜 PE 는 서명이 없으므로.
- `WTHelper*` 는 `windows-sys` 의 `Win32_Security_Cryptography_Catalog` · `_Sip` 기능 아래에 있다(빌드 오류로 확인).
- 서명자 이름 = `CERT_NAME_SIMPLE_DISPLAY_TYPE`(CN 값 — `Get-AuthenticodeSignature` 의 `CN="SOFTCAMP CO.,LTD."` 따옴표 없이). 실제 릴리스 exe 로 통과 확인(`real_signed_release`).

## 테스트 결과

```text
cargo test -p coworkagent-app --test update                 → 10 passed; 0 failed; 2 ignored
cargo test -p coworkagent-app --test update -- --ignored    → 2 passed(latest = v1.1.1 · 서명 통과 · 다른 이름 · 고친 사본 실패)
cargo test --workspace · cargo clippy --workspace --all-targets -- -D warnings → rc 0
```
