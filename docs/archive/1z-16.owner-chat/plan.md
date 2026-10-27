# plan — 1z-16.owner-chat

> 근거: [intent](intent.md)(승인 2026-10-27 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-16-owner-chat`(원 작업 폴더 — worktree 없음). 기준 커밋: `1ecc174`(main — v1.4.0 릴리스 뒤).
> 승인: 2026-10-27 — 사용자 "의견 수용. 개발 진행해"(intent 승인과 함께 plan 승인 질문 없이 구현 — 구현자 해석, report 에 적음).

## 목표

받은 대화(내 에이전트가 답함) 입력창의 '질의' · '요청' 단추를 없애고, 받은 · 보낸 대화에서 내가 직접 쓴 말풍선에 '나' 레일 얼굴 + '나' 타일 색 옅은 면을 준다.
요청 의견 칸 안쪽 여백을 입력창과 맞춘다. 버전 1.4.1.

### 성공 기준(intent 사본)

- A1: 받은 대화 입력창에 '질의' · '요청' 단추가 없고, 거기서 보낸 글은 요청이 아니다(`request` false). 보낸 대화 입력창 단추 · 동작은 그대로.
- A2: 받은 · 보낸 대화의 내 사람 글(CLI 글 제외) 말풍선에 '나' 레일 얼굴(흑연 기본 · 고른 색 · 등록한 그림)이 보인다.
- A3: 그 말풍선 면 = '나' 타일 색의 옅은 면 — '나' 색을 바꾸면 따라 바뀌고, 글자 대비 4.5:1 이상(두 테마), 고른 글(`picked`) 강조와 구별된다.
- A4: 요청 의견 칸 안쪽 여백 = 입력창과 같은 값(`--sp-3` · `--sp-4`) — 대화 · 요청 화면 · 보드 모두.
- A5: 그 밖 기존 동작 · 문구 그대로, 검증 명령 PASS, 매뉴얼 · README 반영, 버전 1.4.1.

## 설계

### 공통 규칙

새 함수는 둘레 규약대로(`export` — 파일 전용 static 함수 없음). 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저. 같은 개념 주석 · 매뉴얼은 함께 고친다.
서버 · API · CLI 는 건드리지 않는다(결정 4).

### stage 1 — 화면(web) · 토큰 · 버전

1. **판정 함수**(시험 가능한 순수 함수, `view.test.mjs`):
   - `logic/conv.js` `asksKind(conv)` — 입력창 '질의' · '요청' 단추를 보일지: 받은 대화(`dir === "in"`)면 `false`, 그 밖 `true`.
   - `logic/author.js` `ownHuman(conv, m)` — 내가 직접 쓴 글: 사람 글(`author !== "agent"`) · `mine === true` · CLI 아님(`via !== "cli"`) · 공유 대화 아님.
   - `logic/peer.js` `ownTint(color)` — 말풍선 면 class 열쇠: 여섯 색(`AGENT_COLORS`)이면 그 색, 아니면 `"rail-2"`(흑연).
2. **입력창**(`conv.js` `showConversation` · `composer.js` submit): `.ask-kind` 를 `!asksKind(conv)` 일 때 `hidden`, 숨길 때 고른 값을 "질의" 로 되돌린다.
   보낼 때 `request` = `asksKind(conv) && ask.value === "request"` — 숨은 단추가 남긴 값이 받은 대화로 새지 않게.
3. **말풍선**(`conv.js` `messageEl`): `ownHuman` 이면 아바타 = `railFace(me_icon, false, me_color, true)`(`.avatar`), article class 에 `own own-{ownTint(me_color)}`.
   `messagesSig` 에 `me_icon` · `me_color` 를 더해 '나' 얼굴 · 색을 바꾸면 다시 그린다.
4. **`app.css`**: 토큰 `--own-{rail-2 · agent-tile · peer-1 ~ peer-5}`(옅은 면, 라이트) · `--dark-own-*`(다크) — `:root` 에서만 리터럴, 다크 매핑 블록 두 곳에 짝.
   `.msg.own .bubble` = `background: var(--own-bg)` + `border-color: var(--border-strong)`, `.msg.own-{색}` 이 `--own-bg` 를 정한다.
   `picked` 는 article 바탕(`--accent-soft`)이고 말풍선은 자기 면 + 단추 테두리 선이라 구별된다. (stage 1 보정: 같은 색 테두리는 `color-mix` 가 필요한데
   macOS 12 Safari 15 에 없어 막혀 있다 — `ux_literals` `literals_absent` #1l.)
5. **요청 의견 칸**: `.req-note textarea` 에 `padding: var(--sp-3) var(--sp-4)` — `noteForm` 하나를 대화 · 요청 화면 · 보드가 함께 쓴다.
6. **버전** 1.4.0 → 1.4.1: `crates/{core,net,agent,app}/Cargo.toml` + `Cargo.lock`.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs`: 새 PRESENT(`asksKind` · `ownHuman` · `ownTint` 붙임, `.msg.own .bubble`, `.req-note textarea` 여백), `token_contrast` 에 `text` × `own-*` 일곱 × 두 테마(4.5:1) 더함(`cases` 88 → 102).

### stage 2 — 실제 화면 · 문서

- 디버그 빌드(`python tools/rebuild_run.py`) · ego-browser 두 테마 — 받은 대화 입력창 단추 없음 · 보낸 대화 단추 있음, 내 말풍선 얼굴 · 면(흑연 · 고른 색), 고른 글 강조, 요청 의견 칸 여백, 콘솔 오류 0.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-16 줄) · README 1.4.1 줄 · masterPlan 환류(종결 — S4 말풍선 · D23 입력창).

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3 — 웹UI 로직 시험(`asksKind` · `ownHuman` · `ownTint`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4 — 화면 원본 literal(단추 숨김 · 말풍선 얼굴 · 면 · 의견 칸 여백) · 토큰 대비 · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A5 — 버전 1.4.1(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v141=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.1' for t in v)))"
  EXPECT: crates=4 v141=4
  EVIDENCE: pending
- [ ] E6: A1·A2·A3·A4·A5 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 받은 대화 단추 없음 · 보낸 대화 단추 있음 · 내 말풍선 얼굴 · 면(흑연 · 고른 색) · 고른 글 강조 구별 · 의견 칸 여백, 콘솔 오류 0
  EVIDENCE: pending
- [ ] E7: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · README · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d readme=%d'%(r('docs/manual/dev/web-ui.md').count('ownTint'),r('docs/manual/user/user-guide.md').count('질의'),r('PRODUCT.md').count('#1z-16'),r('README.md').count('1.4.1')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* readme=[1-9][0-9]*/
  EVIDENCE: pending
