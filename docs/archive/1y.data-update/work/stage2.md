# stage 2 — 자동 업데이트 · 서명 도구 · 키

## 한 일

- an-app `update.rs`: `RELEASES` · `ASSET`(Windows `coworkAgent.exe`, 다른 OS `None`) · `SIG_EXT` · `PUBLIC_KEY`(아래 키) · `MAX_ASSET` 64 MiB, `check` → `Outcome`
  (`Updated` · `UpToDate` · `Skipped`), `latest_tag`(넘겨주기 0 · 302 `Location` `…/tag/vX.Y.Z` 만), `newer`(정수 셋 · 엄격히 큼), `signed_message` · `sha256_hex` · `verify`(ring ed25519),
  `swap`(`.new` 쓰기 → `.old` 로 이름 바꾸기 → 제자리, 실패면 되돌림) · `clean`. HTTP = `ureq 3.4.2`(rustls · `platform-verifier` — OS 인증서 저장소, 해석 1), 시간 제한 연결 5초 ·
  `latest` 10초 · 받기 120초.
- `main.rs` `supervise`: 감시 잠금 뒤 · 앱을 띄우기 전, 트레이 모드(`parent` 없음) · 릴리스 빌드 · `ASSET` 있음이면 `clean` → `check` → 결과를 `update.txt` 에, `Updated` 면 잠금을 놓고 새 exe 를
  인자 없이 띄우고 0. 앱 `start` 가 `update.txt` 를 로그로 옮기고 지운다. `Paths::update_note_file`.
- `src/bin/sign-release.rs`: `keygen`(있으면 거부) · `sign`(→ `.sig`, 내장 공개키와 다르면 실패 코드) · `verify`.
- 키: `sign-release keygen %USERPROFILE%\.coworkagent-release\release.key`(저장소 밖) → 공개키 `c4a16572…7b0818b9` 를 `PUBLIC_KEY` 에. `.gitignore` 에 `*.key`.

## plan 과 다른 점

- 시간 제한: plan "전체 60초" → `latest` 10초 · 받기 120초(11 MB 를 느린 사내망에서 받을 여유). 연결 5초는 그대로.
- `clean` 은 `.old` 와 함께 남은 `.new`(쓰다 끊긴 것)도 지운다.
- 시험 `real_github_latest`(`#[ignore]` — 실제 github.com, 손으로 `--ignored`)를 더했다 — OS 인증서 저장소로 TLS 가 되는지. 2026-10-20 실행: `latest = None`(정식 릴리스 없음 — 결정 22 그대로).

## 확인

- 실제 키로 `sign` → `verify v1.1.0` 맞음 · `verify v1.2.0` 틀림(태그 묶음), `keygen` 두 번째는 "파일이 있습니다"(덮어쓰지 않음).
- 남는 위험(확인 필요): 프록시가 필요한 사내망 — `ureq` 는 `HTTPS_PROXY` 등 환경 변수만 본다(Windows 시스템 프록시 · PAC 안 봄) → 확인 실패 = 지금 버전으로 뜸(업데이트만 안 됨).
  꺼진 네트워크에서는 시작이 연결 시간 제한(5초)만큼 늦을 수 있다.

## 테스트 결과

```text
cargo test -p coworkagent-app --test update update_ → test result: ok. 8 passed; 0 failed
cargo test -p coworkagent-app --test update real_github_latest -- --ignored → latest = None · ok
cargo test --workspace → 80 바이너리 487 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings → 경고 0
```

## 되돌림(2026-10-20 — 사용자 "서명 관리 기능은 rollback … 배포 방법만 그대로 유지", 확인 답 "확인 없음(HTTPS 만)")

- 지움: `src/bin/sign-release.rs` · `update.rs` 의 `SIG_EXT` · `PUBLIC_KEY` · `signed_message` · `sha256_hex` · `verify` · `check` 의 `key` 인자 · app `ring` 의존 ·
  `.gitignore` `*.key` · 개인키 `%USERPROFILE%\.coworkagent-release\release.key`(폴더째) · 빌드된 `sign-release.exe`. 시험의 서명 3(`bad_sig` · `missing_sig` · `replayed_sig`).
- 남김: 정식 릴리스 확인 · 받기 · 교체 · 감시 연결 · `update.txt` · 릴리스 절차(개발자 가이드 2.4 — `gh release create … --latest`).
- 더함(plan 해석 8 — 사용자 확인 필요): 받은 파일이 PE `MZ` 로 시작하지 않으면 바꾸지 않는다(200 HTML 이 exe 자리에 들어가 앱이 뜨지 않는 일 막기). 시험 `update_rejects_non_exe`.

## 걷어냄(2026-10-20 — 사용자 "git release 에 신규파일 등록 시 자동 배포되는 기능은 삭제.")

- 이 stage 의 산출(`update.rs` · `tests/update.rs` · 감시 연결 · `Paths` 업데이트 파일 · `ureq` 의존 · 문서)을 모두 지웠다. 위 기록은 경과로만 남긴다 — plan 개정 기록.
