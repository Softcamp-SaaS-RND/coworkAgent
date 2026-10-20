# stage 3 — 웹UI · 문서

## 한 일

- `index.html`: 주고받은 메시지 머리 `<label class="head-pick">기간 <select id="ui_peer_weeks" data-num>`, 요청 거르기 줄 `archive 항목 보기` 체크(`req_archive`),
  `#req_archive_list`(목록 · 보드 아래 따로 된 절).
- `view.js`: `reqArchiveAction`(완료 = "archive" · archive 됨 = "archive 해제" · 그 밖 `null`, 받은 · 보낸 요청 모두) → `requestRows` 의 `archive`, `PEER_WEEKS` · `peerWeeksOptions`,
  `reqHead` 끝에 `archive {n}`(전체 수 밖).
- `app.js`: `PREFS` 에 `ui_peer_weeks`(`data-num` 이면 숫자로 PUT — 바꾸면 `peerPages` 비우고 다시), `loadRequests` — 체크면 쪽마다 `archived=true` 목록을 받아 `renderReqArchive`
  (절 "archive — 받은 요청" · "archive — 보낸 요청", 처음 펼침), 들어올 때마다 체크 끔(해석 6), 요청 줄 글자 단추 `archiveReq`.
- `app.css`: `.head-pick`(머리 오른쪽) · `.filters input[type="checkbox"]`(늘어나지 않게) — 토큰만.
- node 시험 3(`peer_weeks_options` · `req_head_archive` · `req_archive_action`) → 150, `ux_literals` 있음 표 5줄.
- 문서: 사용자 가이드 · README · 개발자 가이드 · web-ui · an_app · an_core(문서 subagent — 아래 확인).

## 실화면(E11 — 이 PC, CDP)

실제 앱 · 데이터와 떼어 놓은 디버그 인스턴스(`COWORKAGENT_DATA_DIR` = 스크래치 폴더, `coworkAgent start --parent <대기 프로세스>`)에 요청 3(방금 완료 · 9일 전 완료 · 대기)을 넣고
Edge headless + CDP(`scratchpad/ui_probe_1y.mjs`):

```text
weeks { visible: true, n: 12, value: '4', first: '1주', last: '12주' }
weeks saved 8                                   ← 8 고르면 PUT /api/settings → GET /api/messages/peers 다시
list [{"text":"기다리는 요청","btns":["수용","반려","완료"]},{"text":"방금 끝난 요청","btns":["대기","수용","반려","archive"]}]
head 2 | 대기 1 | 완료 1 | archive 1 | 0          ← 9일 전 완료는 목록 · 전체 수 밖, 머리 끝 archive 1
archive box { checked: false, hidden: true }     ← 기본 숨김
archived {"hidden":false,"heads":["archive — 받은 요청",…,"archive — 보낸 요청"],"rows":[{"text":"오래전 끝난 요청","btns":[…,"archive 해제"]}]}
click archive { status: 'archive 로 옮김', archived: [ '방금 끝난 요청', '오래전 끝난 요청' ] }
reopen archive box { checked: false, hidden: true }   ← 다시 들어오면 숨김
```

스크린숏 `scratchpad/1y_archive.png` · `1y_peer.png`. 끝낸 뒤 인스턴스 `stop`(종료함).

## 릴리스 빌드 확인

실행 중인 앱이 `target/release/coworkAgent.exe` 를 잡고 있어 따로 된 대상 폴더(스크래치 `reltarget`)로 `cargo build --release` — `pe_imports`: `imports=16 vcruntime=0
dependent_load_flag=0x800`(새로 `crypt32.dll` — OS 인증서 저장소, System32), 크기 12,154,368(전 11,102,720), 배포처 문자열 포함.

## 테스트 결과

```text
node --test "crates/app/web/*.test.mjs" → # pass 150 · # fail 0
cargo test -p coworkagent-app --test ux_literals → ok. 5 passed
cargo test --workspace → 80 바이너리 487 passed · 0 failed
cargo clippy --workspace --all-targets -- -D warnings → 경고 0
```
