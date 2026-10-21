# eval.md — 30.auto-update 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 9476d79f359c2fa35155ee52597624ed8557342d

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A6 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A2·A3 — 업데이터 시험(로컬 HTTP · 서명 거절 · 서명 없는 exe)
  CHECK: cargo test -p coworkagent-app --test update
  EXPECT: /test result: ok\. 10 passed; 0 failed; 2 ignored/
  EVIDENCE: test result: ok. 10 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 2.02s

- [x] E4: A1·A3 — 실제 배포처(네트워크): 최신 태그 · 최신 정식 exe 서명 통과 · 다른 서명자 이름 · 고친 사본 실패
  CHECK: cargo test -p coworkagent-app --test update -- --ignored
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 10 filtered out; finished in 1.14s

- [x] E5: A2·A3 — 배포처 · 서명자 · 감시 연결 · 버전 · 의존 코드 실재
  CHECK: python -c "import pathlib as P;u=P.Path('crates/app/src/update.rs').read_text(encoding='utf-8');m=P.Path('crates/app/src/main.rs').read_text(encoding='utf-8');c=P.Path('crates/app/Cargo.toml').read_text(encoding='utf-8');v=sum(P.Path('crates/%s/Cargo.toml'%k).read_text(encoding='utf-8').count('version = \x221.3.0\x22') for k in ('agent','app','core','net'));o=sum(P.Path(f).read_text(encoding='utf-8').count('tskimS') for f in ('crates/app/src/update.rs','README.md','PRODUCT.md','docs/manual/user/user-guide.md','docs/manual/dev/developer-guide.md','docs/manual/dev/an_app.md'));print('rel=%d signer=%d trust=%d check=%d settled=%d ver=%d old=%d ureq=%d wintrust=%d'%(u.count('pub const RELEASES: &str = \x22https://github.com/Softcamp-SaaS-RND/coworkAgent/releases\x22;'),u.count('pub const SIGNER: &str = \x22SOFTCAMP CO.,LTD.\x22;'),m.count('exe, update::verify);'),u.count('if let Err(e) = trusted(&new, &tag) {'),m.count('update::settled(&exe);'),v,o,c.count('ureq = { version = \x223.4.2\x22'),c.count('\x22Win32_Security_WinTrust\x22')))"
  EXPECT: rel=1 signer=1 trust=1 check=1 settled=1 ver=4 old=0 ureq=1 wintrust=1
  EVIDENCE: rel=1 signer=1 trust=1 check=1 settled=1 ver=4 old=0 ureq=1 wintrust=1

- [x] E6: A5·A6 — 문서 규약 · 크레이트 경계 · README · 문서 안내 실재
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_readme.py && python -c "import pathlib as P;d=P.Path('docs/manual/dev/developer-guide.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');r=P.Path('README.md').read_text(encoding='utf-8');print('proc=%d sign=%d user=%d readme=%d'%(d.count('gh release create v{X.Y.Z} target/release/coworkAgent.exe -R Softcamp-SaaS-RND/coworkAgent'),d.count('Get-AuthenticodeSignature = Valid'),g.count('회사 인증서 서명이 아님'),r.count('1.2.0 은 **자동 업데이트**(Windows)')))"
  EXPECT: proc=1 sign=1 user=1 readme=1
  EVIDENCE: proc=1 sign=1 user=1 readme=1

- [ ] E7: A1·A4 — 실측: v1.1.1 · v1.2.0 정식 릴리스(서명 · SHA-256 · `releases/latest`), 1.1.9 업데이터 빌드가 임시 폴더에서 v1.2.0 으로 바뀌어 뜸(로그 "자동 업데이트:")
  EVIDENCE: pending


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_314bb629-1744-49d3-be0d-60606165d4e5`.
Run `run_f5af1a8178b0` · Task `task_9bda12a2d72d` · Dispatch `ctx_b8345759f765`, worker_done(`msg_f9afd3eabd4c`) 발신 handle 일치. 범위 `fe4e7d2..2c46325`(self-collect),
메타 `요청 30-auto-update-r1 · revision 2c46325b72aab18e4d77efe69590c58c94068e32 · 라운드 1/2`. 리뷰 뒤 HEAD 2c46325 그대로(작업 폴더 변경은 masterPlan 종결 초안 · 다른 세션 파일뿐). 리뷰어 Dispatch `worker-release` → `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against fe4e7d2
Verdict: needs-attention

