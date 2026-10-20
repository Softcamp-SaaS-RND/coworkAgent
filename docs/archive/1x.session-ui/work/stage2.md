# stage 2 — 웹UI · 문서(세션 생성 · 기록 삭제 ▸ · 나가기 · 사이드바 손잡이 · 머리 부제 · 타일 크기 · 받은 요청 알림 · 효과음)

## 목표

plan stage 2 — 웹UI(`index.html` · `view.js` · `app.js` · `app.css`) · 화면 시험 · 문서(사용자 가이드 · README · 개발 매뉴얼 · PRODUCT 1x 줄 · masterPlan 후보 줄) — gate E3 · E7 ~ E12.

## 산출 파일

- `crates/app/web/index.html` — `data-theme="light"`, `#new` 제목 "세션 생성" · 메시지 · 메시지 종류 칸 삭제 · 단추 "생성", 입력창 `#ask_hint` 삭제, 설정 화면 "타일 크기"(`ui_tile_size`) · 알림
  "효과음"(`notify_sound`), 자동 왕복 도움말 "세션 생성에서".
- `crates/app/web/view.js` — `SESSION_NEW` · `LEAVE_CONFIRM` · `convMenuItems`(기록 삭제 `sub` · 나가기 `confirm`, 닫힌 대화 = 내 글만, 공유 사본 = 나가기만) · `sideSub` · `composeDefaults` · `requestNotice`,
  `newMenu` 첫 항목 = 세션 생성.
- `crates/app/web/app.js` — `applyTheme` 기본 라이트 · `applyTileSize`(`#app.tile-small`), `renderSideHead` → `sideSub`, 세션 생성 제출(본문 없이 → `#c/{id}` · 입력창 초점) · `openCompose`(레일 상대 채움),
  `renderAskHint` 삭제, ⋯ 메뉴 하위 펼침(→ / ←, 넘치면 위로) · `convEndForm`(나가기 = `DELETE` · 기록 삭제 = `POST …/clear`) · 세부정보 공유 "나가기" 도 같은 확인,
  대화 줄 손잡이 `convGrab`(포인터 — 같은 절 옮기기 · 그룹 타일 넣기 `joinGroup`) · 레일만 HTML 끌기 · Alt + ↑/↓ 그대로, 알림 `requestNotice`(같은 tag · `silent` · 누르면 요청 보드) ·
  `chime`(WebAudio 두 음, 첫 입력에서 `AudioContext`), 설정 `ui_tile_size` · `notify_sound`, 남은 "새 대화" 주석 → 세션 생성.
- `crates/app/web/app.css` — 대화 줄 손잡이 자리(`padding-left` · 위치) · 끄는 줄 흐림, 하위 메뉴 들여쓰기 · ▸ 회전, `--rail-w-small` · `#app.tile-small`(넓은 화면만 — 레일 폭 · 24px 타일 ·
  오른쪽 이름 한 줄 · `+ 그룹` 폭 자동).
- 시험: `view.test.mjs` — 새 `conv_menu_items_1x`(옛 `conv_menu_items` 대체) · `side_sub` · `compose_defaults` · `request_notice` · `session_label` · `screen_settings_1x`, 고침 `new_menu` · `ui_polish_new_chat`.
  `crates/app/tests/ux_literals.rs` — 있음 · 없음 표(세션 생성 · 기록 삭제 · 나가기 확인 · 타일 크기 · 효과음 · 라이트 / 새 대화 · 요청 안내 · 메시지 칸 · 이력유지 · 이력삭제 · 지문 · 줄 ☆).
- 문서: [user-guide](../../../manual/user/user-guide.md)(4장 · 7.1 세션 생성 · 7.3 · 7.8 기록 삭제 · 나가기 · 12 알림 · 효과음 · 16 증상) · README · [web-ui](../../../manual/dev/web-ui.md)(+ 용어 표 예외 줄) ·
  [an_app](../../../manual/dev/an_app.md) · [an_core](../../../manual/dev/an_core.md) · [an_agent](../../../manual/dev/an_agent.md) · [an_net](../../../manual/dev/an_net.md) · [developer-guide](../../../manual/dev/developer-guide.md) 8.12 ·
  PRODUCT(1x 줄만 — 다른 세션 수정분 제외) · masterPlan Phase 1 후보 줄 "클립보드 이미지 붙여넣기"(E10).

## 설계 결정

- **하위 메뉴는 그 자리에서 펼친다**(오른쪽 새 상자 대신) — `#float` 하나로 그리는 지금 메뉴 꼴 그대로, 키보드 → 펼침 · ← 접음, 펼쳐 넘치면 상자를 위로.
- **대화 줄은 HTML 끌기를 쓰지 않는다** — 줄 `draggable` 없음 · 링크 `draggable="false"`, `conv_sections` · `agent_convs_list` 의 HTML 끌기 처리기를 뺐다(레일 타일 끌기는 그대로 — 범위 밖).
  옮기기 · 넣기 판정은 기존 `dragInfo` · `dropKind` · `moveItem` 을 그대로 쓴다.
