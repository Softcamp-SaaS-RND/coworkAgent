# plan — 1h.stabilization

> 근거: [intent](intent.md)(승인 2026-10-15 — 라운드 1 · 2). **구현자는 이 문서만 보고 일한다.** D·S 는 [masterPlan](../../masterPlan.md).
> 기준 커밋: `482277b`(`task/1h-stabilization` — 분기점 main). plan 승인: 2026-10-15 — 사용자 원문 "권장방안 승인. 개발 진행"(라운드 2 답과 같은
> 지시 — 아래 "intent 대비 해석" 은 보고 때 사용자에게 공개한다).

## 목표

19 인수 전 안정화 — 레일 타일 글자 · 그룹 모양 · 사이드바 메뉴 세 절 · "북마크" 이름 · 프로필 단추(화면 5건)와 트레이 다시 시도 · 사라진 세션 새로
시작 · 상대 번호 다시 쓰지 않음(결함 3건), 방화벽 프로필 안내(문서).

### 성공 기준(intent 사본)

- A1: 레일 타일 글자가 폭 예산 4(영문 · 숫자 1, 한글 · 한자 · 그림 문자 2) 안에서 앞에서부터 보인다 — "build" → "buil", "박지훈" → "박지", "A팀장" →
  "A팀" — 타일 밖으로 넘치지 않는다.
- A2: 그룹 타일은 겹친 카드 모양이라 상대 타일과 모양으로 구별된다(그림을 넣어도).
- A3: 사이드바 메뉴가 상단 · 중단 · 하단 세 절이고 절 사이에 구분선이 있다 — '나' = 개요 | 내 활동 · 요청 · 주고받은 글 | 북마크 항목 · 에이전트, 상대
  타일 = 요청 · 주고받은 글 | 북마크 항목 · 에이전트, 그룹 = 개요 | 요청 · 주고받은 글 | 북마크 항목. "대화" 메뉴 이름은 "개요" 다.
- A4: 화면에 "저장된 항목" 이 없고 "북마크 항목" · 메시지 작업 "북마크" / "북마크 해제" 로 보인다. 편집 모드 · 설정의 "저장" 은 그대로다.
- A5: 우상단 프로필 단추에 표시 이름이 잘리지 않고 보이며(약 12자까지, 넘으면 "…"), 마우스를 올리면 전체 이름이 보인다.
- A6: 트레이 아이콘 만들기가 실패하면 앱이 바로 끝나지 않고 간격을 늘려 가며 다시 시도한다 — 끝내 실패할 때만 지금처럼 사유를 보이고 끝난다.
- A7: 이어 쓰던 세션이 사라진 대화에 질문하면 새 세션으로 답이 오고, 그다음 질문도 실패하지 않는다.
- A8: 상대를 지우고 새 상대를 이어도 새 상대의 번호가 지운 상대 · 남은 대화 · 보낼 글의 번호와 겹치지 않는다 — 이미 쓰던 DB 를 올려도.
- A9: 사용자 가이드에 방화벽 프로필(Public 포함 여부) 확인 · 끄는 방법이 있다.
- A10: 루트 `PROJECT.md` 검증 명령이 통과한다.

## 코드 사실(실측, 기준 `482277b`)

- **타일 글자** `view.js:39 tileText` = 글자(grapheme) 앞 2개. 레일 타일 40px(`.rail-btn` · `--fs-md` 굵게) · 레일 칸 60px. `app.js tileFace` 를 레일 타일 ·
  '나' 타일 · 타일 편집 미리보기 · 프로필 단추(`#avatar_btn` 32 × 32, `app.js:493`)가 함께 쓴다.
- **그룹 타일** = 상대 타일과 같은 `.rail-btn` + 색(`colorOf(id)` → `peer-1` ~ `peer-5`).
- **사이드바 메뉴** `view.js:361 sideNav(tile)` 평평한 목록 — '나' 6 · 상대 4 · 그룹 4. 그리기 `app.js renderSideNav`. 첫 화면 제목 `#home_title`
  "대화"(`index.html:97` · `app.js:1195`).