지정 revision의 diff와 intent 결정 1~8·plan을 읽기 전용으로 검토했다. 서명된 구버전 재배포로 업데이트를 끊는 경로와 Updated 로그 덮어쓰기가 있어 출하 보류다. 수정 후 E7 실측이 남으며 제공된 테스트 결과는 재실행하지 않았다.

Findings:
- [high] 서명된 파일 버전을 태그와 결합해 다운그레이드를 차단해야 한다 (crates/app/src/update.rs:64-69)
  newer()는 GitHub 태그만 비교하고, 받은 PE에는 아키텍처와 서명자만 확인한다. 따라서 배포 저장소를 탈취한 공격자가 v9.0.0에 공개된 정상 회사 서명 v1.1.1 coworkAgent.exe를 그대로 첨부하면 1.2.0 클라이언트가 이를 설치한다. intent 결정 5와 stage1 기록상 1.1.1에는 업데이터가 없으므로 구버전으로 내려간 뒤 향후 보안 업데이트도 끊긴다. 이는 서명 키 탈취 없이 가능한 코드 경로상 추론이며 실제 공격 재현은 하지 않았다.
  Recommendation: 검증한 동일 파일의 서명으로 보호되는 PE 제품 식별자·버전을 읽어 coworkAgent이며 current보다 크고 tag와 일치하는지 검사한다. 또는 회사 서명 manifest로 제품·버전·파일 해시를 결합한다. 높은 태그에 정상 서명 구버전을 첨부하는 거절 시험을 추가한다.
- [medium] 새 감시가 이전 감시의 Updated 결과를 덮어쓰지 않게 해야 한다 (crates/app/src/main.rs:167-175)
  이전 감시는 Updated(tag)를 update.txt에 쓴 뒤 새 exe의 감시를 실행한다. 새 감시도 앱을 띄우기 전에 같은 check와 무조건 쓰기를 실행하므로, 정상적으로 latest와 버전이 같으면 UpToDate로 덮어쓴다. 앱이 main.rs:239~242에서 읽는 결과는 UpToDate여서 성공한 교체 기록이 사라지고 plan E7이 요구하는 자동 업데이트: Updated("v1.2.0") 로그를 얻을 수 없다. 프로세스 호출 순서로 확인한 결함이며 E7 실측은 미실행이다.
  Recommendation: 업데이트 직후의 새 감시에 handoff 상태를 전달해 중복 확인·결과 덮어쓰기를 건너뛰거나, 교체 이벤트를 별도 내구 기록으로 보존한다. E7에서 이전 버전→새 감시→새 앱 전체 흐름과 Updated 로그를 함께 확인한다.

