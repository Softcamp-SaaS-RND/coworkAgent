# plan — 30.auto-update

> 근거: [intent](intent.md)(승인 2026-10-21 — 결정 1 ~ 8). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md)
> (D12 업데이트 파일 서명 · D36 배포처 · S6 자동 업데이트 · D13 감시 — 종결 때 환류). 브랜치 `task/30-auto-update`. 기준 커밋: `fe4e7d2`(origin/main).
> plan 승인: 2026-10-21(질문 답으로 갈음 — intent 결정 8).

## 목표

서명된 1.1.1 정식 릴리스 등록 · #1y 업데이터 되살리기(배포처 `Softcamp-SaaS-RND/coworkAgent`) · 받은 exe 회사 인증서 Authenticode 확인 · 1.2.0 서명 릴리스.

### 성공 기준(intent 사본)

- A1: `v1.1.1` 정식 릴리스(Latest)에 서명된 `coworkAgent.exe` 가 있고, `releases/latest` 가 `v1.1.1` 로 넘기며 받은 파일 SHA-256 = 등록한 파일(실측).
- A2: 트레이 모드 감시가 앱을 띄우기 전 `Softcamp-SaaS-RND/coworkAgent` 의 최신 정식 릴리스를 확인해 더 새 버전이면 받아 바꾸고 새 버전으로 뜬다 — 실패는 지금 버전(시험 · 검사).
- A3: 받은 exe 는 Authenticode 가 유효하고 서명자 CN 이 `SOFTCAMP CO.,LTD.` 일 때만 바꾼다 — 서명 없음 · 고친 사본 · 다른 서명자 이름은 바꾸지 않고 `.new` 도 남기지 않는다(시험 · 실제 릴리스 실측).
- A4: 1.2.0 서명 릴리스를 Latest 로 등록하고, 업데이터가 든 낮은 버전 빌드가 실제로 그것으로 바뀌어 뜬다(실측).
- A5: 릴리스 절차(빌드 → 서명 → 확인 → 등록)와 자동 업데이트 안내가 개발자 가이드 · 사용자 가이드 · README 에 있다(검사).
- A6: 회귀 없음 — PROJECT.md 검증 명령 PASS.

## 설계

### 릴리스 1.1.1

- `gh release create v1.1.1 <서명 exe 사본> -R Softcamp-SaaS-RND/coworkAgent --target fe4e7d2a4feb481cc5fe3957365785e43c9c485e --latest`(사본 = 빌드가 덮어쓰기 전에 떠 둔 파일, SHA-256 같음).

### 업데이터 — `crates/app`

- `src/update.rs` · `tests/update.rs` = `8610de5^` 파일 그대로 되살린 뒤:
  - `RELEASES` = `"https://github.com/Softcamp-SaaS-RND/coworkAgent/releases"`, 새 `SIGNER` = `"SOFTCAMP CO.,LTD."`.
  - `check(base, asset, current, skip, exe, trusted: impl Fn(&Path) -> io::Result<()>)` — `is_native_pe` 뒤 `{exe}.new` 쓰기 → `trusted(&new)` 실패면 `.new` 지우고
    `Skipped("{tag} 서명 확인 실패 — 바꾸지 않음: …")` → `swap(exe)`(받아 둔 `.new` 를 제자리 — 쓰기는 `check` 로 옮김).
  - `authenticode(path, signer) -> io::Result<()>`(Windows): `WinVerifyTrust`(`WINTRUST_ACTION_GENERIC_VERIFY_V2` · `WTD_UI_NONE` · `WTD_REVOKE_WHOLECHAIN` ·
    `WTD_REVOCATION_CHECK_CHAIN_EXCLUDE_ROOT` · `WTD_STATEACTION_VERIFY`) → `WTHelperProvDataFromStateData` · `WTHelperGetProvSignerFromChain(…, 0, 0, 0)` → 체인 첫 인증서
    `CertGetNameStringW(CERT_NAME_SIMPLE_DISPLAY_TYPE)` = `signer` → `WTD_STATEACTION_CLOSE`. 그 밖 OS = `Unsupported`.
  - 시험: 기존 `update_*` 는 `trusted` = 통과 주입, 새 `update_rejects_untrusted`(거절 → exe 그대로 · `.new` 없음 · 받은 파일을 확인했는지), `authenticode_rejects_unsigned`
    (서명 없는 시험 exe · 없는 파일), `#[ignore]` `real_signed_release`(실제 최신 정식 exe 받기 → 서명 통과 · 다른 서명자 이름 실패 · 한 바이트 고친 사본 실패).
- `src/main.rs` · `src/lib.rs` · `crates/core/src/paths.rs` = `8610de5` 역적용(감시 연결 · `update.txt` · `update-skip.txt` · `settled`) + `check` 에
  `|p| update::authenticode(p, update::SIGNER)`.
- `Cargo.toml`: `ureq = { version = "3.4.2", default-features = false, features = ["rustls", "platform-verifier"] }`, `windows-sys` 기능 `Win32_Security` ·
  `Win32_Security_Cryptography` · `Win32_Security_Cryptography_Catalog` · `Win32_Security_Cryptography_Sip` · `Win32_Security_WinTrust`. 크레이트 버전 넷 1.2.0.

### 릴리스 1.2.0

