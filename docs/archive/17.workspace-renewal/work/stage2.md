# stage2 — 웹UI 뼈대(토큰 · 5영역 · 상단 바 · 메뉴 · 사이드바 내비/섹션 · 경로 · 레일 배지 · 내 활동 · 타임라인 탭 제거)

> 근거: [plan](../plan.md) Stage 2. 사용자 검증은 진행 목표대로 18 종결 뒤 일괄(plan 개정 1).

## 목표

메인 화면을 시안 v4(Nocturne)의 5영역으로 다시 짜고 Stage 1 API 를 화면에 잇는다 — 토큰 · 상단 바(통합 검색 칸 · + 새로 만들기 · 알림 ·
프로필) · 레일 안 읽음 배지 · 사이드바(내비 · 제목 검색 · ★ 즐겨찾기 / 대화 / 에이전트 섹션) · 경로 · '받은 글' → '내 활동' · 타임라인 탭 제거 ·
실행 상세 자리. 대상 gate: E3 · E4 · E5.

## 산출 파일

- `crates/app/web/app.css` — 첫 `:root` 를 plan 토큰 표로 다시 썼다(라이트 이름 그대로 · 다크 `--dark-*` 27개, 글꼴 `Noto Sans KR` · `JetBrains Mono`
  첫 이름 — 내려받지 않음, 크기 12 · 13 · 14 · 16 · 20, 반지름 8 · 10 · 6, 컨트롤 32, 그림자도 토큰). `#app` 그리드(`48px` 상단 바 / 레일 `60px` ·
  사이드바 `--side-w` · 손잡이 · 메인 · 실행 상세 `320px`, 숨긴 영역은 칸 폭 0). 상단 바 · 메뉴 팝오버 · 레일 배지 · 사이드바 내비/섹션/행 · 상태 색 ·
  실행 상세(진행 단계 · 대화 정보 · 자동 왕복 막대 · 저장 목록) · 메시지 작업 막대 · 입력창 · 목록 행 강조. 타임라인 규칙 삭제.
- `crates/app/web/index.html` — 상단 바(`#search_q` placeholder "검색 — 메시지 · 대화 · 사람 · 에이전트" · `Ctrl K` · `#new_btn` "+ 새로 만들기" ·
  `#bell` `title="알림"` · `#avatar_btn` `title="프로필 — 표시 이름 · 알림 일시 중지 · 환경설정"`), 사이드바(`#side_nav` · `#conv_sections` ·
  `#side_agents`, `side_link` 삭제), 메인 절 `#activity`(받은 글 절 대신 — 상대 · 종류 5가지 · 본문 검색) · `#saved` · `#agents_view`, 대화 머리글
  (★ `title="즐겨찾기"` · ✎ · 탭 "대화" · "보낸 메시지" · 아이콘 버튼 "원문 보기"(`id="raw_btn"` 하나) · "저장된 항목" · "세부정보"), 입력창(굵게 · 목록 ·
  코드 · "Enter 보내기 · Shift+Enter 줄바꿈" · 보내기), `aside#detail`("실행 상세" · "닫기" · 진행 · 대화 정보(상대 · 상대 에이전트 · 실행 기본값 ·
  제목 "이 PC 에만 저장") · 자동 왕복 · "멈춤" · 저장된 항목). `#tab_timeline` · `#panel_timeline` · `#inbox` 절 삭제. 설정 절은 그대로.
- `crates/app/web/view.js` — `sideNav` · `newMenu` · `profileMenu` · `convSections` · `convMeta` · `unreadByPeer` · `activityRows` · `messageActions` ·
  `insertMarkdown` · `enterSends` · `runSteps` · `searchGroups` · `shortName`(발화자 이름 자르기 — `authorLabel` 이 쓴다) · `formatClock`(HH:MM:SS).
  `route` — `#c/{id}/m/{글}` · `#activity` · `#saved` · `#agents` · `#settings/{절}`(모르는 절 = 처음) 추가, `#inbox` · `#t/{id}` 삭제(안내 화면).
  `tileKeeps` 의 대화 화면 = 대화 · 보낸 메시지. 주석의 '받은 글' · '타임라인' 표기 정리(`timeline()` 함수는 실행 상세 '진행' 이 쓴다).
