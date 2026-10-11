# eval.md — 14.rename-coworkagent 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: b96fd1ad6639e43b7f7534139ce653c649d66a64

- [x] E1: A3 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A3 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.53s

- [x] E3: A1 — 옛 이름 잔존 없음(코드 · 빌드 · 도구 · PRODUCT · 매뉴얼 — PROJECT · masterPlan 의 "구 이름" 표기 · 옛 데이터 이어받기 상수와 그 테스트는 의도)
  CHECK: git grep -l -i agentnetwork -- crates tools Cargo.toml Cargo.lock PRODUCT.md docs/manual ":!crates/core/src/paths.rs" ":!crates/core/tests/paths.rs" || echo residue=none
  EXPECT: residue=none
  EVIDENCE: residue=none

- [x] E6: A4 — 옛 데이터 이어받기(paths 9건: 기존 6 + legacy_data_adopted · legacy_with_empty_new_root_adopted · legacy_conflict_and_running_refused)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

- [x] E4: A2 — 릴리스 실행 파일 이름
  CHECK: cargo build --release -p coworkagent-app --message-format short && cargo metadata --format-version 1 --no-deps
  EXPECT: "name":"coworkAgent"
  EVIDENCE: {"packages":[{"name":"an-agent","version":"0.1.0","id":"path+file:///D:/dev/orca/agentNetwork-t14/crates/agent#an-agent@0.1.0","license":null,"license_file":null,"description":null,"source":null,"depe

- [x] E5: A3 — 화면 로직 node 테스트
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

## 적대적 평가 — 라운드 1/2

- 수단: codex-review 세션 `/codex:adversarial-review --base 4759b20 --scope branch` · 대상 HEAD `36f9110` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 데이터 경로 개명으로 기존 신원·피어·대화·설정이 단절된다 (crates/core/src/paths.rs:14-24)
  `default_root()`와 DB 파일명이 모두 새 이름으로 바뀌었지만 legacy `%LOCALAPPDATA%\agentNetwork`/`agentNetwork.db`를 탐지하거나 이전하는 경로가 없다. `start()`는 이 새 경로에서 DB를 열고 `identity.key`를 생성하므로, 기존 설치를 업데이트하면 빈 DB와 새 ed25519 신원으로 시작한다. 결과적으로 피어 관계, 대화 이력, 설정이 사라진 것처럼 보이고 원격 PC에는 다른 장치로 인식된다. intent의 '기존 데이터가 없다'는 배포 가정일 뿐 코드가 보장하지 않는다.
  Recommendation: 최초 실행 시 legacy 루트를 감지해 DB와 `identity.key`를 원자적으로 이전하거나 legacy 경로를 계속 읽도록 하라. 중단·재시도·rollback을 안전하게 처리하고, 데이터가 채워진 legacy 설치를 업그레이드하는 회귀 테스트를 추가하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 개명 전 데이터 단절 | 사실(이 PC 에 개명 전 데이터 폴더가 실제로 있음) | `Paths::adopt_legacy` — 새 루트가 없으면 옛 루트를 폴더째 이름 바꾸기(같은 볼륨 — 한 번에), 옛 DB · 로그 이름을 새 이름으로(새 이름 있으면 건드리지 않음 — 다시 부르면 이어서 끝남), 실패(옛 앱 실행 중)면 시작을 멈추고 사유. intent 결정 "옮기기 없음" 개정(plan 개정 2). 회귀 `legacy_data_adopted`(신원 · DB · 로그 그대로 · 재호출 · 중단 뒤 마저 · 새 루트 있으면 손대지 않음). 이전 실행 파일로 롤백은 미조치 |

## 적대적 평가 — 라운드 2/2 (상한 — 재평가 없음)

- 수단: codex-review 세션 `/codex:adversarial-review --base 4759b20 --scope branch` · 대상 HEAD `7139450` · 2026-10-11
- Verdict: needs-attention

### 원문

```text
- [high] 실행 중인 legacy 인스턴스를 잠그기 전에 데이터를 이동한다 (crates/app/src/main.rs:46-58)
  `start()`는 `adopt_legacy()`로 디렉터리와 DB·로그 이름을 바꾼 뒤에야 `instance.lock`을 획득한다. macOS/POSIX에서는 열린 파일이 있어도 rename이 성공할 수 있다. 이후 잠금 획득은 `Secondary`가 되지만, 기존 프로세스는 이미 사라진 legacy 경로를 기준으로 SQLite journal 생성·로그 회전을 수행하게 되어 쓰기 실패 또는 복구 데이터 불일치가 발생할 수 있다. 이는 지원 대상 macOS의 정상적인 업그레이드 시나리오다.
  Recommendation: 이동 전에 legacy `instance.lock`을 배타적으로 획득하라. 잠금이 사용 중이면 아무 파일도 변경하지 말고 기존 인스턴스로 handoff하거나 종료 안내를 반환하라. 잠금을 유지한 채 전체 migration을 완료하고, macOS에서 열린 DB·잠금 파일이 있는 상태의 회귀 테스트를 추가하라.
- [high] 새 루트가 존재하면 populated legacy 데이터가 무조건 방치된다 (crates/core/src/paths.rs:53-68)
  이동 조건이 `!self.root.exists() && legacy_root.is_dir()`뿐이다. 두 루트가 함께 있으면 legacy 루트를 전혀 검사하지 않고 새 루트 내부의 옛 파일명만 처리한다. 이전 renamed 빌드의 부분 실행, 실패 후 남은 빈 디렉터리, 복원된 백업 때문에 두 루트가 공존하면 새 DB·신원을 사용하고 기존 피어·대화·설정·신원을 조용히 무시한다. 테스트도 이 동작을 정상으로 고정하지만 A4의 데이터 이어받기 계약을 충족하지 못한다.
  Recommendation: 두 루트 공존을 명시적 migration 상태로 처리하라. 새 루트가 비어 있으면 안전하게 이전하고, DB 또는 `identity.key`가 양쪽에 존재하면 자동 선택하지 말고 충돌 오류와 복구 경로를 제공하라. 빈·부분 채움·충돌 상태별 회귀 테스트를 추가하라.
```

### 대조 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| 옛 인스턴스 잠그기 전 이동 | 사실 | 옛 `instance.lock` 을 먼저 잡고(실행 중이면 아무것도 안 바꾸고 "끄고 다시 실행" 오류), 쥔 채로 항목별 옮기기(잠금 · instance.json 제외) → 놓고 옛 루트 지움. 회귀 `legacy_conflict_and_running_refused`(옛 잠금 쥔 동안 아무것도 안 바뀜 · 놓으면 옮김) — "실행 중 무시" mutation FAIL. macOS 실환경 확인은 D25(Phase 4) |
| 새 루트가 있으면 옛 데이터 무시 | 사실 | 두 루트 모두 데이터(DB · 신원)면 충돌 오류(두 경로와 정리 안내), 새 루트에 데이터가 없으면 그리로 옮김(있는 로그는 덮어쓰지 않음). 회귀 `legacy_with_empty_new_root_adopted` · 충돌 — "충돌 검사 제거" mutation FAIL |

## 미조치 지적

| 지적 | 판단 | 사유 |
|---|---|---|
| 이어받은 뒤 이전 실행 파일로 롤백(라운드 1) | 사실 | 이어받으면 옛 루트를 지운다 — 롤백하려면 새 루트를 옛 이름으로 되돌려야 한다. 배포 전 프로토타입이라 롤백 도구는 Phase 3 업데이트(S6)와 함께 |
| macOS 열린 DB · 잠금 상태 실환경 확인(라운드 2) | 사실 — 잠금으로 막음 | macOS 확인은 D25(Phase 4) |