- **저장** — 메시지 작업 `view.js:784 messageActions`("저장" / "저장 해제") · `app.js:1513` 상태 문구 · `#saved` 화면(`index.html:159`) · 머리 단추
  `#saved_btn`(`index.html:214`) · 실행 상세 절(`index.html:515`) · 문구 시험 `ux_literals.rs:87 · 97 · 226`. 같은 글자 "저장" 이 편집 모드 · 설정 ·
  타일 폼(`app.js:854 · 998 · 1646` · `index.html:77 · 196`)에도 있다 — 다른 기능.
- **트레이** `tray.rs:93` — `StartCause::Init` 에서 한 번 만들고 실패하면 `show_failure` → 끝(종료 코드 0 → 감시가 정상 종료로 보고 멈춤).
- **세션** — `agent/chat.rs:2985` 저장된 세션이면 `Session::Resume`. 실측(claude 2.1.299, 없는 세션): 종료 1 · stdout 결과 줄
  `{"type":"result","subtype":"error_during_execution","is_error":true,"num_turns":0,"errors":["No conversation found with session ID: …"]}` · stderr 같은
  문장. 원장 실측(2.1.296): JSON 없음 · 종료 1. 지금은 실패 답이 되고 세션을 그대로 둬 다음 요청도 실패한다.
- **상대 번호** — `peers.id INTEGER PRIMARY KEY`(AUTOINCREMENT 없음) → 가장 큰 번호를 지우면 다음 상대가 그 번호를 받는다. 상대 번호를 담는 곳 =
  `conversations.peer_id` · `outbox.peer_id`(+ 설정 `ui_folded` 열쇠 글자). `peers::upsert` 는 `ON CONFLICT(spki) DO UPDATE`(번호 유지). 스키마 16.
- 웹UI node 시험 76건.

## intent 대비 해석 (plan 승인 대상)

1. **폭 예산** — `tileText(name, budget = 4)`: 글자마다 넓이 1(그 밖) 또는 2(한글 · 한자 · 가나 · 그림 문자 — `\p{Script=Hangul|Han|Hiragana|Katakana}` ·
   `\p{Extended_Pictographic}`), 앞에서부터 넓이 합이 예산 안인 만큼. 넓이 3 이상이면 글자를 줄이는 표시(`tileFace` 가 `<span class="tile-text long">`) —
   40px 안에 4자가 들어가게.
2. **겹친 카드** — `.rail-btn.group` 뒤에 오른쪽 위로 4px 비킨 테두리 카드 한 장(`box-shadow` 두 겹 — 레일 바탕색 · 흐린 글자색). 그림 · 글자 · 선택 테두리
   (`aria-pressed`)는 그대로.
3. **세 절** — `sideNav` 가 절 사이에 `{ sep: true }` 를 넣은 목록을 돌려준다(순수 함수 · node 시험). `renderSideNav` 는 `<li class="nav-sep"
   role="separator">`. 절이 비면 구분선도 없다(상대 타일은 상단이 없다).
4. **"개요"** — 메뉴 이름 · 첫 화면 제목(`#home_title` '나' = "개요", 그룹은 지금처럼 "그룹 · {이름}"). 주소(`#`)는 그대로.
5. **북마크** — 화면 글자만: 메뉴 · `#saved` 제목 · 안내 · 빈 목록 문구 · 머리 단추 `title` / `aria-label` · 실행 상세 절 · 메시지 작업 · 상태 문구. 상태 문구 =
   "북마크 — 북마크 항목에 모았다(이 PC 에만)" · "북마크 해제". 편집 모드 · 설정 · 타일 폼의 "저장" 은 그대로.
6. **프로필 단추** — `[그림 또는 머리글자(예산 2)] + 이름`, 이름 칸 `max-width: 12em` · 말줄임, `title` = 전체 이름 + " — 프로필". 아이콘만 있던 32px
   정사각형을 자동 폭으로.
