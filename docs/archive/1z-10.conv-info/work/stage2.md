# stage 2 — 웹UI 대화 정보 · 아이콘 · 단추 줄 · 문서

## 목표

plan "설계 · stage 2" — 세부정보 '대화 정보' 를 `/info` 줄로, 앱 전체 아이콘 없는 항목에 글자 앞 아이콘(시안 매핑 표), 설정 상세 단추 줄을 내용 바로 아래로(결정 8), 매뉴얼 · PRODUCT.

## 산출 파일

- `index.html` — `#info_rows`(옛 `#info_peer` · `#info_agent` · `#info_defaults` 지움), 세부정보 절 제목 · 상단 · 레일 · 대화 머리 · 입력 · 설정 타일 라벨 · 설정 단추 · 디버깅 · 정적 상세 단추 줄 ·
  에이전트 등록 폼에 `.ico … lead`.
- `app.css` — 새 아이콘 30개(mask SVG), `.ico.lead` · `.set-head h2 .ico.lead`(강조색), `.info-cmd` · `.info-hint`, `.md-body { flex: 0 1 auto }` · `.md-detail:has(…) { align-self: start; max-height: 100% }`.
- `app.js` — `lead` · `button(…, icon)` · `field(…, icon)` · `convAct(…, icon)` · `mdFoot` 닫기 아이콘, 연결 사용자 · 그룹 · 에이전트 상세 라벨 · 단추 아이콘, `renderTree` 가 설정 화면 제목에 메뉴 아이콘,
  `renderInfo` · `drawInfo` · `copyLine`.
- `view.js` — `convInfoRows` · `INFO_CMD_HINT`.
- 시험: `view.test.mjs` `conv_info` 새 시험 · `screen_settings_1x` 기대 마크업 갱신, `ux_literals.rs` 아이콘이 든 줄의 기대 문자열 갱신 · #1z-10 행 추가 · 없어야 할 문구(`info_defaults` · `info_peer` · `>+ 사용자<`).
- 문서: web-ui · an_core · an_app · developer-guide · user-guide · PRODUCT.

## 설계 결정

- 매핑 표 밖으로 넓힘(같은 꼴): 요청 화면 "목록 · 보드" 전환(list · grid) · "상태" 라벨(activity), 연결 상세 "승인"(check) · "다시 알리기"(bell) — 인벤토리 때 데이터가 없어 안 보였던 같은 종류 단추.
- `.set-actions` 단추(에이전트 등록 · 진단 정보 복사)는 기존 규칙(`.set-actions .ico` 여백)이 있어 `lead` 없이 `.ico` 만(그룹 추가와 같은 꼴).
- 디버깅 "수준" 라벨은 화면에 안 보이는 라벨(`sr-only`)이라 선택 칸 앞에 아이콘만 둔다.
- 대화 정보 응답은 대화 · 글 수 · 마지막 글 id · 상태가 바뀔 때만 다시 읽는다(5초 새로 고침마다 읽지 않음). 응답 전 "불러오는 중 …", 다른 대화의 늦은 응답은 버린다.
- 단추 줄: 빈 상세("…을 고르세요")는 칸 전체 그대로(`:has` 가 고른 상세 · 그룹 상세 머리가 있을 때만 줄임).
- 시험 문자열의 "+ 사용자" · "+ 그룹" · "+ 새로 만들기" 기대값을 아이콘 + 글자로 바꿈 — S4(#1z-2) 글자 개정(종결 때 masterPlan).

## 테스트 결과

- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 172` · `# fail 0`.
- `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `ok. 5 passed` · `ok. 3 passed`.
- `cargo test --workspace --no-fail-fast` → `test result: FAILED` 0줄(`conv_cmd` 포함).
- `cargo clippy --workspace --all-targets -- -D warnings` → Finished. `check_ddd_docs` PASS · `check_readme` ok · `check_crates` violations=0.
- 실제 화면(디버그 빌드 `start --parent` + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z8\data`, 스키마 28 올림 · 사본 `coworkAgent.db.v27.bak`, ego-browser — 창 최소화라 스크린샷 대신 DOM 판독):
  - 받은 대화 `#c/1` 대화 정보 = 기간 "오늘 10:50 시작 · 마지막 오늘 10:50" · 답하는 도구 "Claude Code" · 읽는 폴더 · 컨텍스트 "압축 1회 · 어제 22:22" · 실행 명령 `& 'C:\Users\…\claude.exe' -p …` +
    안내. 복사 = 저장한 전체 명령 그대로 · 상태 줄 "실행 명령을 복사했습니다". 세부정보 절 제목 7개 모두 아이콘.
  - 보낸 대화 `#c/2` = 기간 · 컨텍스트 "압축 없음" 두 줄. 대화 보기 · 원문 보기 · 질의 · 요청 · 보내기 아이콘.
  - 단추 줄(1440 × 900 · 1440 × 2000, 상자 [위, 아래] · 단추 줄 [위, 아래] · 칸 아래): 초대 생성 [116, 462] · [405, 462] · 칸 1896 / 초대 수락 [116, 431] · [374, 430] / 연결 사용자 [116, 590] ·
    [533, 590] / 그룹 [116, 599] · [542, 598] / 에이전트 [116, 814] · [756, 813] — 화면 높이와 무관하게 단추 줄 = 내용 바로 아래 · 상자 바닥, 빈 상세 = 칸 전체([116, 1958]).
    1440 × 560 에이전트 = 상자 = 칸 [116, 536] · 본문 스크롤(570 > 292) · 단추 줄 [478, 535] 보임. 단추마다 아이콘.
  - 설정 화면 제목 8개 = 메뉴 아이콘(link · users · bot · play · monitor · power · bell · bug) · 색 rgb(36, 91, 171)(강조색), 타일 라벨 · 설정 단추 아이콘, 에이전트 상세 라벨 10개 아이콘.
  - 상단 · 레일: 새로 만들기 · 사용자 · 그룹 · 목록 · 미리보기 아이콘 + 글자("+" 없음).

## 다음 단계

gate 기계 판정(`tools/check_gates.py`) → eval(적대적 평가 라운드 1/2).