- `crates/app/web/app.js` — 위 마크업 배선: `+ 새로 만들기` · 프로필 메뉴(메뉴 밖 클릭 · Esc 로 닫기), 알림 일시 중지 켜고 끄기(`PUT notify_paused`,
  종 흐림), 종(→ '나' 의 내 활동, 안 본 활동이 있으면 점), Ctrl+K 검색 칸 초점, 아바타 = '나' 타일 얼굴, 레일 배지(`unreadByPeer`), 사이드바 내비
  (지금 화면 강조 · 내 활동 안 본 수) · 섹션 · 대화 행(`convMeta`) · 에이전트 섹션('나' = 내 에이전트 → 설정, 상대 = `GET /api/peers/{id}/agents` →
  누르면 새 대화 채움), 경로별 화면(`VIEW_EL`) · 그린 뒤 초점(`focusAfter`), 내 활동(`activityRows` · 거르기 · 열 때 열람 표시), 안 본 수(`GET
  /api/activity?limit=1`), 입력창 서식(`insertMarkdown`) · Enter(`enterSends`) · placeholder "{에이전트} 에게 보내기 — markdown 가능", 대화 열면 읽음,
  실행 상세 열고 닫기, 알림 클릭 이동 · SSE 는 종류와 무관하게 다시 그림. 받은 글 · 타임라인 코드 삭제.
- 테스트 — `view.test.mjs` 새 10건(side_nav · new_menu · profile_menu · conv_sections · activity_rows · message_actions · insert_markdown · enter_sends ·
  run_steps · search_groups) + "화면 위치" 새 경로 · "레일 전환" 의 화면 이름(timeline · inbox → sent · activity). `ux_literals.rs` 새 3건
  (workspace_labels · no_received_label · no_timeline_tab — `view.js` 도 읽는다).

## 설계 결정

- **E3 · E4 를 이 단계에서 다 채운다 — plan 편차 2(stage 경계, gate 무관).** plan Stage 표는 E4 를 Stage 2 에만 둔다(Stage 3 은 E3 · E11). 그런데
  E4 `workspace_labels` 는 실행 상세 · 메시지 작업 · 입력창 라벨까지 plan 목록 전부라, 그 라벨이 Stage 2 에 원본에 있어야 한다. 그래서 라벨을 내는
  순수 함수 10종을 모두 여기서 쓰고(E3 31건), 5영역 정적 마크업(실행 상세 절 · 서식 버튼 · 머리글 아이콘 버튼)을 다 둔다. Stage 3 은 app.js 배선만 —
  대화 머리글 ★ · 부제 model/effort, 메시지 작업, 실행 상세 내용, 저장된 항목 · 에이전트 화면, 통합 검색 결과 · `#c/{id}/m/{글}` 위치.
- **내 활동 화면은 이 단계** — '받은 글' 화면을 바꾸는 것(해석 12)이라 받은 글을 지우는 이 단계에서 같이(지우고 비워 두지 않는다).
- **입력창 · 대화 읽음도 이 단계** — 서식 버튼 · Enter 안내가 마크업에 있으니 동작도 같이. 안 읽음 표시를 이 단계에서 그리므로 지우는 `POST …/read`
  (대화를 그릴 때 안 읽음이 있고 탭이 보이면)도 같이.
- **진행 단계 이름** — 내가 보낸 요청은 plan 문구 "요청 보냄" · "응답 받음", 상대가 보낸 요청은 "요청 받음" · "응답 보냄"(받은 대화에서 상대 요청을
  고르면 "보냄" 이 틀린 말). 보낸 쪽은 상대의 대기 · 실행을 모른다(요청 글은 답이 올 때까지 `sent`, 상태 프레임 없음) — 답 전까지 '대기열' 에 둔다.
  아직 실행 전인 단계 제목 = "실행"(plan 의 "실행 완료" | "실행 중" | "실패" 셋 밖의 대기 상태라 더했다).
- **메뉴의 초점 칸** — 새 대화 → `chat_peer`, 자동 왕복 대화 → `chat_relay_agent`(plan), 초대 생성 → `window_btn`, 초대 수락 → `invite_text`(레일 +
  와 같다), 에이전트 등록 → `agent_name`(접힌 `<details>` 를 편다). `new_menu` 테스트가 index.html 에 그 id 가 있는지 본다.
- **알림 클릭(해석 14)** — 페어링 종류(`pair_request` · `auto_paired` · `approved`)는 설정의 연결 목록(`#settings/peers` — 승인 · 삭제하는 곳), 그 밖
  (메시지 · 자동 왕복)은 '나'(전부)로 바꾸고 `#activity`. 종도 '나' 로 바꾸고 내 활동(점 = 전부의 안 본 수).
- **내 활동 열람(해석 2)** — 경로로 화면을 열 때 그 타일 범위('나' = 빈 몸 = 전부)를 본 것으로, 탭이 보일 때만. 5초 새로 고침 · 거르기는 표시하지
  않는다. 열 때 안 본 항목은 표시 뒤 다시 그려도 강조를 남긴다(`actFresh`). '나' 범위에서만 상대 거르기를 쓴다(상대 타일이면 범위가 그 상대).