7. **트레이 다시 시도** — 실패하면 1 · 2 · 4 · 8 · 15 · 30초 뒤 다시(합 60초, `tray::retry_after(n)` 순수 함수 · 시험), 이벤트 루프를 막지 않는다
   (`ControlFlow::WaitUntil` · `StartCause::ResumeTimeReached`). 끝내 실패하면 지금처럼 사유 표시 · 정리 · **종료 코드 0** — 0 아닌 코드면 감시가 1초 뒤 다시
   띄워 60초 다시 시도를 되풀이하고 사유 파일을 매번 연다(60초 안 3번 규칙에 걸리지 않음). 1e report 의 "0 아닌 종료" 안은 다시 시도로 대신한다.
8. **세션 사라짐** — `runner` 가 사라진 세션을 오류 `SessionMissing` 으로 돌려준다: 결과 줄 `errors` 에 "No conversation found with session ID" 가 있거나,
   JSON 이 없고 stderr 에 그 문장이 있을 때. 실행(`agent/chat.rs`)은 `Session::Resume` 이었고 `SessionMissing` 이면 **새 세션으로 한 번** 다시 실행한다
   (로그 경고). 답 · 새 세션은 지금처럼 기록된다(다음 요청은 새 세션을 잇는다). 앞 대화 맥락은 사라진 세션과 함께 없다 — 대화 메모(지난 실행 뒤 사람 글)는
   지금처럼 붙는다. 판별 문장은 claude 출력(영문)에 기댄다 — 바뀌면 지금 동작(실패 답)으로 돌아간다(`ponytail:` 표시).
9. **스키마 17** — `peers` 를 `id INTEGER PRIMARY KEY AUTOINCREMENT` 로 다시 만든다(SQLite 는 제자리에서 못 바꾼다): 지금 표 정의(`sqlite_master.sql`)의
   `id INTEGER PRIMARY KEY` 를 바꾼 새 표 → 번호째 복사 → 옛 표 지움 → 이름 바꿈(한 트랜잭션, 올리기 전 사본 `.v16.bak`). 순번 시작 = `max(peers.id,
   conversations.peer_id, outbox.peer_id)` — 이미 지운 상대의 번호도 다시 쓰지 않는다. 지운 상대를 다시 이으면(같은 키) 새 번호 — 옛 대화는 '지운 상대'
   절에 남는다(1i A11 과 같음, 이력 보존).
10. **방화벽** — 사용자 가이드 17장 "알아 둘 점" 에 확인 명령(보안 검토 8장) · Public 끄기 한 줄. 끄는 명령의 효과는 이 PC 에서 확인하지 않는다(사용자 PC
    방화벽을 바꾸지 않음 — 확인 필요 표기).

## 설계

### `an-core`
- `db.rs` — 스키마 17(해석 9), `SCHEMA_VERSION = 17`. 시험 `tests/store.rs` `schema17_*`, `tests/paths.rs` 스키마 번호(16 → 17, "더 새 스키마" 18).

### `an-agent`
- `runner.rs` — `SessionMissing` 오류 · `session_missing(&io::Error)` · `run_claude` 판별(해석 8). `chat.rs` — `Resume` + `SessionMissing` 이면 새 세션으로 한 번.
- 시험 `tests/resume_missing.rs`(새) — 가짜 실행기(Resume → `SessionMissing`, New → 답): 답 완료 · 세션 바뀜 · 다음 요청 Resume(새 세션) · 판별(실측 결과
  줄 · stderr).

### `an-app`
- `tray.rs` — `retry_after` · 다시 시도(해석 7). 시험 = 모듈 안 `#[cfg(test)]`.
- 웹UI `view.js` — `tileText` 폭 예산 · `sideNav` 세 절 · `messageActions` 북마크. `app.js` — `tileFace` · `renderSideNav` 구분선 · 프로필 단추 · 북마크 문구 ·
  `#home_title`. `index.html` · `app.css` — 북마크 글자 · 구분선 · 겹친 카드 · 알약 단추 · 긴 타일 글자.
- 시험 — `view.test.mjs`(폭 예산 · 세 절 · 북마크) · `ux_literals.rs`(북마크 · 개요 · 구분선 · 알약 · "저장된 항목" 0).

