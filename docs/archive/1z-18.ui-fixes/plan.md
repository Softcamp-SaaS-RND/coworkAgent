# plan — 1z-18.ui-fixes

> 근거: [intent](intent.md)(승인 2026-10-28 — 결정 1 ~ 8, UI 시안 https://claude.ai/artifact/PP7ByU42ak48y2rAbmj5QL 2 회 개정). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-18-ui-fixes`(원 작업 폴더 — worktree 없음). 기준 커밋: `f4fdbe6`(main — v1.4.2 릴리스 뒤).
> 개정 2026-10-28 — 추가 요청 "북마크 목록에도 북마크 해제" (intent 결정 9 · A6): stage 1 의 7, E4 · E6 보강.
> 개정 2026-10-28(2) — 추가 요청 "트레이 새 요청 표시는 받은 요청 · 연결 요청만, 알림 설정과 무관"(intent 결정 10 · A7): stage 1 의 8, 바꾸는 시험, E1 · E4 · E6 · E7 보강.
> 개정 2026-10-28(3) — 추가 요청 "내가 요청한 대화의 상태 변경도 알림"(intent 결정 11): stage 1 의 8 `tray_marks` 에 `AskState`, E8 문구.
> 개정 2026-10-28(4) — 확인 답(intent 결정 12 · 13 · 14 · A8): `tray_marks` 에 `Approved`, 툴팁 "확인할 일 N건", 설정 "브라우저 알림" · 상태 문구, E4 · E8 보강.
> 승인: 2026-10-28 — 사용자 "위 사항 반영해서 개발 진행해"(시안 확정과 함께 — plan 승인 질문 없이 구현, #1z-16 · #1z-17 과 같은 구현자 해석, report 에 적음).

## 목표

요청 의견 칸을 카드 폭에 맞추고(단추 오른쪽 · 32px), archive 칸을 슬라이드 토글로, 즐겨찾기 줄에 머리 ★ · 세부정보 둘째 줄을, 에이전트 응답 줄에
에이전트 아이콘 · 제목 · 시각 · `[이름]`(항목 1100px, 좁으면 두 줄)을 준다. 버전 1.4.3.

### 성공 기준(intent 사본)

- A1: 요청 카드(목록 · 보드)와 대화 메시지 작업에서 의견 칸을 열면 칸이 카드 안쪽 폭 전체를 쓰고(단추 옆에 끼지 않음), 취소 · 확인 단추가 오른쪽 끝에 붙고 높이 32px.
- A2: 요청 화면 'archive 항목 보기' 가 슬라이드 토글(`role="switch"`)이고 켜고 끄면 지금처럼 archive 목록이 보이고 숨는다.
- A3: 즐겨찾기 화면 줄마다 머리(제목 앞)에 켜진 ★ 단추 — 누르면 즐겨찾기가 풀리고 목록에서 빠진다. 줄 아래 `{상대} · {에이전트} · {상태}`(상태 없으면 생략).
- A4: 에이전트 응답 화면 항목 너비 1100px · 한 줄 = 답한 내 에이전트 아이콘(그 에이전트 색 타일) · 제목 · 시각 · `[{에이전트 이름}]`(절 제목 = 요청한 사용자 그대로).
  항목이 1100px 보다 좁으면 둘째 줄에 시각 · `[{에이전트 이름}]`.
- A5: 그 밖 기존 동작 · 문구 그대로, 검증 명령 PASS, 매뉴얼 · README 반영, 버전 1.4.3.
- A6: 북마크 목록 줄마다 머리(본문 앞)에 켜진 북마크 단추 — 누르면 북마크가 풀리고 목록에서 빠진다(열린 대화 · 주고받은 메시지의 같은 글도 해제로).
- A7: 트레이 주황 점 · 툴팁 "확인할 일 N건" 은 받은 요청 · 연결(페어링) 요청 · 연결 승인 · 내 요청 상태 변경이 올 때만 켜지고 센다(그 밖 이벤트는 켜지 않음). 알림 받기 꺼짐 · 일시 중지 · 집중이어도 켜지고,
  알림을 꺼도 지워지지 않는다(웹UI 열기 · 트레이 열기로 지움). 브라우저 알림은 지금 그대로. Windows · macOS 같은 동작.
- A8: 설정 스위치 이름 "브라우저 알림", 상태 문구 "브라우저 알림을 켰습니다 · 껐습니다" · 권한 없음 "브라우저 알림을 켰지만 이 브라우저는 알림을 허용하지 않아 뜨지 않습니다.
  트레이 아이콘 표시는 그대로입니다."

## 설계

### 공통 규칙

새 함수는 둘레 규약대로(`export` — 파일 전용 static 함수 없음). 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저. 같은 개념 주석 · 매뉴얼은 함께 고친다.
API 는 건드리지 않는다. 서버는 `notify.rs` 트레이 건수만(8).

### stage 1 — 화면(web) · 버전

1. **의견 칸**(`app.css`): `.req-acts > .req-note { flex: 1 0 100%; }`(단추 줄 다음 줄 전체), `.req-note button` 높이 44 → 32px. 대화 메시지 작업(`.msg-actions` 안)은 실제 화면에서 보고
   같은 결함이면 같은 방식으로.
2. **archive 토글**(`index.html`): `<label class="switch-line"><input type="checkbox" id="req_archive" role="switch">archive 항목 보기</label>` — id · 동작 그대로.
3. **줄 함수**(`js/logic/conv.js`, `view.test.mjs` 시험):
   - `starDetail(c, peerName)` — `{상대 짧은 이름} · {에이전트(받은 대화 '내 …')} · {상태}`(빈 값 생략), 공유 대화는 `convLabel` 부제 + 상태.
   - `agentTag(c)` — `[{에이전트 이름}]`.
4. **`js/sidebar.js` `convItem(r, section, sideList, opts)`** — `opts.star` = 줄 머리(끌기 손잡이 다음, 링크 앞)에 `starButton`, `opts.detail` = 제목 줄 아래 `.sub.room-detail`,
   `opts.agent` = 유형 아이콘 대신 `agentTile(그 에이전트 색, "small")` · 시각 뒤 `.room-agent` `[이름]`(시각과 한 묶음 `.room-end`).
   `renderStarred` = `{ star: true, detail: starDetail(…) }`, `renderAgentConvs` = `{ agent: true }`(서명에 `state.agents` 색 더함). 다른 목록은 그대로.
5. **`app.css`**: `#agent_convs_list` 최대 너비 = 항목 1100px + 좌우 여백, 그 안 제목 줄 `flex-wrap: wrap` · 제목 기본 폭을 크게 줘 항목이 1100px 보다 좁으면
   `.room-end`(시각 · `[이름]`)가 둘째 줄로. `.room-agent` 최대 폭 · "…". `.room-detail` = 둘째 줄 작은 글자.
6. **버전** 1.4.2 → 1.4.3: `crates/{core,net,agent,app}/Cargo.toml` + `Cargo.lock`.
7. **북마크 목록**(`js/activity.js`): `savedRowEls(rows)` — `rowEls` 줄 앞에 `iconButton(BOOKMARK_SVG, "북마크 해제")`(`.bookmark.row-mark.lead.on`, `aria-pressed=true`, 줄 `has-mark`),
   `unsaveRow(r)` — `PATCH /api/messages/{id} {saved:false}` · 받은 글 사본 · 열린 대화 글 `saved` 도 false · `loadSaved()`. `app.css` `.row-mark.lead` 왼쪽 여백.
8. **트레이 표시**(`crates/app/src/notify.rs`, `tests/notifier.rs` 시험 먼저): `pub fn tray_marks(e: &PeerEvent) -> bool` — `PairRequest` · `Approved` · `Message { kind: Ask | AskState }` 만 참. `tray.rs` `tip` = "coworkAgent — 확인할 일 {n}건"(개정 4).
   `Notifier::on_event` — 변경 신호 · `Changed` 뒤, 켜짐 · 세부 설정 거르기 **앞**에서 `tray_marks` 면 `pending += 1` · 표시(한 잠금 그대로), 그 뒤 알림 거르기 · 전송은 지금 그대로.
   `set_enabled(false)` · `update`(`notify_inbound` 끔)의 건수 지우기 삭제(트레이는 설정과 무관). `clear`(웹UI `POST /api/seen` · 트레이 열기)만 지운다.
   웹UI 알림 도움말 `tip_notify` 문구: "탭이 열려 있으면 브라우저 알림으로 알립니다. 트레이 아이콘 주황 점은 이 설정과 무관하게 받은 요청 · 연결 요청 · 내 요청의 상태 변경이 오면 켜집니다."
   (`shell.js` 일시 중지 주석 · `notify.rs` 주석 같은 개념 함께). `tray.rs` 는 툴팁 글만(두 OS 공통 `tray-icon`). 설정 `index.html` 스위치 "브라우저 알림" · `prefs.js` 상태 문구(A8).

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `ux_literals.rs` — `<label for="req_archive">archive 항목 보기</label>`(#1y) literal 을 스위치 줄로, 새 PRESENT(의견 칸 한 줄 · 단추 32px · 즐겨찾기 ★ · 세부정보 · 에이전트 줄).
  (개정 2) `tip_notify` 옛 문구 "아니면 트레이 아이콘의 주황 점으로 알립니다" 가 literal 이면 새 문구로.
- `tests/notifier.rs` · `tests/events_api.rs`(개정 2) — 트레이 건수가 알림 설정을 따르던 기대(`disabled_sends_nothing` · `enabled_sends_notice_and_marks_tray` ·
  `clear_and_off_reset_tray` · `turning_off_during_events_leaves_nothing_pending` · `kinds_filtered` · `paused_drops_notices_and_tray` · `resume_after_pause_notifies_again` ·
  `focus_pauses_notices` · `seen_post_clears` · `events_stream_delivers_without_clearing` — 트레이를 켜던 승인 이벤트를 연결 요청으로)를 결정 10 기대로(알림 기대는 그대로, 트레이 기대만). 새 시험 `tray_marks_only_requests` · `tray_ignores_settings`.

### stage 2 — 실제 화면 · 문서

- 디버그 빌드 + scratchpad 시험 데이터 · ego-browser 두 테마 — 의견 칸(목록 · 보드 · 대화), archive 토글, 즐겨찾기 ★ 해제 · 둘째 줄, 에이전트 응답 한 줄(넓은 창) · 두 줄(좁은 창) · 항목 폭.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-18 줄) · README 1.4.3 줄.

## 평가 항목

- [ ] E1: A5 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A5 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A3·A4 — 웹UI 로직 시험(`starDetail` · `agentTag`)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A1·A2·A3·A4·A6·A8 — 화면 원본 literal(의견 칸 · archive 스위치 · 즐겨찾기 ★ · 세부정보 · 에이전트 줄 · 북마크 목록 해제 · 브라우저 알림 문구) · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A5 — 버전 1.4.3(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v143=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.3' for t in v)))"
  EXPECT: crates=4 v143=4
  EVIDENCE: pending
- [x] E6: A1·A2·A3·A4·A6 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 의견 칸 폭 · 단추 오른쪽 32px(목록 · 보드 · 대화), archive 토글 동작, 즐겨찾기 ★ 해제 · 둘째 줄, 에이전트 응답 항목 1100px 한 줄 · 좁은 창 두 줄, 북마크 목록 해제, 콘솔 오류 0
  EVIDENCE: Claude(구현자) 2026-10-28 — 디버그 빌드 1.4.3(`target\t17`) + scratchpad `ui1z18`(`ui1z17` 사본), ego-browser 라이트 · 다크(r01 ~ r07). 의견 칸: 요청 목록 폼 981 = 단추 줄 안쪽 981 · 보드 198 = 안쪽 198 · 대화 `.msg-body` 아래 238, 취소 · 확인 32px · 오른쪽 끝 = 줄 끝. archive `role="switch"` 36 × 20 켜면 목록 보임 · 끄면 숨김. 즐겨찾기 ★ 링크 앞 · 줄 가운데, 둘째 줄 "김도윤 · planner · 완료" 등, ★ 누름 → 그 줄 빠짐. 에이전트 응답 줄 = 타일 26px · 제목 · "10.22 22:22" · "[coder]", 항목 1100(창 1600 · 1700 · 1920) 한 줄 높이 22, 1088 · 1028 · 688 두 줄 높이 46. 북마크 목록 6 줄 머리 해제 단추(`aria-pressed=true`) 누름 → 5 줄 · 서버 `/api/saved` 에서 빠짐. 콘솔 · 페이지 오류 0. 근거 [work/stage2](work/stage2.md).
- [ ] E7: A5 — 문서 규약 · 크레이트 경계 · 코드 정리 · README · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d readme=%d'%(r('docs/manual/dev/web-ui.md').count('starDetail'),r('docs/manual/user/user-guide.md').count('archive 항목 보기'),r('PRODUCT.md').count('#1z-18'),r('README.md').count('1.4.3')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* readme=[1-9][0-9]*/
  EVIDENCE: pending
- [ ] E8: A7 — 트레이 확인할 일 건수 = 받은 요청 · 연결 요청 · 연결 승인 · 내 요청 상태 변경만 · 알림 설정(꺼짐 · 일시 중지 · 집중)과 무관 · 끄기로 안 지워짐, 브라우저 알림은 그대로
  CHECK: cargo test -p coworkagent-app --test notifier --test events_api
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