- **'나' 범위의 대화 행** — 둘째 줄이 `{에이전트} · {상태}`(plan)라 '나' 범위에서는 여러 상대가 섞여 보인다 → 행 툴팁(`title`)에 `{상대} 의 {에이전트}`.
- **상대 에이전트 목록** — 원격 왕복이라 타일을 고를 때만 받는다(새로 고침마다 묻지 않는다, 타일을 다시 누르면 다시 받는다). 실패 = "목록을
  받지 못했습니다"(plan).
- **빈 즐겨찾기 섹션은 숨긴다**(plan 미정 — 빈 제목만 남기지 않는다).
- **'나' 타일 · 아바타 색** — 시안은 보라(`peer-3`)지만 상대 타일 다섯 색과 겹친다 → 15 그대로 `rail-2`.
- **설정 절 경로** — `#settings/{절}`. 전에는 해시를 바꾸기 전에 화면을 먼저 보이고 스크롤했다 → 이제 경로가 그리고 스크롤 · 초점(`focusAfter`).
- **고친 결함(눈 확인으로 발견)** — `.settings-body` 안 sr-only 라벨(절대 위치)이 문서 전체를 스크롤 가능하게 만들어, 절로 가기의
  `scrollIntoView` 가 상단 바를 화면 밖으로 밀어 올렸다 → `.settings-body { position: relative }`(문서 높이 = 창 높이 확인).
- 통합 검색 칸은 이 단계에서 Ctrl+K 초점 · Esc 까지 — 조회 · 결과 상자는 Stage 3(plan "검색 결과").

## 테스트 결과

RED — 테스트를 먼저 써서 실패를 확인했다:

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# SyntaxError: The requested module './view.js' does not provide an export named 'activityRows'
# tests 1
# pass 0
# fail 1
$ cargo test -p coworkagent-app --test ux_literals
test no_timeline_tab ... FAILED
test no_received_label ... FAILED
test workspace_labels ... FAILED
test result: FAILED. 5 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

GREEN(구현 뒤):

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"          # E3
# pass 31
# fail 0
$ cargo test -p coworkagent-app --test ux_literals                       # E4
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
$ python -c "…E5…"                                                        # E5
root=1 violations=0 nocturne=10
$ python -c "…E11…"
targets=29 missing=0
$ cargo test --workspace            # 테스트 실행 파일 43개 전부 ok
$ cargo clippy --workspace --all-targets -- -D warnings                  # Finished
$ python tools/check_ddd_docs.py    # PASS — DDD 문서 규약 위반 없음
$ python tools/check_crates.py      # crates=4 edges=6 violations=0
```

DOM 테스트가 없어(D10 · E12 수동) 두 가지를 더 봤다:

- **id 대조**(scratchpad 스크립트) — app.js 가 찾는 id(`$("…")` · `PREFS` · `VIEW_EL` · `MENUS` · `PANELS` · `for (const id of …)`) 중 index.html 에
  없는 것: `[]`. `node --check` 통과.
- **눈 확인** — 가짜 API(scratchpad `mock_ui.py`, 지어낸 인물 · PC) 위에서 Edge headless(playwright-core)로: 대화(★ 즐겨찾기 · 대화 섹션 · 행 둘째 줄 ·
  실행 중 표시 · "왕복 3/10" · 안 읽음 점, 레일 배지 "안 읽음 3", 종 점, 1440 폭에서 실행 상세 열림, placeholder "guide 에게 보내기 — markdown 가능") ·
  "+ 새로 만들기" 다섯 항목 → 자동 왕복 대화 = `#new` · 초점 `chat_relay_agent` · 프로필 메뉴(알림 일시 중지 = `menuitemcheckbox`) · 종 → 내 활동
  (안 본 항목 강조, 라벨 "자동 왕복 끝 — 상한 도달" · "실패" · "자동 왕복 시작") · 상대 타일(내비에 "주고받은 글", "이도윤의 에이전트" guide · qa-check →
  누르면 `#new` 에 상대 · 에이전트 채움 · 초점 `chat_text`) · `#settings/invite_accept` 새로 읽기(사이드바 숨김, 문서 스크롤 0) · Ctrl+K 초점 ·
  굵게 → "배포 **절차**" · 세부정보로 실행 상세 접기. 라이트 1280 폭에서 실행 상세 열림. page error 0(콘솔 오류는 가짜 서버의 favicon 404 하나).

## 리뷰

`35e2386` 을 서브에이전트가 읽기 전용으로 리뷰했다(커밋 스냅숏을 풀어 node 테스트 · 가짜 API 눈 확인까지). 지적마다 코드로 대조하고, 사실인 것은
고쳤다(테스트로 잡을 수 있는 것은 단언을 먼저 고쳐 실패를 본 뒤 — `enter_sends` · `activity_rows` RED 2건 확인). 새 테스트 함수는 없다(E3 31건 유지).