### 문서
- README · user-guide(레일 · 메뉴 표 · 북마크 · 프로필 · 방화벽) · developer-guide(스키마 17) · an_core · an_app · an_agent 매뉴얼 · PRODUCT. masterPlan —
  D35 개정(메뉴 세 절 · 개요) · 통계 별도 Task 후보 · 개정 기록. 원장 #15 R2-F2r · #17 S1-P 닫힘.

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | 화면 5건 — 타일 글자 · 겹친 카드 · 메뉴 세 절 · 개요 · 북마크 · 프로필 단추 | node · 문구 시험 · mock 화면 | E1 ~ E4 |
| 2 | 결함 3건 — 트레이 다시 시도 · 사라진 세션 · 스키마 17 | core · agent · tray 시험 | E1 · E2 · E5 ~ E7 |
| 3 | 문서 · 방화벽 안내 · 이 PC 실측 · 캡처 | 문서 체커 · CDP 확인 | E1 ~ E9 |

## 평가 항목

- [ ] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A10 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A3·A4 — 화면 로직 node 테스트 82건 이상 · 실패 0(폭 예산 — "build" · "박지훈" · "A팀장" · 그림 문자 · 빈 이름, 세 절 — '나' · 상대 · 그룹 · 구분선
  자리, 메시지 작업 "북마크" / "북마크 해제")
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (8[2-9]|9[0-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: pending
- [ ] E4: A2·A3·A4·A5 — 웹UI 문구 · 자리("북마크 항목" · "북마크" · "북마크 해제" · "개요" · 구분선 `role="separator"` · 겹친 카드 · 알약 단추 · 화면 파일에
  "저장된 항목" 0)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A8 — 스키마 17(번호째 복사 · 사본 `.v16.bak` · 순번 = 지운 상대의 남은 대화 · 보낼 글 번호 이상 · 지우고 새로 이어도 번호 겹치지 않음 · 열 전부 그대로)
  CHECK: cargo test -p an-core --test store schema17
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A7 — 사라진 세션 → 새 세션으로 한 번 다시 실행 · 답 완료 · 다음 요청 새 세션 이음 · 판별(2.1.299 결과 줄 · 2.1.296 stderr) · 사라짐이 아닌 실패는 다시
  실행하지 않음
  CHECK: cargo test -p an-agent --test resume_missing
  EXPECT: /test result: ok\. [4-9] passed; 0 failed/
  EVIDENCE: pending
- [ ] E7: A6 — 트레이 다시 시도 간격(1 · 2 · 4 · 8 · 15 · 30초 · 그 뒤 없음)
  CHECK: cargo test -p coworkagent-app --lib tray::
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E8: A9 — 사용자 가이드에 방화벽 프로필 확인 명령 · Public 끄기 명령이 있다
  CHECK: python -c "s=open('docs/manual/user/user-guide.md',encoding='utf-8').read();print('check=%d off=%d' % (s.count('Get-NetFirewallApplicationFilter'), s.count('Set-NetFirewallRule')))"
  EXPECT: /check=[1-9] off=[1-9]/
  EVIDENCE: pending
- [ ] E9: A1 ~ A7 — 이 PC 실측(CDP · 비파괴, 실제 상대 · 이력은 건드리지 않음): 스키마 17 올리기 · 사본 `.v16.bak` · 상대 번호 그대로 · 타일 글자 · 겹친 카드 ·
  메뉴 세 절 · 개요 · 북마크 글자 · 프로필 알약 · JS 오류 없음. 트레이 다시 시도 · 사라진 세션의 실제 claude 실행은 19 인수 TC(로그인 직후 · 세션 파일
  지움)로 넘긴다.
  EVIDENCE: pending

## 종결 시 할 일

- masterPlan: D35 개정 · 통계 Task 후보 · 개정 기록. 원장 닫힘 2건. README · 매뉴얼 · 캡처(scratchpad `readme3/shot3.mjs` 다시).
- 이 PC 반영(릴리스 빌드 · DB 사본 `.v16.bak` 확인).
- GitHub 반영(push) · 릴리스는 사용자 확인 뒤.

## 개정 기록

- 없음
