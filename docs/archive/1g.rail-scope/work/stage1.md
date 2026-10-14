# stage1 — `view.js` 로직 · 설정 `ui_folded`

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

메뉴 · 경로 · 상대별 묶음 · '대화' 요약의 판단 로직(A1 ~ A4 · A7)과 접힘 설정(A3). 대상 gate: E1 · E3 · E6.

## 산출 파일

- `crates/app/web/view.js` — `sideNav`('나' 6 · 상대 4) · `route`(`#p` = `{view:"peer", id:null}`) · `peerGroups` · `peerSummary` · `waitingByPeer` ·
  `convSections(rows, paired, peerNames)` · `inboxRows` · `requestRows` · `activityRows` 행에 `peer`.
- `crates/core/src/run_opts.rs` — `FOLDED_KEY` · `MAX_FOLDED`(300) · `MAX_FOLD_CHARS`(32) · `clean_folded` · `folded`.
- `crates/app/src/notify.rs` — `SettingsPatch.ui_folded` · `Notifier::folded` · 설정 JSON `ui_folded`(검증은 다른 항목과 함께 쓰기 전에 — 하나라도 틀리면
  아무것도 쓰지 않는다).
- 시험 — `view.test.mjs`(side_nav 고침 · peer_groups · row_peer · waiting_by_peer · peer_summary · conv_sections_by_peer 새로) · `ux_api.rs`
  `settings_folded`.

## 설계 결정

- **묶음 함수 하나**(`peerGroups`) — 내 활동 · 요청 · 저장 · 주고받은 글 · 사이드바 대화가 같이 쓴다. 요약(`peerSummary`)만 대화가 없는 페어링된 상대도
  보인다(대화 0 · "새 대화").
- **접힘 = 서버 설정** — 웹UI 포트가 실행마다 바뀌어 `localStorage` 는 이어지지 않는다(#15 `ui_side_width` 와 같다). 열쇠 형식을 서버가 막아 임의 문자열이
  설정 표에 쌓이지 않게 했다.
- 새 함수는 모듈 export · `run_opts` pub 함수(파일 전용 자유 함수 없음).

## 테스트 결과

```text
$ node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 55 · # fail 0
$ cargo test -p coworkagent-app --test ux_api folded → test result: ok. 1 passed; 0 failed
```

RED: 새 export 가 없어 `view.test.mjs` 가 불러오기에서 실패(# pass 0 · # fail 1). 시험 고칠 때 옛 시험 끝을 잘못 잘라 `});` 한 줄이 남아 구문 오류 —
지움. `settings_folded` 는 구현과 함께 썼다(구현 전이면 `deny_unknown_fields` 로 첫 PUT 이 400).
