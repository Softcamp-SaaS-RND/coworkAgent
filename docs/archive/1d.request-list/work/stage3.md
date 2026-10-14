# stage3 — 요청 화면 · 대화 태그 · 입력창 · 내 활동 정리 · 매뉴얼 · 실화면

> 근거: [plan](../plan.md) Stage 3. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

왼쪽 메뉴 '요청' 화면(받은 · 보낸 요청, 상태 거르기, 받은 요청 상태 바꾸기 + 메모)과 대화의 요청 태그 · 상태 단추, 입력창 요청 안내(A3 · A4 · A5),
'내 활동' 의 질의 · 요청 거르기 삭제(A5). 매뉴얼 · README. 실화면 확인(E8). 대상 gate: E1 ~ E3 · E8.

## 산출 파일

- `crates/app/web/view.js` — `REQ_STATES` · `askTag`("요청 · {상태}") · `reqMoves`(대기 → 수용 · 반려, 수용 → 완료) · `requestRows` ·
  `messageActions`(`req:{상태}` — `done` 삭제, 내 요청도 다시 보내기) · `sideNav(tile, unseen, waiting)`('요청' 배지) · `route('#requests')` ·
  `messagesSigOf`(요청 상태 · 메모).
- `crates/app/web/app.js` — `#requests` 화면(`loadRequests` · `requestEls` · 탭 · 상태 거르기, 고른 타일 범위), `noteForm`(메모 한 줄 칸 + 확인 · 취소 —
  대화 작업 막대 · 요청 행 공용), `loadCounts` 가 받은 요청 대기 수(`/api/messages?side=theirs&ask=request&req=wait`)도, 대화 메타에 메모 줄, 실행 상세
  '요청' 절 상태 · 메모, 입력창: 요청이면 멘션 없이 보내고 안내 `#ask_hint`(멘션 힌트 대신), 새 대화: 요청 + 자동 왕복은 보내지 않음.
- `crates/app/web/index.html` — `#requests` 절 · `#ask_hint` · 내 활동 종류에서 질의 · 요청 삭제. `app.css` — `.req-note`.
- `crates/agent/src/chat.rs` — `start_relay_with` 가 요청이면 대화를 만들기 전에 `InvalidInput`(`REQUEST_NO_RELAY` — 화면의 새 대화 거름과 같은 규칙,
  `requests.rs` 위조 · 규칙 시험에 한 줄).
- 테스트 — `view.test.mjs` +5(`req_moves` · `request_rows` · `requests_route` · `models_tools` · `messages_sig_req`) · `ask_marks` · `side_nav` 새 규칙,
  `ux_literals.rs` `request_list_labels`(+1 — 새 문구 · 없어진 문구), `focus_mention_labels` 에서 완료 표시 문구 삭제.
- 문서 — `README.md`(주요 기능 '요청 목록' 행 · 집중 문구) · `user-guide.md`(4 화면 · 6 도구 · 7.3 질의 · 요청 · 7.5 작업 막대 · 9 요청 목록 · 13 모델
  드롭다운) · `developer-guide.md`(6.4 · 표 · 8.6 요청 목록 시퀀스) · `an_core` · `an_net` · `an_agent` · `an_app`.

## 설계 결정

- **메모 칸은 그 자리에 연다** — 대화 작업 막대 · 요청 행 끝에 한 줄 폼(브라우저 `prompt` 같은 대화 상자 없음). 확인하면 새로 고침이 대화 · 목록 ·
  배지를 함께 다시 그린다.
- **배지 = 받은 요청 중 대기** — 고른 타일 범위, 기존 새로 고침(`loadCounts`)에 한 번 더 묻는다(최대 500 — 넘으면 500 으로 보인다).
- **요청 화면 탭 바꾸기는 거르기를 유지**, 화면을 새로 열 때만 거르기를 "전체" 로.
- 실화면 확인은 임시 두 PC 하네스(`tests/zz_e8.rs` — 커밋하지 않음, 가상 인물 `user-a` · `user-b`)로 — 실행 중인 사용자 앱 · 실제 DB 는 건드리지 않았다.

## 테스트 결과

```text
$ cargo test --workspace            → 315 passed, 0 failed (합)
$ cargo clippy --workspace --all-targets -- -D warnings → Finished(경고 0)
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 47 # fail 0
$ python tools/check_readme.py      → readme ok
$ python tools/check_crates.py      → crates=4 edges=6 violations=0
```

E8 실화면(헤드리스 Edge — 두 PC 웹 47991 · 47992, `e8_check.mjs`):

```json
{
 "modelTag": "SELECT", "modelOptions": ["opus", "sonnet", "haiku"],
 "agentToolCheckboxes": 0,
 "agentToolsNote": "탐색 도구(읽기 전용) 항상 사용 — Read · Grep · Glob · LS · NotebookRead",
 "peerModelOptions": ["", "opus", "sonnet", "haiku"],
 "askHintShown": true, "aTag": "요청 · 대기",
 "bBadge": "1", "bRowState": "대기", "bRowAfter": "수용", "bMoves": ["완료"],
 "aSentRow": "user-b\n수용\n19:59\nguide 에게\n결제 로그 형식을 JSON 으로 바꿔 주세요\n메모: 이번 주 안에 반영",
 "aTagAfter": "요청 · 수용",
 "errors": []
}
```

## 다음 단계

적대적 평가(`eval.md`) — gate 기계 판정 → codex 리뷰(라운드 최대 2).
