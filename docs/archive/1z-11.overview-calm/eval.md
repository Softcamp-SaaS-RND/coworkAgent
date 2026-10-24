# eval.md — 1z-11.overview-calm 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: d79020ce3413a8bd98e730b6ec91d1b58ab10140

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.30s

- [x] E3: A3 — 웹UI 로직 시험(미리보기 크기 · 말풍선)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A3·A4 — 화면 원본 literal(머리 · 카드 · 말풍선 · 확대 없음 · 다크 대입) · CSS 토큰
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.18s

- [x] E5: A5 — 문서 규약 · 크레이트 경계 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--pv-bg'),r('docs/manual/user/user-guide.md').count('새 글'),r('PRODUCT.md').count('#1z-11')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: web=1 guide=2 product=1

- [x] E6: A1 · A2 · A3 · A4 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 미리보기 절 · 카드 · 말풍선이 시안 C(이름 14px 500 · 말풍선 12px · 확대 없음), 미리보기 크기 100 · 200%
  EVIDENCE: Claude(구현자) 2026-10-24 — 디버그 빌드 + scratchpad `ui1z11\data`(ui1z8 사본 + `seed1z11.py` — 상대 3 · 대화 10 · 안 읽음 · 실행 중 · 오류), ego-browser DOM 판독 + 스크린샷


WEAK: E6 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_f54b9aa8-170f-4dc7-83a2-940349f7aa2f`.
Run `run_f36e552fba87` · Task `task_8e39b20180b2` · Dispatch `ctx_316bc5c4aa81`(시작 receipt `input_accepted`), worker_done(`msg_dc292b1cf371`) 발신 handle 일치 · outcome succeeded.
범위 `e65f617..e9c0f4b`(self-collect), 메타 `요청 1z-11-overview-calm-r1 · revision e9c0f4b · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD e9c0f4b · 추적 파일 변경 없음(구현자의 report.md 초안만 미추적).
리뷰어 Dispatch `worker-release` → `released`(원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: approve

지정 diff를 읽기 전용으로 검토하고 결정 1~7·A1~A5·plan 개정과 CSS 우선순위·상태 매핑·크기를 대조했다. 차단할 신규 결함은 찾지 못했으며 색 토큰 28개 모두 다크 대입되고 계산한 글자 대비 최저는 라이트 5.54:1·다크 5.98:1이다. 실제 화면·화면 낭독기·macOS·두 PC 연동은 이번 리뷰에서 실행하지 않아 런타임 확인은 남는다.

No material findings.

Next steps:
- 화면 낭독기에서 카드 링크 이름과 안 읽음 정보 탐색을 확인한다.
- 기록된 E6 외 macOS·실제 두 PC 연동 검증 범위를 별도로 유지한다.
```

판정: 지적 0 — 고칠 것 없음. 코드가 바뀌지 않아 라운드 2/2 는 돌리지 않았다(같은 revision 재리뷰는 새 정보가 없음). next steps 둘은 이 Task 범위 밖 검증(화면 낭독기 · macOS · 두 PC) — report 에 미실행으로 적음.

## 미조치 지적

없음