- squash push(1.2.0 코드) → `cargo build --release -p coworkagent-app` → 선생님 서명(`target/release/coworkAgent.exe`) → `Get-AuthenticodeSignature` Valid · CN · 속성 버전 1.2.0 확인 →
  `gh release create v1.2.0 … --target <origin/main SHA> --latest`.
- 실측(E7): 같은 코드를 버전 1.1.9 로 빌드한 사본을 임시 폴더에서(떠 있는 앱을 끈 뒤) 트레이 모드로 실행 → v1.2.0 받기 · 서명 확인 · 교체 · 새 버전으로 뜸 · `update.txt` →
  로그 "자동 업데이트: Updated(\"v1.2.0\")" 확인, 끝나면 사본 정리 · 원래 앱 다시 실행.

### 문서

- [개발자 가이드](../../manual/dev/developer-guide.md) 2.4(릴리스 절차 · 서명 확인) · 3.1 · 4.1 · 7.1 · 9장, [an_app](../../manual/dev/an_app.md) · [an_core](../../manual/dev/an_core.md),
  [사용자 가이드](../../manual/user/user-guide.md)(2 · 트레이 · 문제 해결), `README.md` · `PRODUCT.md` — `8610de5` 역적용 + 배포처 · 서명 확인 · v1.2.0.
- masterPlan(종결 환류): D12(검증 = 회사 인증서 Authenticode) · D36(배포처 GitHub `Softcamp-SaaS-RND` · 자동 업데이트 앞당김) · S6 · Phase 3 표 · 개정 기록.

## 평가 항목

- [ ] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A6 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A2·A3 — 업데이터 시험(로컬 HTTP · 서명 거절 · 서명 없는 exe)
  CHECK: cargo test -p coworkagent-app --test update
  EXPECT: /test result: ok\. 10 passed; 0 failed; 2 ignored/
  EVIDENCE: pending
- [ ] E4: A1·A3 — 실제 배포처(네트워크): 최신 태그 · 최신 정식 exe 서명 통과 · 다른 서명자 이름 · 고친 사본 실패
  CHECK: cargo test -p coworkagent-app --test update -- --ignored
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A2·A3 — 배포처 · 서명자 · 감시 연결 · 버전 · 의존 코드 실재
  CHECK: python -c "import pathlib as P;u=P.Path('crates/app/src/update.rs').read_text(encoding='utf-8');m=P.Path('crates/app/src/main.rs').read_text(encoding='utf-8');c=P.Path('crates/app/Cargo.toml').read_text(encoding='utf-8');v=sum(P.Path('crates/%s/Cargo.toml'%k).read_text(encoding='utf-8').count('version = \x221.2.0\x22') for k in ('agent','app','core','net'));o=sum(P.Path(f).read_text(encoding='utf-8').count('tskimS') for f in ('crates/app/src/update.rs','README.md','PRODUCT.md','docs/manual/user/user-guide.md','docs/manual/dev/developer-guide.md','docs/manual/dev/an_app.md'));print('rel=%d signer=%d trust=%d check=%d settled=%d ver=%d old=%d ureq=%d wintrust=%d'%(u.count('pub const RELEASES: &str = \x22https://github.com/Softcamp-SaaS-RND/coworkAgent/releases\x22;'),u.count('pub const SIGNER: &str = \x22SOFTCAMP CO.,LTD.\x22;'),m.count('exe, update::verify);'),u.count('if let Err(e) = trusted(&new, &tag) {'),m.count('update::settled(&exe);'),v,o,c.count('ureq = { version = \x223.4.2\x22'),c.count('\x22Win32_Security_WinTrust\x22')))"
  EXPECT: rel=1 signer=1 trust=1 check=1 settled=1 ver=4 old=0 ureq=1 wintrust=1
  EVIDENCE: pending
- [ ] E6: A5·A6 — 문서 규약 · 크레이트 경계 · README · 문서 안내 실재
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_readme.py && python -c "import pathlib as P;d=P.Path('docs/manual/dev/developer-guide.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');r=P.Path('README.md').read_text(encoding='utf-8');print('proc=%d sign=%d user=%d readme=%d'%(d.count('gh release create v{X.Y.Z} target/release/coworkAgent.exe -R Softcamp-SaaS-RND/coworkAgent'),d.count('Get-AuthenticodeSignature = Valid'),g.count('회사 인증서 서명이 아님'),r.count('1.2.0 은 **자동 업데이트**(Windows)')))"
  EXPECT: proc=1 sign=1 user=1 readme=1
  EVIDENCE: pending
- [ ] E7: A1·A4 — 실측: v1.1.1 · v1.2.0 정식 릴리스(서명 · SHA-256 · `releases/latest`), 1.1.9 업데이터 빌드가 임시 폴더에서 v1.2.0 으로 바뀌어 뜸(로그 "자동 업데이트:")
  EVIDENCE: pending

## 개정 기록

- 2026-10-21 · 적대적 평가 R1 반영 — `verify`(서명 + 버전 리소스 제품 · 파일 버전 = 태그) · `trusted(.new, tag)` · `update.txt` 덧붙임. E5 CHECK 보정(EXPECT 그대로 —
  eval-protocol 규약 2): `trust` = `exe, update::verify);`, `check` = `trusted(&new, &tag)`.
