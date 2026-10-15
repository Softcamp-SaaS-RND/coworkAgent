# eval.md — 1i.rail-group 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: aaea9bfcacd41f5edf38459137c524594b159bb7

- [x] E1: A12 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A12 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.23s

- [x] E3: A1·A2·A3·A4·A7·A9·A11 — 그룹 · 순서 API 시험(만들기 · 이름 1 ~ 40자 · 같은 이름 409 · 넣고 빼기 · 여러 그룹 소속 · 삭제 뒤 대화 남음 · 순서
  CHECK: cargo test -p coworkagent-app --test groups_api
  EXPECT: /test result: ok\. (1[2-9]|[2-9][0-9]) passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s

- [x] E4: A4·A9 — 스키마 16 올리기(첫 자리 = 지금 보이는 순서 · 사본 `.v15.bak`) · 다시 열어도 그룹 · 순서 남음
  CHECK: cargo test -p an-core --test store schema16
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.10s

- [x] E5: A2·A6·A8·A9·A10·A11 — 화면 로직 node 테스트 75건 이상 · 실패 0(`pos` 정렬 · 전체 순서에 끼우기 · 한 칸 옮기기 경계 · 놓을 수 있는 곳(종류 · 절) ·
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (7[5-9]|[89][0-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 132.6936

- [x] E6: A1·A3·A5 — 웹UI 문구 · 자리("그룹 +" · "대화 추가" · "그룹에서 빼기" · "그룹 삭제" · "타일 편집" · "이미지 지우기" · ⋯ · `aria-live`)
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

- [x] E7: A1 ~ A11 — 이 PC 실측(사람 · CDP 확인, 비파괴 — 실제 상대 · 실제 이력은 건드리지 않고 시험용 그룹만 만들고 지운다): 그룹 만들기 · 이름 거절
  EVIDENCE: 2026-10-15 headless Edge · playwright(scratchpad `e7.mjs`, 이 PC 실제 앱 — 릴리스 `task/1i-rail-group` 빌드(처음 `8ab4b36`, 라운드 2 뒤 `aaea9bf` 로 다시 27/27) · 스키마 15 → 16(`.v15.bak` · 실행 전 사본 `coworkAgent.db.pre-1i.bak`), 페어링된 상대 1 · 대화 2) 단계 a 22/22 + 앱 다시 실행 뒤 단계 b 5/5 = 27/27 PASS, JS 오류 0 — 스키마 16 · 올린 직후 자리 = 최근 활동순 · 레일 "그룹 +" · 구분선 · '나' 우클릭 메뉴 · 상대 타일 Shift+F10 메뉴(열기만) · 빈 이름 · 같은 이름 거절 문구 · 시험 그룹 둘 · 그룹 메뉴 4 · 빈 그룹 안내 · 대화 ⋯ 그룹 체크 · 그룹 타일에 끌어 놓아 넣기 · 그룹 요약 = 대화 있는 상대만 · "대화 추가" 적용 · "그룹에서 빼기"(대화 남음) · 그룹 이름 · 그림 바꿈 · 이미지 지우기 · Alt+↑ 그룹 타일 · 그룹 타일 끌기 · 대화 끌기(같은 절) · Alt+↓ 대화 + 알림 / 다시 실행 뒤 대화 자리 · 시험 그룹 남음 → 메뉴로 그룹 삭제 · 대화 순서 원래대로 · 대화 수 그대로. 찾아 고친 것 2(stage 3): "대화 추가" 긴 제목이 체크 칸을 0 폭으로 밀어냄 · 아이콘 그림 객체 URL 이 CSP 에 막힘. 상대가 하나라 상대 타일 끌기 · 여러 상대 절은 E5(node) · mock(`s3.mjs` 16/16)이 잰다


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-15 · `/codex:adversarial-review --base 136e829 --scope branch` · 세션 codex-review · 실행 HEAD `8ab4b36`(요청과 일치, 작업 트리 깨끗) ·
  Verdict needs-attention(high 2 · medium 3).

### 지적 원문

- [high] 그룹 삭제 경쟁 시 새 대화가 그룹 없이 전송된다 (crates/agent/src/chat.rs:1202-1205) — `POST /api/conversations`는 그룹을 별도 선검사하지만,
  실제 트랜잭션의 `groups::join` 반환값을 무시한다. 선검사 뒤 다른 창에서 그룹을 삭제하면 `join`은 `false`인데도 트랜잭션이 커밋되고 메시지가 상대에게
  전송된다. 이는 "없는 그룹이면 404이고 대화를 만들거나 보내지 않는다"는 계약을 깨는 비가역적 외부 동작이다. 자동 왕복 경로도 같은 패턴이다.
- [high] 그룹 소속 변경 실패가 성공 메시지로 덮인다 (crates/app/web/app.js:2578-2585) — `changeGroup`은 API 오류를 잡아 상태 줄에 기록한 뒤 오류를 다시
  던지거나 실패값을 반환하지 않는다. 따라서 호출자는 항상 성공으로 진행한다. 예를 들어 그룹 타일 drop, "그룹에서 빼기", 일괄 적용은 404·DB 오류 뒤에도
  각각 "넣었습니다", "뺐습니다", 변경 건수를 표시해 실패를 숨긴다.
- [medium] 겹친 순서 저장 요청이 최신 사용자 동작을 되돌릴 수 있다 (crates/app/web/app.js:2689-2705) — `moveItem`은 요청 세대나 직렬화 없이 낙관적
  렌더링 후 `PUT /api/order`와 재조회까지 수행한다. 빠른 연속 이동이나 여러 창에서 요청이 겹치면 서버 도착·커밋·재조회 응답 순서가 사용자 동작 순서와
  달라질 수 있다. 오래된 요청이 나중에 커밋하거나 오래된 재조회가 마지막에 적용되어 최신 이동이 사라지고 그 상태가 영속화될 수 있다.
- [medium] 브라우저 입력 제한이 그룹 이름 40자 계약보다 좁다 (crates/app/web/app.js:2544) — 서버와 `groupNameError`는 Unicode code point 기준 40자를
  허용하고 테스트도 이모지 40자를 허용하지만, 실제 입력은 `maxlength="40"`을 사용한다. HTML `maxlength`는 UTF-16 code unit 기준이므로 비-BMP 문자는 두
  단위로 계산되어 이모지 이름은 약 20자에서 막힌다. 타일 편집 경로도 동일하게 `maxLength = 40`을 설정한다.
- [medium] #float 종료 경로가 키보드 초점을 잃는다 (crates/app/web/app.js:2535-2539) — `closeFloat(false)`는 초점이 있던 자식들을 제거하면서 여는 요소로
  초점을 돌리지 않는다. 바깥 클릭, 그룹 생성 성공, 그룹에서 빼기, 그룹 삭제 같은 경로에서 초점이 `body`로 떨어질 수 있으며, anchor가 새로 렌더링되거나
  삭제되면 Escape 경로도 복귀 대상이 없다. 키보드 사용자는 작업 위치를 잃는다.

### 판단 · 조치

| 지적 | 판단 | 조치 |
|---|---|---|
| high 그룹 삭제 경쟁 | 사실 | `Chat::send_with` · `start_relay_with` 의 새 대화 트랜잭션에서 `groups::join` 이 거짓이면 `NotFound`("없는 그룹") — 트랜잭션을 버려 대화 · 글 · 보관 줄이 남지 않고 보내지 않는다. API 시험 `group_deleted_after_check_sends_nothing`(선검사 뒤 삭제 = API 를 건너뛴 `Compose{group}` 직접 호출, 일반 · 자동 왕복 둘 다, 상대 DB 빈 것까지) — 수정 전 RED(1 failed) 확인 |
| high 실패를 성공으로 | 사실 | `changeGroup` 이 성공 여부를 돌려주고, 끌어 놓기 · "그룹에서 빼기" · "대화 추가" 적용은 참일 때만 성공 문구. mock 회귀(서버에서 그룹을 지운 뒤 끌어 놓기 → "그룹 바꾸기 실패: …") |
| medium 순서 저장 겹침 | 사실(한 창 안 · 늦게 온 읽기) | 종류별 `SendLatest` 로 한 번에 하나 · 마지막 순서만 이어서. 저장 중이거나 읽는 사이 옮김 · 저장이 있었던 목록 응답에는 마지막으로 놓은 순서를 덮어 씌운다(`view.orderToApply` — node 시험 `order_to_apply`, 역순 응답 포함). 저장 실패면 서버 값으로 되돌린다. 여러 창의 마지막 저장 우선은 plan 해석 16(위험 수용) 그대로 |
| medium maxlength | 사실 | 그룹 이름 입력에서 HTML `maxlength` 를 없애고 `groupNameError`(코드 포인트 40자 — 서버와 같음)로만. 타일 편집도 그룹이면 제한 없음(표시 이름은 32 그대로). mock 회귀(그림 문자 41자 거절 · 40자 입력값 그대로 · 만들어짐) |
| medium 초점 | 사실 | `closeFloat(true)` 는 연 요소가 다시 그려져 없으면 '나' 타일로. 작업 뒤 초점 `focusAfterFloat` — 만들기 = 새 그룹 타일 · 빼기 = 목록의 대화 링크(없으면 "대화 추가") · 적용 = "대화 추가" · 삭제 = "그룹 +". 바깥 누름은 누른 곳에 초점이 가므로 돌리지 않는다. mock 회귀 5건 |

검증: `cargo test --workspace` EXIT 0 · clippy Finished · node 76/76 · mock 회귀 `r1.mjs` 10/10 + `s1` · `s2` · `s3` 다시 실행 실패 0.

## 적대적 평가 — 라운드 2/2

- 2026-10-15 · `/codex:adversarial-review --base 136e829 --scope branch` · 세션 codex-review(라운드 1 과 같은 세션) · 실행 HEAD `70b1ad6`(요청과 일치) ·
  Verdict needs-attention(medium 3). 라운드 1 지적 5건 중 4건 반영 확인, #float 초점은 바깥 누름 경로가 남음.

### 지적 원문

- [medium] 10,001번째 대화부터 모든 순서 변경이 영구 실패한다 (crates/core/src/groups.rs:186-187) — `reorder`는 `ids.len() > 10_000`을 거절한다. 웹UI는
  `orderOf("conv")`에서 전체 대화 ID를 항상 전송하고, 저장소에는 대화 삭제 경로가 없다. 따라서 누적 대화가 10,001개가 되는 순간 drag/drop과 Alt+↑/↓가 항상
  400으로 되돌아가 A8·A9 기능을 더는 사용할 수 없다. 현재 76개 node 테스트에도 10,000/10,001 경계가 없다.
- [medium] 라운드 1의 #float 바깥 클릭 초점 손실이 그대로 남아 있다 (crates/app/web/app.js:3177-3178) — 문서 바깥 클릭은 대상이 빈 배경처럼 초점을 받을 수
  없는 요소여도 무조건 `closeFloat(false)`를 호출한다. 이 경우 초점은 팝오버 안 버튼에 남아 있다가 자식 제거와 함께 `body`로 떨어진다. '누른 곳에 초점이
  간다'는 가정이 성립하지 않아 라운드 1 지적과 A5 키보드 경로가 여전히 깨진다.
- [medium] remove의 모르는 대화 ID가 400 대신 성공으로 처리된다 (crates/app/src/groups_api.rs:166-174) — 소속 변경 API는 `req.add`만
  `unknown_conversations`로 검증하고 `req.remove`는 검증하지 않는다. 따라서 존재하지 않는 대화 ID를 remove에 넣어도 DELETE no-op 후 200을 반환한다. 이는
  plan의 '모르는 대화 → 400' 계약을 축소하고 stale/잘못된 클라이언트 요청을 조용히 성공으로 숨긴다.

### 판단 · 조치(2 라운드째 반영 — 상한, 재리뷰 없음)

| 지적 | 판단 | 조치 |
|---|---|---|
| medium 10,000 상한 | 사실 | `reorder` · 소속 바꾸기의 개수 상한(`IDS_MAX`)을 없앴다 — 요청 크기는 서버 본문 상한(axum `Json` 기본 2 MiB, id 10,001개 ≈ 70 KB)이 막는다. core 시험 `reorder_more_than_ten_thousand`(10,001개 저장 · 준 순서) |
| medium 바깥 누름 초점 | 사실 | 바깥을 눌러 닫을 때 초점이 body(또는 상자 안)로 떨어졌고 누른 곳이 초점을 받지 못하는 바탕이면 연 요소로 돌린다. 입력칸 · 링크 등을 눌렀으면 그대로. mock 회귀 `r2.mjs` 3/3(바탕 → 연 ⋯ · 입력칸 → 입력칸) |
| medium remove 검증 | 사실 | 넣기 · 빼기 id 합집합을 `unknown_conversations` 로 — 없는 대화면 400. 있는 대화가 그룹에 없을 때 빼기만 그대로(200). API 시험 `membership_many_groups` 고침(빼기에 없는 대화 400 · 그룹에 없는 대화 빼기 200) |

검증(`aaea9bf`): `check_gates.py` 7/7 · `cargo test --workspace` EXIT 0 · clippy Finished · node 76/76 · mock `s1` 14 · `s2` 15 · `s3` 16 · `r1` 10 · `r2` 3
전부 PASS · E7 이 PC 27/27 다시.

## 미조치 지적

없음
