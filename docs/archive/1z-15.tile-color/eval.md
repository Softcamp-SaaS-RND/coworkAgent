# eval.md — 1z-15.tile-color 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 260440ad250b1f4a7fb262451e6524669cb7747a

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.51s

- [x] E3: A3·A4 — 웹UI 로직 시험(`tileTint`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A3·A4 — 화면 원본 literal(색 단추 · 색 붙임 · 그룹 그림) · 토큰 대비 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

- [x] E5: A1·A2 — 저장(스키마 29 · 무작위 색 · 색 바꾸기 · `me_color`) 시험
  CHECK: cargo test -p an-core tile_color
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 18 filtered out; finished in 0.33s

- [x] E6: A5 — 버전 1.4.0 그대로(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v140=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.0' for t in v)))"
  EXPECT: crates=4 v140=4
  EVIDENCE: crates=4 v140=4

- [x] E7: A1·A2·A3·A4·A5 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 색 저장 · 다시 열기 · 머리글자 네모 색 · 그룹 세 사람, 콘솔 오류 0, README 사진 셋
  EVIDENCE: Claude(구현자) 2026-10-26 — 디버그 빌드 1.4.0(`104e052` 저장 + 웹 원본 stage 2) + scratchpad `ui1z12\data` 사본(스키마 28 → 29 — 있던 상대 셋 · 그룹 둘 무작위 색 저장), ego-browser 1440 × 900 라이트 · 다크 × 상대 · 타일 편집(상대 · 그룹) · 대화 · 주고받은 메시지 · 연결 = 12 화면. 판독: 레일 상대 · 그룹 바탕 = 그 색 토큰 · 흰 글자 · '나' 흑연, 그룹 그림 원 3 · 상대 원 1, 타일 편집 단추 상대 · 그룹 6(지금 색 눌림) · '나' 7(흑연 눌림), 미리보기 = 고른 색, 연결 목록 · 말풍선 · 주고받은 메시지 상대 아바타 = 그 상대 색. 저장 흐름: 상대 1 peer-3 → peer-1 저장 · 새로 읽기 뒤 레일 · 단추 peer-1, '나' peer-2 저장 = 레일 · 프로필 얼굴 peer-2 · API `me_color` peer-2 → 흑연 저장 = 레일 흑연 · 프로필 무채 · API null. 콘솔 · 페이지 오류 0. README 사진 셋 다시(`184aa7b`, 다크 @2x, `readme pop=true errs=[]`). 근거 [work/stage3](work/stage3.md).

- [x] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('core=%d web=%d guide=%d product=%d'%(r('docs/manual/dev/an_core.md').count('스키마 29'),r('docs/manual/dev/web-ui.md').count('tileTint'),r('docs/manual/user/user-guide.md').count('아이콘 색'),r('PRODUCT.md').count('#1z-15')))"
  EXPECT: /core=[1-9][0-9]* web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: core=1 web=2 guide=6 product=1


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

측정 기록: 위 원장은 여섯째 실행(8/8). 첫째(`184aa7b`) = E1 FAIL(`store.rs` peers 열 수 19 — 개정 ①) · E7 pending. 둘째(`26621a3`) = E5 링크 실패(LNK1104 — 리뷰어와 같은 때
  `store` 시험 실행 파일 잠김) · E8 측정 신선도. 셋째(`65230f7`, `CARGO_TARGET_DIR=target\t15` — 다른 디버그 인스턴스가 기본 target 실행 파일을 잡음) = E1 · E2 FAIL —
  E2 는 라운드 1 고침의 clippy 린트(`bool::then` → `then_some`, `2b2c6d5` 에서 고침), E1 은 그 빌드와 함께. 넷째 = E8 신선도만(원장 갱신 전). 다섯째 = E1 rc=101(간헐 —
  실패 시험 이름은 gate 출력에 없음, 바로 단독 `cargo test --workspace` exit 0). 이 Task 안에서 같은 간헐 실패를 단독 실행으로 둘 잡았다: `ux_api` `home_zoom_setting_persists` ·
  앞서 #1z-14 의 `cli_tx` — 둘 다 로컬 API 연결 `os error 10055`(WSAENOBUFS). 이 Task 변경과 닿지 않는 환경 문제로 본다(추론, #1z-14 report-3 과 같은 건).

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_da18a5bade91` · Task `task_31bdb5659215` · Dispatch `ctx_2a183537243f` · 터미널 `term_8d791bc3-718b-4988-a6bb-7ca480c60703`(시작 receipt `input_accepted`).
범위 `bea90b3..26621a3`(self-collect — stage 1 ~ 3 · 개정 ①), 메타 `요청 1z-15-tile-color-r1 · revision 26621a3 · 라운드 1/2`, worker_done `msg_7d61b55e32e2`(outcome `succeeded`).
리뷰어 Dispatch `worker-release` → `retained`(화면에 보이는 터미널 — 프로세스 동작 없음, 터미널 닫기는 사용자 몫).

```text
# Codex Adversarial Review

Target: branch diff against bea90b3
Verdict: needs-attention

bea90b3..26621a3 고정 범위를 읽기 전용으로 검토했다. 그룹설정 목록 색 누락과 상대 복합 PATCH 부분 저장을 확인해 출시 보류 의견이다. 두 결함 수정·회귀 검증이 남으며 앱·브라우저 실동작과 workspace 테스트는 재실행하지 않았다.

Findings:
- [medium] 그룹설정 목록에서 저장된 그룹 색이 유실된다 (crates/app/web/js/settings/groups.js:25-27)
  [확인] 이 줄의 g는 state.groups 원본이 아니라 groupSettingsList 결과다. logic/rail.js:24가 id/name/icon/count/convs만 복사해 color를 버리므로 g.color는 항상 undefined이고 tileTint는 빈 class를 낸다. 고정 소스의 groupSettingsList를 Node에서 실행해 color=peer-4 입력의 출력에서 color가 사라짐을 확인했다. 그림 없는 그룹은 레일·상세와 달리 그룹설정 왼쪽 목록에서 계속 무채색이므로 결정 5·A3를 충족하지 않는다.
  Recommendation: groupSettingsList의 그룹 결과에 color를 보존하거나 원본 그룹에서 조회하라. 색 있는 그룹을 필터 전후 목록으로 렌더해 tinted class가 유지되는 회귀 검증을 추가하라.
- [medium] 복합 상대 PATCH 실패 뒤 색만 영구 저장된다 (crates/app/src/peers_api.rs:269-283)
  [확인] color는 set_peer_color에서 독립 UPDATE로 먼저 커밋되고 alias/icon 및 meta는 뒤에서 별도로 기록한다. 제품 peers.rs의 UPDATE 두 문장을 메모리 SQLite에서 같은 순서로 실행하고 max_page_count=2로 공간 부족을 유도하니, 검증 형식에 맞는 20,033자 icon의 두 번째 UPDATE가 database or disk is full로 실패한 뒤 (alias='old', icon=NULL, color='peer-3')가 남았다. [추론] 같은 저장 실패가 실제 PATCH에서 나면 502·타일 저장 실패를 반환하면서 색만 바뀌며, 포함한 model/effort까지 일부 적용될 수 있다. HTTP 경로 자체는 실행하지 않았다.
  Recommendation: 검증된 color/alias/icon/meta 변경을 동일 연결의 write_tx 안에서 모두 적용하고 성공 시에만 커밋하라. 두 번째 쓰기 실패를 주입해 응답 실패 후 모든 기존 값이 보존되는 API 회귀 검증을 추가하라.

Next steps:
- 그룹 목록의 color 전달 누락 수정 후 레일·목록·상세 색 일치 검증.
- 상대 PATCH 원자화 후 color+icon 및 color+meta 실패 시 전체 롤백 검증.
```

raw JSON: scratchpad `r15_r1_raw.txt`(worker_done 본문 그대로).

판정:

- R1-1(그룹설정 목록 색 유실) — 사실. `js/logic/rail.js` `groupSettingsList` 가 그룹 결과에 `color` 를 싣지 않았다 → 싣음, `view.test.mjs` 그룹설정 목록 시험에 색이 거른 뒤에도 남는지 더함(고치기 전 fail 1 → 뒤 fail 0 · pass 173).
- R1-2(상대 복합 PATCH 부분 저장) — 사실(색을 먼저 따로 쓴 이 Task 의 변경 + 기존 실행 설정 · 타일 차례 쓰기). `Node::set_peer_fields` 하나로 실행 설정 · 타일 · 색을 한 쓰기 트랜잭션에
  (`write_tx`), 옛 `set_peer_meta` · `set_peer_tile` · `set_peer_color` 지움(부르는 곳은 이 API 하나). `ux_api.rs` `peer_and_me_tile_color` 에 표시 이름 쓰기를 DB 트리거로 막아
  함께 준 색 · 그룹이 쓰이지 않는지 더함(ok 50).
- 실행 메모: 같은 때 다른 `target\debug\coworkAgent.exe`(11:50 시작 — 이 세션이 띄운 것 아님, 그대로 둠)가 실행 파일을 잡아 기본 target 빌드가 막혀, 고친 뒤 시험은 `CARGO_TARGET_DIR=target\t15` 로 돌렸다.

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(라운드 1 과 같은 Run · 작업 폴더). Run `run_da18a5bade91` · Task `task_fd7b5debcc52` · Dispatch `ctx_25bafe785b26` ·
터미널 `term_2fb614f2-5348-4a3c-b44c-095c8c0989ab`(시작 receipt `turn_start_unobserved` — 요청 글이 입력 칸에 남아 있어 `terminal send --enter` 로 보냄, 그 뒤 `working`).
범위 `bea90b3..65230f7`(라운드 1 고침 포함), 메타 `요청 1z-15-tile-color-r2 · revision 65230f7 · 라운드 2/2`, worker_done `msg_3e97db1390f2`(outcome `succeeded`).
리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`). 리뷰 중 구현자가 clippy 린트(`bool::then` → `then_some`, `2b2c6d5`)를 고쳤다 — 고정 범위 밖, 동작 같음.

```text
# Codex Adversarial Review

Target: branch diff against bea90b3
Verdict: approve

bea90b3..65230f7을 읽기 전용으로 검토했다. R1-1 색 전달과 R1-2 복합 PATCH 원자화·트리거 회귀 시험을 확인했으며 추가 차단 결함은 입증하지 못했다. 빌드·테스트·브라우저 실행은 재수행하지 않았고 수정 후 전체 평가 갱신과 미실행 환경 검증이 남는다.

Findings:
- (없음)

Next steps:
- 최종 코드 기준 workspace·clippy·E1~E8을 재측정하고 eval.md의 E5 실패·E8 신선도 실패를 해소하라.
- macOS·두 PC·화면 낭독기·서명 빌드 미검증을 종결·배포 판단에 명시하라.
```

raw JSON: scratchpad `r15_r2_raw.txt`.

판정: 지적 0. next steps 1 = 위 원장 재측정(이 절 뒤 실행), 2 = report 한계에 적음. 라운드 상한 2/2 — 더 돌리지 않는다.

## 미조치 지적

없음
