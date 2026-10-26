# plan — 1z-14.ui-touchup

> 근거: [intent](intent.md)(승인 2026-10-26 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-14-ui-touchup`(원 작업 폴더 — worktree 없음). 기준 커밋: `c9459a3`(main — #1z-13 squash).
> 승인: 2026-10-26 — 사용자 지시가 설계를 정함(질문 답 U1 ~ U4), plan 승인 질문 없이 구현(구현자 해석 — report 에 적음).

## 목표

레일 타일 = 사람 · 사람 그룹 그림, 개요 미리보기 카드 테두리 = 대화 유형 세 색, 사이드바 대화 목록의 내 대화 줄 아이콘 지움, 버전 1.4.0, README 사진 새 UI.

### 성공 기준(intent 사본)

- A1: 레일 = 사람 · 사람 그룹 그림(그림 없을 때), 두 테마.
- A2: 미리보기 카드 테두리 = 유형 세 색, 안 읽음 = 그 색 2px, 대비(테두리 = 글자가 아님 — 바탕 대비 3:1 이상).
- A3: 사이드바 대화 목록 내 대화 줄 아이콘 0, 나머지 유형 아이콘 그대로.
- A4: 동작 · 문구 그대로 — 기존 시험 통과(바뀌는 시험 행은 근거와 함께).
- A5: 1.4.0 · README 사진 새 UI · 검증 명령 PASS · 매뉴얼 반영.

## 설계

### 공통 규칙

- 값은 `app.css` 첫 `:root` 토큰만. 새 토큰 = 라이트 이름 + `--dark-*` + `:root[data-theme="dark"]` 대입. 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음.
- 새 함수는 둘레 규약대로(모듈 `export`) — 파일 전용 static 함수를 새로 만들지 않는다. 같은 개념 주석 · 매뉴얼은 함께 고친다. TDD — 시험 먼저.

### stage 1 — 구현 · 시험 · 문서 · 버전

1. 레일(결정 1): `core/widgets.js` 에 `PERSON_SVG` · `GROUP_SVG`(`d86a728` 그림 그대로) · `railFace(icon, group = false)`(그림이 있으면 그림, 없으면 `.tile-shape` 안 그림).
   `rail.js` 의 나 · 상대 · 그룹 타일과 `tiles.js` 타일 편집 미리보기가 `railFace` 를 쓴다. `tileFace` · `rowTile`(머리글자)은 그대로. CSS: `.rail-btn .tile-shape svg` 크기(기본 22px · 작게 16px).
2. 카드 테두리(결정 2 · 3): 새 토큰 `--card-agent: var(--agent)` · `--card-mine`(라이트 #46607E · 다크 #8AA4C4) · `--card-watch: var(--run)`,
   `.pv-card[data-kind="in"|"out"|"shared"]` 테두리 = 그 색, `.pv-card.unseen` = 테두리 2px(색은 유형 그대로 — 강조색 덮어쓰기 지움).
3. 사이드바(결정 4): `sidebar.js` `convItem` 에 셋째 인자 `sideList`(사이드바 대화 목록에서만 참) — 참이고 `c.dir === "out"` 이면 유형 아이콘 없음. `renderConvList` 만 참으로 부른다.
4. 버전 1.4.0 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`, README 1.3.7 줄 → 1.4.0(결정 7).
5. 문서: [web-ui](../../manual/dev/web-ui.md)(레일 타일 · 미리보기 카드 · 사이드바 대화 줄 · 토큰 표) · [사용자 가이드](../../manual/user/user-guide.md)(레일 · 유형 표시) · PRODUCT(#1z-14 줄) ·
   masterPlan 환류(종결 — S4 레일 타일 · 대화 유형 · D35 미리보기).

### stage 2 — 실제 화면 · README 사진

- 디버그 빌드 + scratchpad 시험 데이터(#1z-13 E7 와 같은 사본), ego-browser — 레일(기본 · 작게) · 개요 미리보기(안 읽음 · 유형 셋) · 사이드바(나 · 상대 · 그룹 타일) × 라이트 · 다크, 콘솔 오류 0.
- README 사진 셋 다시: `docs/images/workspace.png`(대화 · 세부정보 · @멘션 후보, 1440 × 900 @2x) · `pairing.png`(설정 → 연결 상세) · `agents.png`(설정 → 에이전트) — 시험 데이터의 가짜 이름만.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`:
  - 바뀜: `tileFace(g.icon, g.name)` · `tileFace(state.settings.me_icon, meName)` · `tileFace(p.icon, displayName(p)),`(레일 머리글자) → `railFace(…)`, `.pv-card.unseen { border-color: var(--accent); }` → 유형 색 2px,
    `kindChip(c, "room-kind"),` → 사이드바 조건 줄.
  - 지움: ABSENT `PERSON_SVG` · `GROUP_SVG` · 사람 그림 path(#1z-13 결정 2 · 12 — 레일 부분 개정), ABSENT `data-kind="shared"`(카드 관전 테두리 선택자가 쓴다) → 새 ABSENT `.pv-card[data-kind="shared"] { border-style`(점선 없음 그대로) · `tileFace(state.settings.me_icon`(레일은 머리글자 아님).
  - 새 PRESENT: `railFace` · 카드 테두리 셋 · `--card-mine` 토큰 · 사이드바 조건.
- `css_tokens.rs` 그대로. `token_contrast` 에 내 대화 테두리 `--card-mine` × `--bg` · `--surface` × 두 테마(3:1 — 글자가 아닌 선) 4 경우 더함(78 → 82, 청록 = `--accent` · 호박 = `--run` 은 이미 4.5:1 짝).

## 평가 항목

- [ ] E1: A4 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A4 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A4 — 웹UI 로직 시험
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3 — 화면 원본 literal(레일 그림 · 카드 테두리 · 사이드바 조건) · 토큰 대비 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A4 — 문구 그대로: `index.html` 한글 글이 기준 커밋 `c3568f5` 와 같다
  CHECK: python docs/archive/1z-13.design-taste/design/labels.py c3568f5
  EXPECT: /labels n=[1-9][0-9]* missing=0 added=0/
  EVIDENCE: pending
- [ ] E6: A5 — 버전 1.4.0(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v140=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.0' for t in v)))"
  EXPECT: crates=4 v140=4
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A5 — 실제 화면(디버그 빌드 · ego-browser) 레일 · 미리보기 · 사이드바 × 두 테마, 콘솔 오류 0, README 사진 셋 새 UI
  EVIDENCE: pending
- [ ] E8: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d'%(r('docs/manual/dev/web-ui.md').count('--card-mine'),r('docs/manual/user/user-guide.md').count('1.4.0')+r('docs/manual/user/user-guide.md').count('사람 그룹'),r('PRODUCT.md').count('#1z-14')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]*/
  EVIDENCE: pending

## 개정 기록

- 없음.
