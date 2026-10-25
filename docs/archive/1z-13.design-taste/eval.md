# eval.md — 1z-13.design-taste 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: d3698d7b78686fc84b156aefaa633aa64c896716

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.32s

- [x] E3: A3 — 웹UI 로직 시험(머리글자 · 단추 줄 순서 · 대화 종류)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A1·A2·A3 — 화면 원본 literal(주 단추 흑연 · 머리글자 · 입력창 순서 · 지운 층 · 종류 표시) · 토큰 대비 70 경우 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

- [x] E5: A1·A2 — AI 티 검사: 웹UI 원본 50개에서 그라데이션 · 순검정 · 이모지 · 사람 아이콘 아바타 · 옛 파랑 · 강조 채도 45 % 이상 · 주 단추가 흑연 아님 · 옆 띠 / 자간 / 글자 기호 단추 · 선택 줄 막대 0
  CHECK: python {TASK}/design/taste.py
  EXPECT: taste files=50 checks=9 violations=0
  EVIDENCE: taste files=50 checks=9 violations=0

- [x] E6: A3 — 문구 그대로: `index.html` 한글 글(글 마디 · aria-label · title · placeholder)이 기준 커밋 `c3568f5` 와 같다
  CHECK: python {TASK}/design/labels.py c3568f5
  EXPECT: /labels n=[1-9][0-9]* missing=0 added=0/
  EVIDENCE: labels n=280 missing=0 added=0