| # | 지적(요지) | 판단 | 조치 |
|---|---|---|---|
| M1 | Enter 연타 · 키 반복으로 같은 글이 여러 번 POST — 상대 PC 실행이 그만큼 더 돈다. | 사실(실측 — 같은 본문 POST 2건). | `enterSends` 가 `repeat` 을 거른다 + `sendText` 보내는 중 표시(입력창 · 다시 보내기 공용). 단언 `enter_sends`(키 반복). 눈 확인: Enter 3번(하나는 반복) → POST 1건. |
| M2 | 거르기(종류 · 상대 · 본문)를 남긴 채 내 활동을 열면, 보이지 않는 항목까지 본 것으로 처리 — 안 본 수 · 종 점이 지워지고 거르기를 풀어도 강조가 없다. | 사실. | 열 때 거르기를 처음(전체 · 빈 검색)으로 — 본 것으로 표시하는 범위 = 보이는 범위(해석 2 "열면 그 범위"). 눈 확인: '실패' 로 거른 뒤 다른 화면 → 종 = 종류 "전체" · 4행. |
| M3 | 5초 새로 고침이 레일 · 내비를 매번 다시 그려 키보드 초점 · 누르는 중인 클릭을 잃는다(54c9cf0 대비 회귀). | 사실(실측). | `unchanged(box, sig)` — 그리는 데 쓰는 값의 서명이 같으면 건너뛴다(레일 · 내비 · 대화 목록 — 대화 목록의 같은 문제는 이전부터 있던 것이라 함께). 사용자 동작은 서명이 바뀌므로 그대로 그린다. 눈 확인: 레일 · 내비 초점이 6초 뒤에도 그대로. |
| L4 | 여는 요청이 오가는 사이 새로 고침의 `loadActivity()` 가 번호표를 받으면 열람 표시 · 강조가 통째로 빠진다. | 추정(창은 로컬 왕복 시간) — 구조상 가능. | 여는 동안(`actOpening`) 새로 고침의 내 활동 읽기를 건너뛴다. |
| L5 | 상대 에이전트의 실패 답이 내 활동에서 "에이전트" 로만 보인다 — 해석 12 "실패 상태로 보인다" 와 어긋남. | 사실. | 상대 쪽 글이 `error` 면 "{종류} · 실패". 단언 `activity_rows`. |
| L6 | 상태 줄에 글이 있으면 눌려 넘쳐 설정 절 이동이 문서를 5px 스크롤(상단 바 −5). | 사실(실측). | `#status_msg { flex: 0 0 auto }`. 눈 확인: 상태 글이 있을 때 `#settings/peers` → 문서 스크롤 0. |
| L7 | 설정 화면에서 프로필 → "표시 이름 · 아이콘" 은 사이드바가 숨은 채 초점을 줘 초점이 없다. | 사실(실측). | 사이드바가 숨어 있으면 그린 뒤 초점(`focusAfter = "tile_alias"`). 눈 확인: 초점 `tile_alias`. |
| N | Esc 로 메뉴를 닫으면 초점이 단추로 돌아오지 않음 · `pickRail` 이 안 본 수를 비우기 전에 내비를 그림 · 모르는 상대 경로(`#p/9`)가 에이전트 목록을 물음 · `const act` 가 함수 `act` 를 가림 · combobox 역할 없는 검색 칸의 `aria-expanded` · 같은 초 자동 왕복 기록의 열쇠 겹침(추정). | 사실(마지막만 추정). | 모두 고침 — Esc 는 메뉴 안 초점을 연 단추로, 비운 뒤 그림, 페어링된 상대만 물음, 이름 `actSel`, `aria-expanded` 뺌(`aria-controls` 만), 열쇠에 사유 포함. |
| N | `#settings/` · `#c/1/m/` 같은 잘린 경로는 안내 화면. | 사실 · 의도대로(잘린 주소). | 없음. |
| T | `workspace_labels` 는 화면에 안 그려지는 코드로도 통과(일부 라벨이 다른 글자의 일부로도 맞는다) · E3 의 A2 · A6 · A8 은 순수 함수 단위. | 사실 — 목록은 plan 이 정했다(편차 2 와 같은 맥락). Stage 3 에서 `messageActions` · `runSteps` 를 app.js 가 쓰게 됐다. | 기록만. 테스트 제목의 '받은 글 행' → '메시지 행'(gate 파일 아님). |

리뷰 반영 뒤:

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# pass 31
# fail 0
$ cargo test -p coworkagent-app --test ux_literals
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 다음 단계

Stage 3 — 대화 머리글(★ · 부제 model/effort) · 메시지 작업(저장 · 복사 · 다시 보내기) · 실행 상세 내용(진행 · 대화 정보 · 자동 왕복 · 저장된 항목) ·
저장된 항목 · 에이전트 화면 · 통합 검색 결과 · `#c/{id}/m/{글}` 위치.
