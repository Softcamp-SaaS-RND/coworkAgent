# stage 1 — 사람 ↔ 에이전트 구분 · 입력창 · 의견 · 레일 끌기(한 stage)

## 목표

plan "설계" 전부 — an-core 의견 여러 줄 · 보내는 꼴(U+2028), an-agent 보내기 · 받기, 웹UI 말풍선 A · 에이전트 아바타 · 목록 카드 · 발화자 아이콘 · 입력창 여백 ·
의견 칸 · 레일 끌기, 매뉴얼 · README.

## 산출 파일

- `crates/core/src/chat.rs` — `clean_req_note`(여러 줄 · CRLF → LF · 오류 "의견은 …") · `note_to_wire` · `note_from_wire`.
- `crates/agent/src/chat.rs` — `set_req_state` 가 `note_to_wire` 로 보냄, `inbound_req_state` 가 `note_from_wire` 뒤 검사.
- `crates/app/web/view.js` — `speakerIcon` · `RAIL_W` · `railSize` · `NOTE_MAX` · `noteCount`, `inboxRows` · `activityRows` 행 `author`.
- `crates/app/web/app.js` — `messageEl`(에이전트 = 로봇 아바타 양쪽 · "에이전트" 칩) · `feedbackEl`("요청 의견 · ") · `speakerEl` · `convItem` · `rowEls` · `activityEls` ·
  `noteForm`(textarea · 글자 수 · Enter / Shift+Enter) · `fitComposer` · `saveTileSize` · `#rail_resize` 끌기 · 키, 남은 "피드백" 주석도 "의견".
- `crates/app/web/index.html` — `#rail_resize`, `reply_text rows="1"`, 타일 크기 "크게".
- `crates/app/web/app.css` — 토큰 `--rail-w-normal` · `--chat-gutter`, `#app --rail-w`, 말풍선 A · 목록 카드 · `.speaker` · 의견 칸 · 손잡이 · 좁은 화면 여백.
- 시험 — `core/tests/requests.rs`(`note_lines_200` · `note_wire_old_peer`), `agent/tests/requests.rs`(`note_lines_reach_requester`, 거부 목록 줄바꿈 → 탭),
  `view.test.mjs`(새 4 + `composer_a` · `screen_settings_1x` 를 새 명세로), `ux_literals.rs`(PRESENT 9 · ABSENT 6, 레일 72px 문구 → 토큰).
- 문서 — 사용자 가이드 · 웹UI · an_core 매뉴얼 · README.

## 설계 결정

- 서버 문구는 `clean_label` 호출이 아니라 `clean_req_note` 자기 검사(줄바꿈만 허용해야 해서) — E9 CHECK 보정(plan 개정 기록).
- CSS 리터럴은 첫 `:root` 에만(`css_tokens` 시험) — `max(48px, 8%)` · 72px 를 토큰으로, 카드 간격 6px → `--sp-2`(8px). E7 CHECK 보정.
- `ux_literals` 는 시험 5개 — E10 CHECK 를 세 시험으로 거름.
- 끌기 중 폭은 `#app` 인라인 `--rail-w`(클래스의 `--rail-w` 보다 앞섬), 놓으면 지우고 클래스(크게 72 · 작게 168)로. 손잡이는 `#app` 기준 절대 위치(격자 칸을 늘리지 않음).
- 좁은 화면(720px 이하): 손잡이 숨김, 말풍선 영역 · 입력창 좌우는 `--sp-4`(48px 이상이면 화면이 너무 좁아진다 — 명세 밖 보정).
- 에이전트 대화 목록 · 즐겨찾기도 `convItem` 이라 같은 카드 · 아이콘. 북마크 줄도 `inboxRows` 라 아이콘.
- rustfmt: 워크스페이스에 이전부터 맞지 않는 곳이 있어 `cargo fmt --all` 은 하지 않음 — 이번에 쓴 시험 블록만 맞춤.

## 테스트 결과

```text
cargo test -p an-core --test requests note_                      → test result: ok. 2 passed; 0 failed
cargo test -p an-agent --test requests -- receiver_moves_state_with_note note_lines_reach_requester → test result: ok. 2 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs"      → # pass 155 / # fail 0 (전 151)
cargo test -p coworkagent-app --test ux_literals                 → test result: ok. 5 passed; 0 failed
cargo test --workspace                                           → 78 바이너리 481 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings            → Finished(경고 0)
```

RED 확인: 구현 전 `receiver_moves_state_with_note` 가 "둘째\n줄" 거부를 기대해 실패(줄바꿈 허용 뒤 — 거부 목록을 탭으로 바꿈), `css_tokens` `literals_only_in_root` 가 리터럴 4곳으로 실패 → 토큰화.

## 다음 단계

- gate 원장(`check_gates.py`) · 적대적 리뷰 라운드 1/2(codex-peer-review).
- E11 실 화면(에이전트 아바타 · 스크롤 간격 · 여백) — 앱을 이 PC 에 다시 띄워야 한다(`tools/rebuild_run.py` — 사용자 확인 필요).