Next steps:
- 서명된 구버전 재배포 거절과 Updated 이벤트 보존을 수정한다.
- E7의 1.1.9→서명된 1.2.0 실제 교체·시작·로그를 검증한다.
- FFI VERIFY/CLOSE 수명은 코드상 유지되며 동일 CN 허용은 intent 결정 3의 명시적 수용 사항으로 별도 결함 처리하지 않았다.
```

판정 · 조치(`c88f3bf`):

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 high 서명된 옛 버전을 높은 태그에 붙여 내려 깔기 | 사실 — 태그만 비교, 받은 파일은 서명자만 확인 | `update::verify` = `authenticode` 뒤 `identity`(서명이 덮는 버전 리소스 `ProductName` = `coworkAgent` · 파일 버전 = 태그). 실제 v1.1.1 exe 로 `v99.0.0` 태그 · 다른 제품 이름 거절 확인(`real_signed_release`) |
| R1-2 medium 새 감시가 `Updated` 결과를 `UpToDate` 로 덮음 | 사실 — 두 감시 모두 `std::fs::write` | `update.txt` 덧붙임(`note`) · 앱 시작이 줄마다 로그. E7 실측에서 확인 예정 |

시험: `cargo test -p coworkagent-app --test update -- --include-ignored` 12 passed, `cargo clippy --workspace --all-targets -- -D warnings` Finished, gate 재측정(위 원장).

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_ce4b271e-2c27-4902-9f56-fc9af774def0`.
Run `run_f5af1a8178b0` · Task `task_2047dba22af5` · Dispatch `ctx_de6d559a159a`, worker_done(`msg_7a9e2013c513`) 발신 handle 일치. 범위 `fe4e7d2..5e97252`(self-collect),
메타 `요청 30-auto-update-r2 · revision 5e9725254d1b867f4ac57b8ce5fa680c163df7d3 · 라운드 2/2`. 리뷰 뒤 HEAD 5e97252 그대로. 리뷰어 Dispatch `worker-release` → `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against fe4e7d2
Verdict: needs-attention

고정 범위의 diff·intent 결정 1~8·plan·eval R1을 읽기 전용으로 대조했다. R1의 태그-파일 버전 결합과 Updated 보존은 연결됐지만 정상 시작 확인을 파일 삭제 성공에 종속시킨 되돌리기 결함으로 출하 보류다. Windows 교체 실측 E7과 아래 실패 경로 검증이 남으며 제공된 테스트는 재실행하지 않았다.

Findings:
- [medium] 정상 시작 확인을 .old 삭제 성공과 분리해야 한다 (crates/app/src/update.rs:283-285)
  main.rs:185~193은 이전 exe(.old)의 감시를 최대 15초 살려 두지만 새 앱은 시작 완료 즉시 settled()를 한 번 호출한다(main.rs:359~361). 이때 .old 삭제가 실행 이미지 잠금 등으로 실패하면 오류를 버리고, pending()은 계속 true여서 이미 정상 시작한 버전을 미준비 상태로 취급한다. 이후 앱 재시작이 settled() 전에 실패하면 main.rs:197~201이 정상 적용했던 버전을 되돌리고 update-skip.txt에 그 태그를 영구 제외한다. Windows의 매핑된 파일 삭제 실패는 [DeleteFile 문서](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-deletefile)에 근거하며, 15초 안에 준비되는 실제 업데이트에서의 발생은 코드 경로상 추론으로 실측 확인 필요다. 기존 시험은 실행 중인 이미지가 아닌 일반 파일에 settled()를 호출하므로 이 경로를 검증하지 않는다.
  Recommendation: 준비 완료를 별도 상태/신호로 확정해 pending 판정에 사용하고 .old 삭제는 이전 감시 종료 후 재시도한다. 삭제 실패를 기록하고, 이전 감시가 살아 있는 동안 새 앱이 준비된 후 다음 시작이 준비 전에 실패해도 정상 적용 버전을 되돌리거나 skip 처리하지 않는 Windows 통합 시험을 추가한다.

Next steps:
- 준비 완료 상태와 .old 파일 정리를 분리하고 삭제 실패를 관측 가능하게 만든다.
- E7에서 1.1.9→서명된 1.2.0 교체·Updated 로그와 빠른 시작 시 pending 해제를 실측한다.
- 라운드 2/2로 종료하며 추가 재리뷰는 수행하지 않는다.
```

판정 · 조치(`378f3e0` — 상한 도달, 재리뷰 없음):

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 medium 지난 감시가 `.old` 이미지로 떠 있어 `settled` 삭제 실패 → `pending` 남아 정상 버전 되돌림 · skip | 사실 — 교체한 감시는 `.old` 이름의 이미지로 최대 15초 살아 있고 Windows 는 실행 중 이미지를 지우지 못한다(새 앱은 그 안에 준비될 수 있다) | `settled`: 바로 못 지우면 스레드가 1초마다 최대 60번 다시, 끝내 못 지우면 `warn`(다음 시작의 `settled` 가 다시). 남는 창: 새 앱이 첫 15초 안에 거듭 죽어 `GaveUp` 이면 되돌림 — 그 경우는 뜨지 못한 버전으로 본다. E7 실측에서 `.old` 가 지워지는지 확인 |

## 미조치 지적

- R2-1 의 Windows 통합 시험(지난 감시가 살아 있는 동안 새 앱 준비 → 다음 시작 실패 시 되돌리지 않음) — 원문 "…Windows 통합 시험을 추가한다." · 판정 사실(시험 없음) ·
  미조치 사유: 리뷰 상한 2/2 도달, 프로세스 둘을 띄우는 통합 시험은 E7 실측(실제 교체 뒤 `.old` 삭제 확인)으로 갈음.
- #1y 미조치 1(교체 중단 창 — `swap` 의 두 이름 바꾸기 사이 전원 차단 시 exe 자리 빔) 그대로 이어받음(개발자 가이드 2.4 손 복구).