- **작은 타일 = `#app` 클래스** — 레일 폭은 `#app` 격자 첫 칸이라 `#rail` 클래스로는 바꿀 수 없다(plan 은 `#rail` — 같은 결과).
- **효과음은 받은 요청 알림에만, 브라우저 알림 권한과 무관** — 서버가 알림 설정 · 일시 중지 · 집중을 거른 뒤 보낸 것만 소리를 낸다. 브라우저는 첫 입력 전에는 소리를 막는다(autoplay).

### 구현자 해석(사용자 승인 2026-10-20 — 종결 승인)

1. **세부정보의 공유 "나가기" 도 목록에서 삭제** — intent 결정 23("모든 대화 — 나가기 = 확인 뒤 목록에서 삭제")에 맞춰 세부정보 단추도 ⋯ "나가기" 와 같은 확인 · `DELETE` 로 바꿨다
   (전에는 사본을 "공유 해제됨" 으로 남김). `POST …/leave` 는 CLI · API 호환으로만 남는다.
2. **용어 표 예외** — #1p 용어 표는 화면 문구에 "대화방" · "내 글" 을 쓰지 않는데, 사용자가 글자 그대로 준 "대화방이 목록에서 삭제됩니다." · "내 글만 삭제" 는 그대로 쓰고(지시 충실도) web-ui 용어 표에
   예외 줄을 달았다. 내가 쓴 확인 문구는 표를 따른다("대화는 남습니다").
3. **기록 삭제 알림 글** — 제목 "대화 기록 삭제 — {에이전트}" · 본문 "{이름} · {PC}\n{이름} 이(가) 대화 기록을 모두 지웠습니다"(plan "기록 삭제 — {이름}" 과 다름 — 다른 알림과 같게 제목에 에이전트).

### plan 개정(CHECK 보정)

- E8 `kind` 를 세션 생성 폼 안으로 좁혔다 — 대화방 입력창 질의 | 요청 세그먼트의 `aria-label="메시지 종류"` 는 요청 범위 밖(plan 개정 기록).

## 테스트 결과

- node: `# pass 147 · # fail 0`(기준 142 → 새 6 · 옛 `conv_menu_items` 1 대체).
- `cargo test -p coworkagent-app --test ux_literals` → `5 passed`. `python tools/check_readme.py` → `readme ok`.
- 실화면(CDP — Edge headless, 떠 있는 1.0.1 앱 59616 에 작업 폴더의 `index.html` · `app.js` · `view.js` · `app.css` 를 끼움, `PUT /api/order` · `PUT /api/groups/*` 는 가로채 204 — 실제 순서 · 그룹 안 바뀜,
  스크립트 `scratchpad/ui_probe.mjs`):

```text
theme light
side_sub(me) tskim · TSKIM-SAAS-SOFT                     ← 지문 없음(별칭이 있어 이름 · 호스트)
rows { n: 2, handles: 2, stars: 0, draggable: 0, anchorsNoDrag: 2, starHead: true }
group pick {"conv":"1","group":"12",...}
during ghost=true into=12
dragIntercepted false saved [{"path":"/api/groups/12/conversations","body":"{\"add\":[1],\"remove\":[]}"}] status 그룹 plugin 에 넣었습니다
menu { before: ['plugin','DS6','기록 삭제▸','나가기'], opened: [...,'기록 삭제▸','내 글만 삭제','모든 사용자 글 삭제','나가기'], expanded: 'true' }
clear confirm 기록 삭제 · 모든 사용자 · workbook나와 상대 PC 의 대화 기록을 모두 지웁니다. …
leave confirm 대화 나가기 · workbook대화방이 목록에서 삭제됩니다.취소나가기
peer tile { sub: 'saas2 · SAAS2-RND-SERIV', label: 'saas-svr · SAAS2-RND-SERIV · 1시간 전 통신' }   ← 부제에 접속 문구 없음, aria-label 엔 남음
compose { h1: '세션 생성', peer: '1', rail: '1', agents: 1, text: false, submit: '생성' }
small rail width 168
```

- 같은 절 안 줄 옮기기는 이 PC 데이터에 같은 절 줄이 둘 이상 없어 실화면으로 못 봤다(판정 `dropKind` 는 기존 node 시험, 끌기 경로는 그룹 넣기와 같은 `convGrab`).
  처음 메뉴 화면에서 펼친 하위 항목이 화면 아래로 넘쳐 → 펼칠 때 상자를 위로 올리도록 고쳤다(두 번째 화면에서 확인).

## 다음 단계

gate 판정(`check_gates.py`) → 적대적 평가(`codex-peer-review`, 최대 2 라운드) → report · 환류.
