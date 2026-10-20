# eval.md — 1y.data-update 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 33576f88770ae3f96fe0683460e17da788263376

- [x] E1: A9 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E2: A9 — 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.31s

- [x] E3: A9 — 화면 로직 node 테스트 · 실패 0 · 150 이상
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (15[0-9]|1[6-9][0-9])\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 171.3567

- [x] E4: A1 — 주고받은 메시지 기간 설정 · 거르기
  CHECK: cargo test -p coworkagent-app --test ux_api -- peer_weeks_setting_persists peer_messages_within_weeks
  EXPECT: /test result: ok\. 2 passed; 0 failed/
  EVIDENCE: test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.33s

- [x] E5: A2·A3·A4 — 요청 archive 자동 · 손 · 상태 바뀜 · 이 PC 만 · 인자 · 대기 되돌리기 보완
  CHECK: cargo test -p coworkagent-app --test ux_api -- request_archive_ req_state_back_to_wait
  EXPECT: /test result: ok\. 6 passed; 0 failed/
  EVIDENCE: test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 40 filtered out; finished in 0.60s

- [x] E6: A5 — 스키마 23 · VACUUM 조건 · 시작 실패 없음(`?` 아님)
  CHECK: cargo test -p an-core --test store -- schema23_req_archived vacuum_ && python -c "import pathlib,re;m=pathlib.Path('crates/app/src/main.rs').read_text(encoding='utf-8');print('vacuum_q=%d vacuum=%d'%(len(re.findall(r'vacuum_if_sparse\([^)]*\)\?',m)),min(m.count('vacuum_if_sparse'),1)))"
  EXPECT: vacuum_q=0 vacuum=1
  EVIDENCE: vacuum_q=0 vacuum=1

- [x] E7: A6·A7 — 업데이트 받기 · 실행 파일 꼴 · 교체 · 건너뛰기(로컬 HTTP)
  CHECK: cargo test -p coworkagent-app --test update update_
  EXPECT: /test result: ok\. [6-9] passed; 0 failed/
  EVIDENCE: test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 1 filtered out; finished in 2.07s

- [x] E8: A8 — 릴리스 절차 문서(`--latest`) · 서명 관리 없음(서명 도구 · 공개키 · `ring` 의존 · 추적되는 키 파일)
  CHECK: python -c "import pathlib,subprocess;u=pathlib.Path('crates/app/src/update.rs').read_text(encoding='utf-8');c=pathlib.Path('crates/app/Cargo.toml').read_text(encoding='utf-8');g=pathlib.Path('docs/manual/dev/developer-guide.md').read_text(encoding='utf-8');f=subprocess.run(['git','ls-files'],capture_output=True,text=True).stdout.split();print('bin=%d key=%d ring=%d tracked_keys=%d doc_sign=%d doc_latest=%d'%(pathlib.Path('crates/app/src/bin/sign-release.rs').exists(),'PUBLIC_KEY' in u or 'ED25519' in u,'ring' in c,sum(x.endswith(('.key','.sig')) for x in f),g.count('sign-release'),min(g.count('--latest'),1)))"
  EXPECT: bin=0 key=0 ring=0 tracked_keys=0 doc_sign=0 doc_latest=1
  EVIDENCE: bin=0 key=0 ring=0 tracked_keys=0 doc_sign=0 doc_latest=1

- [x] E9: A1·A3 — 화면 로직: 기간 고르기 · archive 머리 수 · archive 단추
  CHECK: node --test --test-reporter=tap --test-name-pattern="peer_weeks_options|req_head_archive|req_archive_action" crates/app/web/view.test.mjs
  EXPECT: /# pass [3-9]\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 68.68

- [ ] E10: A9 — 문서 규약 · 의존 경계 · 화면 문구 표
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && cargo test -p coworkagent-app --test ux_literals
  EXPECT: crates=4 edges=6 violations=0
  EVIDENCE: FAIL(rc=1) - STATE.md:14 dead link -> docs/tasks/1y.data-update/work/stage2.md / - STATE.md:14 dead link -> docs/tasks/1y.data-update/work/stage3.md / - docs/archive/1y.data-update/eval.md: 측정 커밋 73466642 이후 코드가

