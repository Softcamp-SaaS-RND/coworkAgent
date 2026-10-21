# stage 1 — 에이전트 상위 한단계 탐색 허용

## 목표

intent 결정 1 ~ 5 — 에이전트별 체크박스(기본 끔), 켜면 `--add-dir <바로 위 폴더>`(드라이브 루트 · 홈 · 데이터 루트를 품는 위 제외), 바꾸면 세션 비움.

## 산출 파일

- `crates/core/src/db.rs`(스키마 24) · `agents.rs`(`parent_read` · `add_with` · `update(…, parent_read)`) · `paths.rs`(`parent_read_dir`).
- `crates/agent/src/runner.rs`(`claude_args` — `--add-dir` · `[읽기 범위]`), `crates/app/src/chat_api.rs`(등록 · 수정 · JSON).
- `crates/app/web/index.html` · `app.js` · `view.js`(`PARENT_NOTE`) — 등록 칸 · 상세 체크박스 · "읽는 위 폴더".
- 시험: `crates/core/tests/paths.rs`(`parent_read_dir_guards` · 세대 · 기본 끔 · 스키마 24) · `store.rs`(스키마 상수) · `crates/agent/tests/runner.rs`(`parent_read_adds_dir`) ·
  `parent_read_canary.rs`(새, `#[ignore]` 실제 claude) · `crates/app/tests/chat_api.rs`(등록 · PATCH · `parent_dir`) · `Agent` 리터럴 · `update` 호출 맞춤.
- 문서 — an_core · an_agent · an_app · 개발자 가이드 · 사용자 가이드 6장 · PRODUCT.

## 설계 결정

- `add` 시그니처는 그대로(시험 호출 40여 곳) — 등록만 `add_with`. `update` 는 호출이 적어(앱 1 · 시험 6) 인자를 늘렸다.
- 위 폴더 판단을 `claude_args` 안에서 — 실행 · 압축 모두 `run_claude` 를 지나 한 곳에서, 실행마다(등록 뒤 링크가 바뀌어도) 다시 판단.
- 드라이브 루트 판단은 그대로 경로와 정규화 경로 둘 다 — `D:\a` 가 다른 드라이브 루트로 가는 junction 이면 정규화 쪽이 잡는다.
- canary 의 두 단계 위 시도는 시스템 프롬프트를 가드만으로 바꿔 돌린다 — `[읽기 범위]` 안내("그 위는 읽을 수 없다")가 있으면 모델이 시도조차 안 해
  (실측: 답 없음 · 거부 기록 없음) 막는 것이 안내인지 `--add-dir` 범위인지 가릴 수 없었다. 바꾼 뒤 Read 거부 기록으로 범위가 막는 것을 확인.
- 디버그 빌드 화면 확인은 `COWORKAGENT_DATA_DIR`(시험 데이터 폴더) + `coworkAgent start --parent <PID>` — 떠 있는 사용자 앱을 끄지 않고 따로(피어 포트는 47100 이 차 있으면 임의 포트).

## 테스트 결과

```text
cargo test --workspace                                           → rc 0(모두 ok)
cargo clippy --workspace --all-targets -- -D warnings            → Finished
node --test "crates/app/web/*.test.mjs"                          → pass 160 · fail 0
cargo test -p an-agent --test parent_read_canary -- --ignored    → on_up=(true, false) off_up=(false, true) on_far=(false, true) (읽음, 거부) · 1 passed
실제 화면(ego-browser, 디버그 빌드)                              → 등록 체크 · 상세 "읽는 위 폴더 …\ui052\proj" · 끄고 저장 → 다시 읽기 끔 · Desktop(위 = 홈) = "열지 않음(…)"
```
