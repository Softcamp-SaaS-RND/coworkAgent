# stage3 — 관람자 초대: API · 화면 · CLI 발화자

> 근거: [plan](../plan.md) 해석 17 · 18 · 21 · Stage 3.

## 목표

stage 2 의 공유 기능을 로컬 API 로 열고, 화면(공유 대화 묶음 · 읽기 전용 · 관람자 목록)과 CLI `history` 발화자를 맞춘다. 대상 gate: E13 · E14.

## 산출 파일

- `crates/app/src/chat_api.rs` — `POST /api/conversations/{id}/observers {peer_id}`(204 · 사본 · 원 상대 · 5명 400 · 이미 관람자 · 상대 앱이 모름 409) ·
  `DELETE /api/conversations/{id}/observers/{peer}`(204 · 아니면 404) · `POST /api/conversations/{id}/leave`(204 · 사본 아니면 400). `fail` 이 `Unsupported` → 409.
  대화 목록 JSON — 사본 `share{other, owner_dir, ended}`(끝나면 `ended: true`), 그 밖 `observers`(관람자 id) · `watchers`(원 상대가 받은 수), 글 JSON `owner`.
- `crates/app/src/cli.rs` — `author_label` 공유 대화 갈래(웹UI `sharedAuthor` 와 같은 규칙) · 단위 시험 `share_author_label`. 파일 전체 rustfmt 는 하지 않았다(1n 이
  main 에서 같은 파일을 다시 줄바꿈해 합칠 때 겹치지 않게 — 손으로 rustfmt 꼴).
- 화면(커밋 `aa39946`) — `view.js` `CONV_GROUPS` 셋째 "공유 대화" · `sharedAuthor` · `shareHead` · `shareCandidates` · `MAX_OBSERVERS` · `convMeta.tag`
  "공유 해제됨" · `relayText` 사본 null, `app.js` 사본은 입력창 · 편집 · 컨텍스트 단추 숨김 · 편집 모드 막음 · 세부정보 "관람자" 절(목록 · 빼기 · 추가 · 나가기 확인),
  `index.html` `#detail_share_sec`, `app.css` `.share-tools` · `.share-row`.
- 시험 — `chat_api` `share_observer_api`(세 앱: 거절 · 추가 · 409 · 사본 이력 · owner · 다시 공유 / 글쓰기 400 · 원 상대 watchers · 나가기 → 관람자 빔 ·
  ended · 404 · 400 · 다시 넣기 · 빼기), node `share_groups` · `share_labels` · `share_read_only` · `share_head_candidates`.

## 설계 결정 · plan 편차

- 공유 대화 말풍선 — 처음엔 모두 왼쪽, 사용자 지시로 공유한 사람 쪽 글(`owner`) 오른쪽(`bubbleSide`, plan 개정 기록).
- `ux_literals` 두 시험의 옛 단정을 의도된 변경으로 고침 — `polish_literals`(받은 대화만 '내 ' — 공유 대화는 이름 그대로), `peer_group_settings_literals`(연결 목록
  "읽기 전용" 표시 없음은 app.js 만 — view.js `shareHead` "읽기 전용" 은 별개).

## 테스트 결과

```text
cargo test -p coworkagent-app --test chat_api share   → test result: ok. 1 passed; 0 failed; 7 filtered out
node … --test-name-pattern="^share_" view.test.mjs    → # pass 4 / # fail 0
node --test "crates/app/web/*.test.mjs"               → # pass 112 / # fail 0
cargo test --workspace                                → test result: ok 73건 · 실패 0
cargo clippy --workspace --all-targets -- -D warnings → Finished
```

## 다음 단계

- stage 4 — 사용자 가이드 · 개발자 가이드 · 매뉴얼(an_net · an_core · an_agent · an_app) · README · 캡처 · mock 확인.
