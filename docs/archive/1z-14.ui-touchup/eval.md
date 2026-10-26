# eval.md — 1z-14.ui-touchup 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 5ec83ca0944ce4b65e9d54a3d30c84094da2d554

- [x] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

- [x] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s

- [x] E3: A4 — 웹UI 로직 시험
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A3 — 화면 원본 literal(레일 그림 · 카드 테두리 · 사이드바 조건) · 토큰 대비 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

- [x] E5: A4 — 문구 그대로: `index.html` 한글 글이 기준 커밋 `c3568f5` 와 같다
  CHECK: python docs/archive/1z-13.design-taste/design/labels.py c3568f5
  EXPECT: /labels n=[1-9][0-9]* missing=0 added=0/
  EVIDENCE: labels n=280 missing=0 added=0

- [x] E6: A5 — 버전 1.4.0(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v140=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.0' for t in v)))"
  EXPECT: crates=4 v140=4
  EVIDENCE: crates=4 v140=4

- [x] E7: A1·A2·A3·A5 — 실제 화면(디버그 빌드 · ego-browser) 레일 · 미리보기 · 사이드바 × 두 테마, 콘솔 오류 0, README 사진 셋 새 UI
  EVIDENCE: Claude(구현자) 2026-10-26 — 디버그 빌드 1.4.0(`cae5ac3` 웹 원본) + scratchpad `ui1z12\data` 사본, ego-browser 1440 × 900, 라이트 · 다크 × 개요 · 상대 둘 · 작은 타일 = 8 측정(화면 16 장). 판독: 레일 나 · 상대 셋 = 사람 그림(원 1) · 그룹 둘 = 사람 그룹 그림(원 2), 22px · 작게 16px · 카드 테두리 색 = 유형 토큰(in `--card-agent` · out `--card-mine` · shared `--card-watch`) 불일치 0 · 실선 · 안 읽음 폭 = 보통의 2 배(화면 배율 125% 라 계산값 0.8 · 1.6px) · 사이드바 대화 목록 out 줄 아이콘 0 · shared 줄 눈 그대로 · 콘솔 · 페이지 오류 0. README 사진 셋 다시(`88ddbb7` — workspace 2880 × 1800 · pairing 1904 × 1504 · agents 1904 × 958, 다크, 가짜 이름 · 오류 0). 근거 [work/stage2](work/stage2.md).

- [x] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--card-mine'),r('docs/manual/user/user-guide.md').count('1.4.0')+r('docs/manual/user/user-guide.md').count('사람 그룹'),r('PRODUCT.md').count('#1z-14')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: web=2 guide=2 product=2


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

측정 기록: 위 원장은 셋째 실행(측정 커밋 `5ec83ca`, 8/8). 첫째(`88ddbb7`) = E7 만 pending(EVIDENCE 적기 전). 둘째(`5ec83ca`) = E1 FAIL(rc=101 — gate 출력엔 실패 시험 이름이 없음) → 단독 `cargo test --workspace` exit 0 · 셋째 8/8.
원인 찾기로 `cargo test --workspace` 를 네 번 더 돌림: 1 ~ 3 exit 0, 4 exit 101 — `crates/app/tests/cli_tx.rs` `unreachable_peer_stalls_with_code_7_then_wait_resumes`(256 줄 — 로컬 API 요청이 `os error 10055`(WSAENOBUFS · 소켓 버퍼 · 포트 고갈)로 실패).
연속 실행으로 쌓인 소켓 자원 문제로 보이고(추론), 이 Task 변경(웹 화면 원본 · 문서)과 닿는 곳이 없다. 후속은 report 한계에 적음.

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_1651b014d575` · Task `task_195683a88fa0` · Dispatch `ctx_fb6f6513b9ae` · 터미널 `term_d5844379-6e85-43e3-bb59-ef3d314a3a15`(시작 receipt `input_accepted` · turnStart `observed`).
범위 `c9459a3..88ddbb7`(self-collect — stage 1 · 2), 메타 `요청 1z-14-ui-touchup-r1 · revision 88ddbb7 · 라운드 1/2`, worker_done `msg_28d9d67241e9`(payload outcome `succeeded`).
리뷰는 고정 커밋 범위라 작업 폴더 변경(이 eval · masterPlan 환류 · 다른 세션 PRODUCT.md · 미추적 tech 문서)과 무관. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).

```text
# Codex Adversarial Review

Target: branch diff against c9459a3
Verdict: approve

c9459a3..88ddbb7의 결정 1~7·시험 변경·문서·README 사진 셋을 읽기 전용으로 검토했다. 과거 SVG 일치, railFace 분기 실행, 두 테마 카드 대비 5.619~8.543:1을 확인했고 출하를 막을 실질 결함은 찾지 못했다. 전체 gate·실제 브라우저는 재실행하지 않았으며 macOS·두 PC·화면 낭독기·릴리스 서명 검증은 남는다.

Findings:
- (없음)

Next steps:
- 라운드 2/2에서 후속 수정이 있으면 고정 범위로 재검토한다.
- 미실행 플랫폼·접근성·릴리스 검증 결과를 별도 기록한다.
```

raw JSON: scratchpad `r14_r1_raw.txt`(worker_done 본문 그대로).

판정: 지적 0 — 고칠 것 없음. 라운드 2/2 는 돌리지 않음(1 라운드 = 리뷰 + 지적 수정, 수정이 없어 재리뷰 대상 변경 없음).
next steps 의 미실행 검증(macOS · 두 PC · 화면 낭독기 · 서명)은 report 한계에 적음.

## 미조치 지적

없음
