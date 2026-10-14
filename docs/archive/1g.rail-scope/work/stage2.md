# stage2 — 웹UI 연결 · 이 PC 화면 확인

> 근거: [plan](../plan.md) Stage 2.

## 목표

`app.js` · `index.html` · `app.css` 에 메뉴 · 상대별 절 · '대화' 요약 · 접힘 · 주소 바꿈을 잇고 사이드바 에이전트 절 · 내 활동 상대 거르기 칸을 없앤다
(A1 ~ A5 · A7). 대상 gate: E1 ~ E6.

## 산출 파일

- `crates/app/web/app.js` — `pairedIds` · `foldEl`(`<details>`, 열쇠 `data-fold`) · `setFolded`(바뀌면 목록 통째로 `PUT /api/settings`) · `grouped` ·
  `refreshHome` · `renderHome`. `renderConvList`('나' = ★ 아래 상대별 절, 절마다 + = 그 상대 새 대화) · `pickTile`(상대 = `#p/{id}`) · `render`(상대 타일인
  채 `#` → `location.replace("#p/{id}")`, `#activity` → '나') · `loadActivity`(상대 거르기 없음, 본 표시 = 전부) · `loadCounts`(상대별 대기 →
  `state.waitingBy`) · `loadRequests` · `loadSaved` · `loadPeerMessages(null)` · `renderAgentsView`('내 에이전트' 절 맨 앞). `renderSideAgents` · 타일
  고를 때 상대 에이전트 미리 받기 제거. 접힘은 `showSettings` 에서 시작할 때만 읽는다.
- `crates/app/web/index.html` — 홈 절 = 머리 "대화" · `#home_summary` · `#home_intro`(상대가 없을 때), `act_peer` · 사이드바 에이전트 절 제거.
- `crates/app/web/app.css` — `.peer-fold`(절 머리 · ▸ ▾) · `details.side-section`.
- 시험 — `ux_literals.rs` `rail_scope_literals`.

## 설계 결정

- **절 클래스 = `peer-fold` · `li.fold`** — 처음 `group` 으로 썼다가 설정 화면 연결 · 에이전트 그룹(`.group`, 테두리 상자)과 겹쳐 목록이 상자로 그려졌다
  (화면 사진으로 발견) → 이름을 바꿨다. 설정 화면은 그대로임을 사진으로 확인.
- **`#` 바꿈은 `location.replace`** — 뒤로 가기가 `#` ↔ `#p/{id}` 사이를 돌지 않게.
- 접힘 토글 이벤트는 처음 그릴 때도 올 수 있다 — `setFolded` 는 값이 같으면 아무것도 하지 않는다.

## 테스트 결과

```text
$ cargo test --workspace → 0 failed(합 349 passed)
$ cargo clippy --workspace --all-targets -- -D warnings → Finished
$ cargo test -p coworkagent-app --test ux_literals → test result: ok. 12 passed; 0 failed
$ node ui_check_1g.mjs(headless Edge · CDP, 실제 앱) → 26/26 PASS
```

이 PC 반영: 실행 중이던 감시 + 앱을 멈추고(DB 사본 `coworkAgent.db.pre-1g.bak`) 릴리스를 빌드해 다시 띄웠다. 화면 확인은 비파괴 — 접힘 설정은 끝에 원래
값으로 되돌렸다. 페어링된 상대가 하나라 절도 하나씩 — 여러 상대 묶음 순서는 node 시험이 잰다. 요청 · 저장된 항목은 이 PC 에 글이 없어 빈 안내만
보였다(묶음 그리기는 내 활동 · 주고받은 글 · 에이전트 · 요약으로 확인).