- [x] E7: A1·A4 — 실제 화면: 디버그 빌드 + 시험 데이터 · ego-browser 1440 × 900 — 화면 23 × 라이트 · 다크가 Graphite 언어(흑연 주 단추 · 청록 강조 하나 · 머리글자 네모 · 카드 대신 선 ·
  EVIDENCE: Claude(구현자) 2026-10-25 ~ 26 — 디버그 빌드 1.3.7(`24a4607` 웹 원본) + scratchpad `ui1z12\data` 사본 + 시드(즐겨찾기 2 · 북마크 3 · 요청 5 · 공유 1), ego-browser 1440 × 900, plan 목록대로 25곳(앞 줄 '23' 과 다름 — 개정 ⑨) × 라이트 · 다크 = 50장. 판독: 콘솔 · 페이지 오류 0(error · unhandledrejection · console.error 감시 — 자원 404 · CSP 는 세지 않음) · 사람 SVG 0 · 아바타 자리 사람 아이콘 0 · 주 단추 14/14 = `--primary` · 빈 머리글자 0 · 글자 대비 미달 0/4,278 · 채도 있는 색 = 토큰만 · 그라데이션 0 · 판 표시 v1.3.7 · 종류 아이콘 bot/user/eye. 사용자 확인: 버전 1(https://claude.ai/artifact/Ndz7qpArMaEjgDwf845CfT) 답 "대화목록 섹션에서 내가 보낸 메세지는 화살표위 아이콘이 아닌 사람 아이콘으로 변경" · stage4-1 지움 · stage4-2 앱 스위치와 같게 · stage4-3 그대로 → 반영 뒤 버전 2 답 "확인 · 평가 진행"(2026-10-26). R1-1 고침(`d3698d7` — 토큰 값 · 흐림 한 줄)은 다시 찍지 않았다(대비는 `token_contrast` 78 경우가 덮음). 근거 [work/stage4](work/stage4.md).

- [x] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--primary'),r('docs/manual/user/user-guide.md').count('머리글자'),r('PRODUCT.md').count('#1z-13')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: web=2 guide=5 product=5

- [x] E9: A5 — 버전 1.3.7(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v137=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.3.7' for t in v)))"
  EXPECT: crates=4 v137=4
  EVIDENCE: crates=4 v137=4


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

측정 기록: 위 원장은 셋째 실행(측정 커밋 `d3698d7`, 9/9). 첫째 = E7 만 pending(EVIDENCE 적기 전). 둘째 = E1 FAIL(rc=101 — 같은 때 구현자가 잘못 부른 `check_gates.py --help`(인자 분석이 없어 gate 를 그대로 돌림)가 2분에 끊기며 남긴 cargo 와 겹친 실행, 실패 시험 이름은 남지 않음) · E8 FAIL(report · STATE 종결 링크를 archive 전에 씀 — 되돌림). 그 뒤 단독 `cargo test --workspace` exit 0 · 셋째 실행 9/9.

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_794feb778d9a` · Task `task_8422231b896a` · Dispatch `ctx_e73739de4cc4` · 터미널 `term_3bd0ff18-8ed8-4891-b8ca-2f9a9ab15d69`(시작 receipt `input_accepted` · turnStart `observed`).
범위 `d86a728..24a4607`(self-collect — stage 1 ~ 4), 메타 `요청 1z-13-design-taste-r1 · revision 24a460743d0dd9d93ca7add3f26e91192a278480 · 라운드 1/2`, worker_done `msg_7bbeb8f3499f`.
리뷰 전후 HEAD · 다른 세션 파일(PRODUCT.md · 미추적 tech 폴더 security-posture 문서) 해시 같음. 리뷰 중 구현자가 `work/stage4.md` 에 시험 결과 한 줄을 더했다(고정 범위 밖 작업 폴더 — 다음 커밋 `d3698d7` 에 들어감).
리뷰어 Dispatch `worker-release` → `released`.

```text
# Codex Adversarial Review

Target: branch diff against d86a728963d6c9f044a5db6c6ae27be5e92cd77f
Verdict: needs-attention

고정 범위의 코드·명세·시험을 읽기 전용으로 대조했다. 실제 상태의 글자 대비 미달을 70개 대비 시험이 놓치므로 출하 보류다. CSS 조정과 누락 조합 회귀 검증이 남았다.

Findings:
- [medium] 실제 오프라인·올림 상태의 대비 미달을 검사하지 않는다 (crates/app/tests/ux_literals.rs:856-860)
  PAIRS는 머리글자를 avatar-text/avatar-bg로만, run을 surface 위에서만 검사한다. 그러나 app.css:806은 그림 없는 오프라인 프로필 머리글자를 faint/avatar-bg로 바꿔 라이트 4.163:1·다크 4.159:1이 된다. 또한 .tool-btn:hover(:777)와 .ctx-btn[data-level="warn"](:766)가 결합하면 컨텍스트 경고 글자가 run/surface-2로 라이트 4.408:1이다(기준 revision은 5.368:1). 고정 revision의 CSS 값·선택자를 적용한 정적 계산이며 실제 브라우저 재현은 미실행이다. 작은 머리글자와 경고 글자가 4.5:1 계약을 위반해도 E4가 통과한다.
  Recommendation: 오프라인 머리글자는 avatar-text를 유지하고 상태는 점으로 표현하는 등 실제 조합을 4.5:1 이상으로 조정하라. 경고 단추의 hover 배경·글자 조합도 수정하고 두 테마의 faint/avatar-bg 및 run/surface-2 실제 상태를 회귀 시험에 추가하라.

Next steps:
- 대비 조정 뒤 오프라인 프로필과 경고 컨텍스트 단추의 hover·focus를 라이트·다크 실제 앱에서 확인한다.
- 누락된 대비 조합을 추가해 E4를 재검증하고, 변경되는 시험 경우 수는 plan에 명시한다.
```

raw JSON: scratchpad `r1_result13.json`(worker_done 본문 그대로 — 위 render 의 원문).

판정:

- R1-1(medium) — **사실**. 구현자가 다시 계산: `--faint`/`--avatar-bg` 라이트 4.163 · 다크 4.159, `--run`/`--surface-2` 라이트 4.408. stage 2 가 옛 연결 끊김 표시(사람 그림 `opacity: .6`)를
  머리글자 `--faint` 로 옮기며 생긴 조합이고, 경고 글자는 라이트 `--run` 값과 올림 바탕이 만난 조합이다. 같은 종류를 더 찾아 요청 태그(`.tag.ask` — `--run` · `--ok` / `--subtle`)도 짝에 넣었다.
  고침(`d3698d7`) = 연결 끊김 머리글자 흐림 지움(`--avatar-text` — 상태는 점 색) · 라이트 `--run` #8A6A2F → #80632B(올림 4.93 · 바탕 5.62 · 칩 5.11) · `token_contrast` 짝 넷(36 짝 · 78 경우) ·
  `ux_literals` ABSENT 1 · plan 개정 ⑪ · 매뉴얼. RED(`token_contrast` · `literals_absent` FAILED) → GREEN(ok 3 · ok 5). next_steps 첫째(실제 앱 hover · 연결 끊김)는 대비가 토큰 짝으로만
  정해져 시험이 덮는다 — 실제 화면 다시 찍기는 하지 않았다(연결 끊김은 요청 실패 2번이 있어야 켜진다).

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(같은 작업 폴더 · 읽기만). Run `run_794feb778d9a`(라운드 1 과 같은 Run) · Task `task_136e825c0528` · Dispatch `ctx_780f6ccb9f96` ·
터미널 `term_e919ac33-481e-43d4-a742-f775b916084d`(시작 receipt `input_accepted` · turnStart `observed`). 범위 `24a4607..d3698d7`(R1 고침 — 전체 맥락은 라운드 1 범위),
메타 `요청 1z-13-design-taste-r2 · revision d3698d7b78686fc84b156aefaa633aa64c896716 · 라운드 2/2`, worker_done `msg_d346b436df93`. 리뷰 전후 HEAD · 다른 세션 파일 해시 같음.
리뷰어 Dispatch `worker-release` → `released`.

```text
# Codex Adversarial Review

Target: branch diff against 24a460743d0dd9d93ca7add3f26e91192a278480
Verdict: approve

고정 diff 읽기 검토와 독립 대비 계산 결과, 출하를 막을 추가 결함 없음: run/surface-2는 4.4078→4.9336이고 실제 글자 사용 바탕도 4.5:1 이상이다. 다크 토큰·대입은 동일하며 프로필 머리글자는 8.2306/9.1386, 연결 끊김 점·접근 이름은 유지된다. 기존 32짝 보존·4짝 추가·ABSENT 추가로 시험 약화 없고 개정 ⑪은 E4 항목·EXPECT를 유지한 검사 확대이며, 78경우 독립 계산 실패 0이다.

No material findings.

Next steps:
- 선택 줄의 .side-list .title .room-time 및 .pm-time: faint/accent-soft는 시험 밖이나 라이트 4.5546·다크 4.5489로 충족하므로 회귀 짝 추가 권장.
- .rail-dot.focus의 run/rail은 라이트 3.1531:1: 모든 상태 점까지 4.5:1이라는 주장은 성립하지 않지만 비문자 3:1은 충족한다.
- 최종 revision에서 check_gates를 실행해 평가 원장을 갱신하고 라운드 2/2로 종료할 것; 본 검토는 읽기 전용으로 빌드·브라우저 재실행 없이 수행했으며 화면 근거는 work/stage4.md 판독 기록이다.
```

raw JSON: scratchpad `r2_result13.json`.

판정: 지적 0. next_steps 첫째(`--faint`/`--accent-soft` 짝 추가 — 지금 4.55 로 충족) · 둘째(상태 점 = 글자가 아닌 표시 — 3:1 충족, 4.5:1 은 글자 계약)는 결함이 아니라 권고라
라운드 상한(2/2) 뒤 코드를 바꾸지 않고 둔다. 셋째(최종 측정)는 위 원장.

## 미조치 지적

없음
