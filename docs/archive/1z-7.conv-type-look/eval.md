# eval.md — 1z-7.conv-type-look 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: f8b9474f3e8bf3a4d839baf4f5bcb0e000541807

- [x] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A2·A4 — 웹UI 로직 시험(유형 아이콘 · 색 기본값 · 바뀐 색만 보냄)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A4 — 스키마 26 · 기본 색 · 색만 바꾸면 세대 · 세션 그대로 · API 색 저장 · 목록 밖 400
  CHECK: cargo test -p an-core --test paths schema26 && cargo test -p coworkagent-app --test chat_api agents_crud
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.12s

- [x] E5: A1·A2·A3 — 화면 원본 literal(제목 칩 · 유형 토큰 · 머리 선 · 줄 테두리 · 카드 윤곽선 · data-kind 셋 · 이름 · 아이콘 색 · 닫기 · 로봇 타일, 이름 표 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

- [x] E6: A5 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d webui=%d core=%d app=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.3\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/web-ui.md').count('**대화 유형 색**(#1z-7'),r('docs/manual/dev/an_core.md').count('스키마 26(#1z-7'),r('docs/manual/dev/an_app.md').count('color?}'),r('docs/manual/user/user-guide.md').count('**대화 유형 색**')))"
  EXPECT: ver=4 webui=1 core=1 app=1 guide=1
  EVIDENCE: ver=4 webui=1 core=1 app=1 guide=1

- [x] E7: A1·A2·A3·A4 — 실제 화면: 디버그 빌드(시험 데이터 폴더 — `in` · `out` · `shared` 대화 시드)에서 대화 머리 아이콘 · 선, 사이드바 줄 · 개요 카드 · 목록 줄 색 · 아이콘,
  EVIDENCE: Claude(구현자) 2026-10-23 — `target/debug/coworkAgent.exe start --parent <대기 프로세스>` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z7\data`(사용자 앱 그대로, 상대 2 · 대화 5(in 2 · out 2 · shared 1) · 글 10 sqlite 시드, 에이전트 coder API 등록), ego-browser DOM 판독 · 스크린샷 — 유형 아이콘 · 선 · 줄 · 카드 · 목록 색(라이트 · 다크) · 상세 격자 값 열 611px 일치 · 닫기 초점 · 색 저장 유지 모두 맞음, 개요 목록 공유 줄 점선 결함 1건 수정 뒤 재확인(관측 전문 = plan.md E7 EVIDENCE).


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_c5d0c5a9-ca4c-469f-8465-6fcffc64cb69`.
Run `run_d382c4e87613` · Task `task_8be287d04146` · Dispatch `ctx_b28c9aa9ff65`, worker_done(`msg_a68b46bc3d8a`) 발신 handle 일치 · outcome succeeded. 범위 `fbd7bad..dc3142c`(self-collect),
메타 `요청 1z-7-conv-type-look-r1 · revision dc3142c30ece852b78d11c0901b34b3728bffa29 · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD dc3142c 그대로(그 사이 손댄 것 = 이 eval.md 원장만 — 코드 아님).
리뷰어 Dispatch `worker-release` → `captured`(화면에 보이는 터미널 — 닫기는 사용자 몫). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

지정 revision diff를 읽기 전용으로 검토했습니다. 대화 유형 접근성 누락과 공유 대화 머리의 테두리 결함 때문에 출하 보류를 권합니다. 두 항목 수정 후 실제 화면·접근성 트리 재확인이 필요하며 빌드·시험은 실행하지 않았습니다.

Findings:
- [medium] 개요 카드의 대화 유형을 접근성 이름에도 포함해야 합니다 (crates/app/web/app.js:218-220)
  kindChip은 in/out 유형 정보를 aria-hidden=true로 숨기며 previewCardEl의 링크 aria-label도 `${title} 대화 열기`뿐입니다(1957행). 사용자가 같은 제목으로 이름 붙인 in/out 대화 카드는 화면에서는 사람·로봇으로 구별되지만 접근성 이름에는 유형 차이가 전혀 없어, 스크린리더 사용자는 열기 전에 문의/내 에이전트 응답을 구분할 수 없습니다. 목록용 칩도 같은 helper를 사용합니다. 정적 DOM 생성 경로에서 확인한 누락이며 실제 보조기기 실행은 하지 않았습니다.
  Recommendation: 유형 칩에 role=img와 KIND_NAMES 기반 aria-label을 제공하고, 명시적 aria-label이 있는 미리보기 링크에도 유형 이름을 포함하세요. 같은 사용자 지정 제목의 in/out 카드·목록 접근성 이름을 확인하세요.
- [medium] 공유 대화 머리에는 border-bottom-style만 적용해야 합니다 (crates/app/web/app.css:1012)
  공유 선택자에 border-style:dashed를 적용하면서 .conv-head까지 네 방향 스타일을 모두 켭니다. .view-head(347행)와 .conv-head[data-kind](1003행)는 아래쪽 테두리만 정의하므로 위·좌·우에는 기본 border-width:medium 및 border-color:currentColor가 적용됩니다. 따라서 shared 대화를 열면 명세의 회색 2px 아래 선 외에 본문 글자색의 점선 테두리 세 면이 추가되고, in/out으로 이동할 때 사라집니다. CSS 선언에 따른 정적 판단이며 E7에 기록된 아래 선 관측만으로는 이 결함을 검출하지 못합니다.
  Recommendation: 공유 머리 선택자를 분리해 border-bottom-style:dashed만 적용하고, 줄·카드에만 border-style:dashed를 유지하세요. 라이트·다크에서 shared 머리의 위·좌·우 border-width가 0이고 아래만 2px 회색인지 확인하세요.

Next steps:
- 두 결함 수정 후 같은 제목의 in/out 카드 접근성 이름과 shared 머리 네 방향 computed border를 실제 화면에서 확인하세요.
- 옛 DB·색 단독 PATCH·uid/세션 유지 시험은 수정 검증 때 기존 회귀 항목으로 유지하세요.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 medium 개요 카드의 대화 유형을 접근성 이름에도 | 사실 — 칩 `aria-hidden`, 카드 링크 `aria-label` 은 제목만(같은 제목이면 구분 불가) | 칩 = `role="img"` · `aria-label` `KIND_NAMES`(줄 · 목록 링크 이름에 들어감), 카드 링크 이름 앞에 유형 이름 · `ux_literals` 두 줄. 실제 화면: 카드 "내가 문의 · 배포 일정 확인 요청 대화 열기" · "내 에이전트 응답 · 빌드 오류 원인 분석 대화 열기" · 공유 "회의록 정리 대화 열기", 사이드바 칩 `img:내가 문의` |
| R1-2 medium 공유 대화 머리는 아래 선만 | 사실 — `border-style: dashed` 가 머리의 네 방향을 켬(E7 은 아래 선만 봄) | 머리는 `border-bottom-style: dashed` 따로 · `ux_literals` 한 줄. 실제 화면: shared 머리 위 · 좌 · 우 0px none, 아래 dashed rgb(104,108,113) · out 머리 아래 solid 파랑 |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_d03fb58c-d394-4ab7-9752-f22ce8b13924`.
Run `run_d382c4e87613` · Task `task_05a3bca7744b` · Dispatch `ctx_92ffe1dd66ac`, worker_done(`msg_b54f90529f0c`) 발신 handle 일치 · outcome succeeded. 범위 `fbd7bad..f8b9474`(self-collect),
메타 `요청 1z-7-conv-type-look-r2 · revision f8b9474f3e8bf3a4d839baf4f5bcb0e000541807 · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 HEAD f8b9474 그대로(그 사이 손댄 것 = masterPlan 환류 — 범위 밖 미커밋 문서).
리뷰어 Dispatch `worker-release` → `captured`. 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: approve

지정 범위 diff를 명세 결정 1~9·A1~A5·E1~E7과 읽기 전용 대조했습니다. R1 접근성 이름·공유 머리 테두리 수정과 E7 재관측 기록을 확인했고, 마이그레이션·옛 응답 기본값·색 검증·uid/세션 보존·원격 색 제외·CSS에서 추가 출하 차단 결함은 찾지 못했습니다. 빌드·시험·브라우저 실행은 하지 않았으며 macOS와 실제 상대 PC 두 대 연동 검증은 남았습니다.

No material findings.

Next steps:
- 릴리스 전 macOS 화면·마이그레이션과 실제 두 PC 연동에서 색의 로컬 유지 확인.
```

판정 · 조치: 지적 없음. next step(macOS 화면 · 두 PC 연동)은 이 프로젝트의 기존 조건 대기 항목(19 acceptance · Mac 실기 — D28)에 속한다 — report "남은 것". 상한 2/2 — 재리뷰 없음.

## 미조치 지적

없음