- [x] E11: A3·A1 — 실화면(이 PC, CDP): 주고받은 메시지 기간 고르기 → 목록 다시 · 요청 "archive 항목 보기" → 따로 된 절 · archive / archive 해제 단추.
  EVIDENCE: 2026-10-20 CDP(떼어 놓은 디버그 인스턴스 · 요청 3) — weeks n=12 value=4 · 8 고름 → PUT /api/settings · GET /api/messages/peers · 저장 8 / head "2 | 대기 1 | 완료 1 | archive 1" · 체크 끔 · 목록 숨김 / 체크 → "archive — 받은 요청" 절 "오래전 끝난 요청"[archive 해제] / "archive" 누름 → "archive 로 옮김" · 절로 / 다시 들어오면 끔 (work/stage3.md)

- [ ] E12: A6 — 실화면: 업데이터가 든 정식 릴리스(v1.1.0)를 손으로 설치한 PC 가 다음 정식 릴리스를 실행 때 받아 바꾸고 새 버전으로 뜬다(두 릴리스 필요 — 프로토타입 인수 Task(D28)로 이월).
  EVIDENCE: pending


WEAK: E11 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 기존 Codex 세션 `review2`(title `review2 | skills`, handle `term_da923fe5-89f9-438a-ae76-2f2fade5c8b3`, agentIdentity codex, Ready 확인) — `codex-peer-review` 스킬.
`review` 는 Ready 였으나 다른 coordinator 의 T140 리뷰를 막 끝내 그 라운드 2 가 곧 올 상황이라 idle 확정이 어려워 건너뜀(확인 필요 분류). Run `run_db0bbe841ce9` · Task
`task_bc3b06b060b2` · Dispatch `ctx_a3bdae73598c`(`dispatch`(주입 없음) + preamble · spec 파일을 `terminal send` 로 — #1x 와 같은 전달), worker_done payload taskId · dispatchId 일치.
범위 `d8a4b5e..4bf5e3e`(self-collect), 메타 `요청 1y-data-update-R1 · revision 4bf5e3e · 라운드 1/2`. 리뷰 뒤 작업 폴더 HEAD 4bf5e3e 그대로(다른 세션 PRODUCT.md · security-posture.md 외 변경 없음). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against d8a4b5e
Verdict: needs-attention

고정 범위 d8a4b5e..4bf5e3e의 명세·구현·시험을 읽기 전용으로 대조했다. 업데이트 실패 시 실행 경로 상실 2건과 기간 밖 캐시 잔존 1건으로 출시 보류다. 교체 중단 복구·실행 실패 보존·캐시 만료 검증이 남았으며 Windows 실교체와 전체 빌드·시험은 이번 리뷰에서 실행하지 않았다.

Findings:
- [high] 새 실행 파일을 띄우지 못해도 기존 버전으로 기동하지 않는다 (crates/app/src/main.rs:158-163)
  update::check는 MZ 두 바이트만 확인하므로 잘린 PE·다른 아키텍처 파일도 설치하며, 시험 NEW=b"MZ new"도 Updated로 인정한다. 이후 이 spawn이 ERROR_BAD_EXE_FORMAT 또는 접근 거부 등으로 실패하면 오류 파일만 열고 종료한다. 정상 파일은 .old로 이동한 상태라 다음 일반 실행도 실패한다. 서명 부재에 관한 지적이 아니라 손상된 배포/기동 실패가 A7의 기존 버전 기동 보장을 깨는 경로다.
  Recommendation: 설치 전 PE 구조·대상 아키텍처를 검증하고, 새 exe spawn 실패 시 정상 파일을 복구해 기존 앱을 기동한다. spawn 성공 뒤 즉시 종료하는 실패도 준비 확인으로 검출하고 정상 기동 전에는 .old를 지우지 않는다.
- [high] 두 rename 사이 중단되면 정식 실행 경로가 사라진다 (crates/app/src/update.rs:129-132)
  exe→.old 성공 후 .new→exe 전에 프로세스가 강제 종료되거나 PC가 재부팅되면 exe 경로가 없다. 복구는 두 번째 rename이 Err를 반환할 때만 실행하므로 중단 상황에서는 실행되지 않으며, 자동 시작·바로가기는 존재하지 않는 exe를 가리켜 앱과 복구 코드 모두 시작할 수 없다. 두 번째 rename 오류 때도 복구 rename 결과를 무시해 같은 상태를 숨길 수 있다. 개별 rename의 원자성은 이 두 단계 전체의 원자성을 보장하지 않는다.
  Recommendation: 정식 실행 경로가 계속 유효한 Windows 교체 방식으로 바꾸고, 실패 시 복구 결과를 반드시 검사·보고한다. 각 파일 작업 직후 중단을 주입해 다음 일반 실행이 기존 또는 새 버전으로 기동하는지 검증한다.
- [medium] 기간 설정 변경 때만 캐시를 비워 시간 경과로 만료된 글이 남는다 (crates/app/web/app.js:3903-3905)
  새 기능은 기간을 바꿀 때만 peerPages.clear()를 호출한다. 평상시 loadPeerPage는 기존 mergePage(ms, now)를 그대로 쓰며, 서버 첫 페이지가 20건이면 기존의 더 오래된 꼬리를 모두 유지한다. 따라서 더 보기로 받은 글이 시간 경과로 기간 밖이 되어도 5초 새로 고침 후 화면에 남아 A1을 위반한다. 순수 함수 재현에서 서버 첫 페이지 20건과 만료된 캐시 1건을 병합한 결과 rendered=21, expiredRetained=true였다. 다른 창에서 기간을 줄인 경우도 기존 창의 꼬리는 같은 경로로 남는다.
  Recommendation: 서버가 반환하는 기간 기준/설정 세대로 캐시를 무효화하거나, 매 새로 고침에서 받은 꼬리 전체를 동일한 since로 걸러낸다. 20건 이상인 상태에서 시간 경과와 다른 창의 기간 변경을 검증한다.

Next steps:
- 정상 실행 파일을 보존하는 교체·재기동 실패 처리를 보완하고 중단 지점별 복구를 검증한다.
- 기간 경계와 설정 세대가 캐시에도 적용되도록 고친다.
- 수정 후 허용된 라운드 2/2에서 검토하고 정식 릴리스 실교체 검증은 별도로 수행한다.
```

raw JSON: 보관 `scratchpad/review_1y_r1.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 새 exe 를 띄우지 못해도 지난 버전으로 뜨지 않는다(`MZ` 두 바이트만 확인) | 사실 | `is_native_pe`(PE 머리 · 같은 아키텍처) · 새 exe 를 못 띄우거나 새 버전 앱이 시작 실패 · `GaveUp` 이고 `.old` 가 남아 있으면 `rollback`(지금 → `.bad` · `.old` → 제자리) 뒤 지난 버전, 그 태그는 `update-skip.txt` 로 다시 받지 않음, 앱 준비 뒤 `settled` 가 `.old` 지움(다음 시작의 `clean` 은 `.new` · `.bad` 만). 시험 `update_rejects_non_exe`(HTML · `MZ` 만 · 잘림 · 다른 아키텍처) · `update_skips_rolled_back_tag` · `update_rollback_and_settled` · `update_clean_leftovers`. 감시의 되돌리기 분기 자체는 프로세스 시험 없음(함수 단위만) |
| [high] 두 rename 사이 중단 → exe 경로가 사라짐 · 복구 rename 결과 무시 | 사실 · 일부 수용 | 복구 rename 실패를 숨기지 않고 두 오류 · 손으로 바꿀 경로를 알림. 두 이름 바꾸기 사이(마이크로초) 전원 끊김 창은 남는다 — 실행 중 이미지는 덮어쓸 수 없어(Windows) 한 번에 바꿀 길이 없음, `ponytail:` 주석 · 개발자 가이드 2.4 에 손 복구(`.old` → exe) 기록. 중단 주입 시험 없음 |
| [medium] 기간을 바꿀 때만 캐시를 비워 시간이 지나 기간 밖이 된 글이 남는다 | 사실 | `mergePage(fresh, have, page, since)` — 받아 둔 꼬리도 `since`(설정 주 · 서버 `peer_since` 와 같은 셈) 이후만. node `merge_page_since`. 다른 창에서 기간을 바꾼 경우는 이 창 설정을 다시 읽을 때까지 이 창 기간으로 거름(남는 차이) |

수정 뒤 `cargo test --workspace` 79 바이너리 487 passed · 0 failed, clippy 0, node 151.

## 적대적 평가 — 라운드 2/2

평가자: 같은 Codex 세션 `review2`(handle `term_da923fe5-…`) — Run `run_db0bbe841ce9` · Task `task_f35f3743a947` · Dispatch `ctx_8871046e6457`(라운드 1 과 같은 전달), payload taskId · dispatchId 일치.
범위 `d8a4b5e..403b4a4`, 메타 `요청 1y-data-update-R2 · revision 403b4a4 · 라운드 2/2`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against d8a4b5e
Verdict: needs-attention

고정 범위 d8a4b5e..403b4a4와 R1 수정·원장을 읽기 전용으로 대조했다. R1의 교체 중단 창은 미해결이고 새 감시 기동 확인과 다른 창의 기간 변경 처리도 부족해 출시 보류다. merge_page_since 1 passed와 캐시 잔존 재현을 확인했으며 Windows 프로세스 복구·정식 릴리스 실교체는 미실행이고 라운드 2/2 상한으로 추가 재리뷰하지 않는다.

Findings:
- [high] 새 감시의 spawn 성공을 준비 완료로 간주해 복구 감시가 사라진다 (crates/app/src/main.rs:170-174)
  Updated 분기는 Child 핸들을 버리고 즉시 0으로 끝난다. 새로 추가한 pending 기반 rollback은 새 감시가 watchdog::supervise까지 도달해야 실행된다. 코드 경로 추론(프로세스 시험 미실행): 프로세스 생성 후 로더 초기화 실패나 새 감시 자체의 조기 종료가 발생하면 구 감시는 이미 끝났고 새 감시는 rollback에 도달하지 못해 .old가 있어도 앱이 뜨지 않는다. 또한 이 분기는 R1과 달리 spawn 전에 lock을 놓지 않는다. 구 감시가 spawn 직후 150ms 이상 스케줄링 지연되면 새 감시의 InstanceLock::acquire 재시도 4회가 소진되고, 앱이 아직 없어 hand_off도 실패한다. spawn 성공만으로 두 경우를 검출할 수 없다.
  Recommendation: 구 감시가 새 감시·앱의 준비 신호 또는 조기 종료를 확인할 때까지 Child와 복구 책임을 유지한다. 잠금 인계 순서를 명시하고 준비 확인 실패 시 구 버전으로 복구한다. spawn 성공 뒤 즉시 종료와 구 감시 지연을 주입하는 프로세스 시험을 추가한다.
- [high] R1의 두 rename 사이 실행 경로 소실은 여전히 미해결이다 (crates/app/src/update.rs:145-149)
  exe→.old 뒤 .new→exe 전에 종료·재부팅되면 정식 exe 경로가 없다. 보강된 오류 처리는 두 번째 rename이 반환한 Err만 처리하므로 중단에는 실행되지 않으며 자동 시작과 바로가기로 복구 코드도 띄울 수 없다. eval.md도 '사실 · 일부 수용', 중단 주입 시험 없음으로 기록한다. ponytail 주석과 수동 .old 복원 안내는 기존 버전 기동 보장을 복구하지 않으며, 이 실패를 허용하는 사용자 승인도 plan 개정 기록에는 없다.
  Recommendation: 정식 실행 경로를 잃지 않는 교체·복구 설계를 마련하고 중단 지점별 다음 실행을 검증한다. 현 단일 exe·Windows 제약 아래 이를 보장하지 못한다면 미해결 결함으로 유지하고 사용자에게 수동 복구 위험 수용 여부를 명시적으로 결정받는다.
- [medium] 다른 창의 기간 축소는 새 since 필터에도 반영되지 않는다 (crates/app/web/app.js:3160-3162)
  peerSince는 서버가 목록에 적용한 기간이 아니라 이 창의 state.settings.ui_peer_weeks를 사용한다. GET /api/settings는 초기 showSettings에서 읽고 정기 목록 갱신에서는 다시 읽지 않는다. 다른 창이 4주→1주로 바꾸면 서버는 1주로 거르지만 기존 창은 4주 since로 꼬리를 유지한다. 현 revision의 mergePage로 최신 20건과 2주 전 캐시 1건을 병합해 serverWeeks=1, cachedSettingsWeeks=4, rendered=21, expiredRetained=true를 재현했다. 시간 경과에 대한 단위 시험은 통과하지만 R1의 다중 창 경로와 A1은 닫히지 않았다.
  Recommendation: 목록 응답에 서버가 사용한 since 또는 설정 세대를 포함해 캐시에도 같은 기준을 적용하거나 설정 변경을 모든 창에 전파한다. 서버 기간을 줄인 뒤 기존 창에 20건 이상과 오래된 꼬리가 있는 경우를 검증한다.

Next steps:
- 라운드 2/2 미해결 지적을 원장에 남기고 추가 재리뷰는 수행하지 않는다.
- 새 감시 준비 확인·잠금 인계와 교체 중단 복구를 보완하고 프로세스 시험으로 확인한다.
- 다중 창 기간 변경 시 서버와 캐시의 기준을 일치시킨다.
```

raw JSON: 보관 `scratchpad/review_1y_r2.json`.

### 판정 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 새 감시 spawn 성공을 준비로 간주 · Updated 분기가 잠금을 놓지 않음 | 사실(잠금 — R1 수정이 `drop(lock)` 을 빠뜨린 회귀) | Updated 분기에서 잠금을 먼저 놓고 새 감시를 띄운 뒤 `NEW_SUPERVISOR_GRACE`(15초) 동안 지켜보다, 새 감시가 실패 코드로 끝났고 `.old` 가 남아 있으면 직접 `rollback`. 그 뒤 실패는 새 감시가 되돌린다. 프로세스 시험 없음(함수 시험만) |
| [high] 두 rename 사이 중단 — R1 미해결 | 사실 | **미조치** — 아래 |
| [medium] 다른 창의 기간 축소가 `since` 에 반영되지 않음 | 사실 | `loadPeerMessages` 가 요약과 함께 `GET /api/settings` 를 다시 읽어 `ui_peer_weeks` 를 맞춘다(5초 새로 고침마다). node 시험 없음(DOM — #1w R1-D2 와 같은 판단) |

라운드 2/2 상한 — 위 수정 뒤 재리뷰하지 않고 종료(전역 지침). 수정 뒤 `cargo test --workspace` 79 바이너리 487 passed · 0 failed, clippy 0, node 151.

자동 업데이트를 걷어냄(2026-10-20 — 사용자 "git release 에 신규파일 등록 시 자동 배포되는 기능은 삭제.") — 라운드 1 · 2 의 업데이트 지적(R1-H1 · R1-H2 · R2-H1 · R2-H2, 위 미조치였던 "두 rename 사이 중단" 포함)은
대상 코드가 없어 닫힘(원장 `#1y 자동 업데이트`). 기간 지적(R1-M1 · R2-M1)은 반영 그대로.

## 미조치 지적

없음
