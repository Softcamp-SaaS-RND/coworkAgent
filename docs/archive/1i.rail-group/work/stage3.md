# stage3 — 끌어 놓기 · 그룹 타일에 놓아 넣기 · Alt + ↑ / ↓ · 이 PC 화면 확인 · 문서

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

상대 타일 · 그룹 타일 · 같은 절 대화의 자리 옮기기(해석 14 · 15), 대화를 그룹 타일에 놓아 넣기(A10), 이 PC 실제 앱 확인(E7), README · 매뉴얼 ·
`PRODUCT.md`. 대상 gate: E1 ~ E7.

## 산출 파일

- `crates/app/web/view.js` — `moveInOrder` · `stepTarget` · `dropKind` · `withOrder` · `movedText` · `groupLeaves` · `groupUnread`.
- `crates/app/web/app.js` — 끌기 정보(`data-drag` · `data-id` · `data-section`) · `orderOf` · `moveItem`(화면 먼저 → `PUT /api/order` → 다시 읽기) ·
  `#rail` · `#conv_sections` 위임 `dragstart` · `dragover`(놓을 자리 선 · 그룹 타일 점선) · `drop` · `dragend` · Alt + ↑ / ↓(초점 유지 · `#order_live`) ·
  `iconFromFile` 을 data URL 로(아래 실측). `index.html` — `#order_live`(`aria-live`). `app.css` — 끌기 표시 · "대화 추가" 목록 줄임.
- 시험 — `view.test.mjs` 11건 추가(75건) · `ux_literals.rs` 문구 4개 추가.
- 문서 — README(그룹 · 자리 옮기기 줄) · user-guide 4장(레일 범위 표 그룹 열 · 4.1 그룹 · 4.2 타일 메뉴 · 4.3 자리 옮기기) · developer-guide(스키마 16 표) ·
  an_app(#1i API · 화면) · an_core(`groups` · `Scope` · 스키마 16) · `PRODUCT.md`(레일 그룹 · 손으로 놓는 자리 규칙).

## 설계 결정

- **브라우저 기본 드래그** — 라이브러리 없음. 끄는 것은 모듈 변수(`dragging`)로 — `dragover` 에서는 `dataTransfer` 를 읽을 수 없다. `effectAllowed =
  "copyMove"`(옮기기 = move · 그룹에 넣기 = copy — `"move"` 만 주면 그룹 타일에 놓을 수 없었다, mock 실측).
- **위임 한 번** — 목록은 5초마다 다시 그려지므로 요소마다 붙이지 않고 `#rail` · `#conv_sections` 에 한 번.
- **옮긴 뒤 초점** — 다시 그린 새 요소를 같은 `data-id` 로 찾아 초점(링크에서 옮겼으면 링크).
- **아이콘 그림 = data URL**(#17 기존 결함) — 객체 URL(`blob:`)은 CSP `img-src 'self' data:` 가 막아 실제 앱에서 "The source image cannot be decoded"
  (mock 은 CSP 가 없어 통과했다). CSP 를 넓히지 않고 `FileReader.readAsDataURL` 로 읽는다.

## 테스트 결과

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 75 · # fail 0
$ cargo test -p coworkagent-app --test ux_literals             → test result: ok. 13 passed; 0 failed
```

mock(`s3.mjs`, 가상 데이터) 17/17 PASS: 그룹 · 상대 타일 끌기 · '나' 상대별 절 = 레일 순서 · 상대 → 그룹 칸 놓기 없음 · 같은 절 대화 끌기 · 상대 타일에서도
같은 앞뒤 · 다른 절 놓기 없음 · 그룹 타일에 놓아 넣기 · Alt + ↓ · 초점 유지 · 알림 문구 · 끝에서 그대로 · Alt + ↑ 그룹 · '나' 끌기 없음 · 새로 고침 뒤
남음 ×2.

**E7 이 PC 실제 앱**(릴리스 빌드 · 스키마 15 → 16, 사본 `.v15.bak` · 실행 전 사본 `coworkAgent.db.pre-1i.bak`, headless Edge, 시험 그룹만 만들고 지움 ·
대화 순서는 원래대로 되돌림 · 실제 상대는 메뉴만 열고 고치지 않음): 단계 a 22/22 + 앱 다시 실행 뒤 단계 b 5/5 = **27/27**, JS 오류 0. 찾아 고친 것 2:
(1) "대화 추가" 목록에서 긴 제목(줄바꿈 포함)이 체크 칸을 0 폭으로 밀어냄 → 체크 칸 `flex: none` · 제목 줄임, (2) 위 아이콘 data URL.

## 다음 단계

적대적 평가 — `tools/check_gates.py` 로 원장 → Codex 적대적 리뷰(최대 2 라운드) → `eval.md`.
