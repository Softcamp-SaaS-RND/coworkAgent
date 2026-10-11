# plan — 14.rename-coworkagent

> 근거: [intent](intent.md)(승인 2026-10-11). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).

## 목표

제품 이름 agentNetwork → coworkAgent(사용자 노출 전부).

### 성공 기준(intent 사본)

- A1: 살아 있는 코드 · 문서에 옛 이름(`agentNetwork` · `agentnetwork`)이 없다(archive · 인터뷰 기록 제외).
- A2: 릴리스 빌드 결과가 `coworkAgent.exe` 다.
- A3: 루트 `PROJECT.md` 검증 명령이 통과한다.
- A4: 개명 전 데이터 폴더(`agentNetwork`)가 있으면 첫 실행 때 신원 · DB · 로그를 새 폴더로 이어받는다(개정 — 평가 라운드 1).

## 설계

- 바꿈: `agentNetwork` → `coworkAgent`, `agentnetwork-app` → `coworkagent-app`, `agentnetwork-invite` → `coworkagent-invite`, 알림 태그
  `agentnetwork-` → `coworkagent-`. 대상 = 저장소 전부에서 `docs/archive` · [부트스트랩 인터뷰](../../overview/bootstrap-interview.md) 제외.
- masterPlan D14 · D15 literal 개정 표시 + 개정 기록, PROJECT 목표에 구 이름.

### 옛 데이터 이어받기(개정 2)

- `Paths::legacy_root()` · `Paths::adopt_legacy(legacy)` — 옛 루트의 `instance.lock` 을 먼저 잡고(옛 앱 실행 중이면 아무것도 바꾸지 않고
  오류), 잡은 채로 옛 항목을 새 루트로 옮김(옛 DB · 로그는 새 이름, 잠금 · instance.json 은 제외, 있는 대상은 덮어쓰지 않음) → 옛 루트 지움.
  두 루트 모두 데이터(DB · 신원)면 고르지 않고 충돌 오류. 실패면 시작을 멈추고 사유 — 새 빈 신원으로 시작하지 않는다. `main` 이 `ensure`
  전에 부른다(개정 3).

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 이름 바꾸기 · 문서 | 코드 · 문서 · Cargo.lock | E1 ~ E5 |

## 평가 항목

- [ ] E1: A3 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A3 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — 옛 이름 잔존 없음(코드 · 빌드 · 도구 · PRODUCT · 매뉴얼 — PROJECT · masterPlan 의 "구 이름" 표기 · 옛 데이터 이어받기 상수와 그 테스트는 의도)
  CHECK: git grep -l -i agentnetwork -- crates tools Cargo.toml Cargo.lock PRODUCT.md docs/manual ":!crates/core/src/paths.rs" ":!crates/core/tests/paths.rs" || echo residue=none
  EXPECT: residue=none
  EVIDENCE: pending
- [ ] E6: A4 — 옛 데이터 이어받기(paths 9건: 기존 6 + legacy_data_adopted · legacy_with_empty_new_root_adopted · legacy_conflict_and_running_refused)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 9 passed; 0 failed
  EVIDENCE: pending
- [ ] E4: A2 — 릴리스 실행 파일 이름
  CHECK: cargo build --release -p coworkagent-app --message-format short && cargo metadata --format-version 1 --no-deps
  EXPECT: "name":"coworkAgent"
  EVIDENCE: pending
- [ ] E5: A3 — 화면 로직 node 테스트
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending

## 종결 시 할 일

- 19 acceptance 의 실행 파일 · 데이터 경로 · 시작 실패 파일 이름 갱신, STATE.

## 개정 기록

- 개정 1(2026-10-11, 첫 측정): E3 대상을 코드 · 빌드 · 도구 · PRODUCT · 매뉴얼로 — PROJECT · masterPlan 에 남긴 "구 agentNetwork" 표기는 의도한 기록.
- 개정 2(2026-10-11, 적대적 평가 라운드 1 반영): 옛 데이터 이어받기 추가(intent 결정 "옮기기 없음" 개정 — 이 PC 에 개명 전 데이터가 실제로
  있다). A4 · E6 추가, E3 에서 이어받기 상수 · 테스트 제외.
- 개정 3(2026-10-11, 적대적 평가 라운드 2 반영): 옮기기 전에 옛 `instance.lock` 을 잡고 쥔 채로 항목별 옮기기(폴더째 이름 바꾸기 대신),
  빈 새 루트면 그리로, 두 루트 모두 데이터면 충돌 오류. E6 7 → 9건.
