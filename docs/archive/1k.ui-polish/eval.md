# eval.md — 1k.ui-polish 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 0319ff9acac7330a1b9b405ccef70e396299137a

- [x] E1: A6 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E2: A6 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E3: A2·A3·A6 — 화면 로직 node 테스트 98건 이상 · 실패 0(주고받은 글 에이전트 칩 · 다시 보내기 없음)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# pass (9[8-9]|[1-9][0-9]{2})\s+# fail 0/
  EVIDENCE: # skipped 0 / # todo 0 / # duration_ms 122.5515

- [x] E4: A1·A2·A3·A4·A5 — 웹UI 문구 · 자리(연필 · 상대 타일 · 칩 · 페어링 단추 둘 · 관리 · 에이전트 등록 · 두 열, 없어진 것 0 — 두 안내문 · "다시 보내기")
  CHECK: cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

- [x] E5: A6 — 문서 규약
  CHECK: python tools/check_ddd_docs.py
  EXPECT: PASS
  EVIDENCE: PASS — DDD 문서 규약 위반 없음

- [x] E6: A6 — README 사용자 관점
  CHECK: python tools/check_readme.py
  EXPECT: readme ok
  EVIDENCE: readme ok

- [x] E7: A6 — 의존 경계
  CHECK: python tools/check_crates.py
  EXPECT: /crates=4 edges=[0-9]+ violations=0/
  EVIDENCE: crates=4 edges=6 violations=0

- [x] E8: A1·A2·A3·A4·A5 — 이 PC 실측(CDP · 비파괴 — 실제 상대 · 이력 · 설정 값은 바꾸지 않음): 에이전트 연필 → 설정 그 에이전트 펼침 · 주고받은 글 타일 · 칩 ·
  EVIDENCE: 2026-10-15 headless Edge · playwright(scratchpad `j1/e8k.mjs`, 이 PC 실제 앱 — 릴리스 `task/1k-ui-polish` 빌드(`tools/rebuild_run.py`), 비파괴 — 화면 · 읽기만, 관리 · 수락 칸은 열었다 닫음) · 10/10 PASS — 에이전트 머리 안내문 없음 · 내 에이전트 4 줄 연필 4 · 연필 → `#settings/agents` · 초점 `agent{id}_manage` 펼침 · 주고받은 글 안내문 없음 · 줄 6 = 타일 6 = 칩 6 · 대화 말풍선 4 · 다시 보내기 0 · + → `#settings/conn` 페어링 카드 위 0px · 수락 칸 펼침 · 초점 `invite_text` · 두 열 [conn, peers] | [me, agents] · 페어링 단추 둘 같은 class · 관리 · 줄 타일 · 에이전트 등록 · 관리 열고 닫기 · 초대 수락 단추로 접기 · JS 오류 0


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

- 2026-10-15 · `/codex:adversarial-review --base 7420dba --scope branch` · 세션 codex-review · 시작 HEAD `01fe778`(요청과 일치) — 리뷰 도중 문서 전용 커밋
  `76d97e0`(가이드 · 매뉴얼 · 캡처)이 올라감(코드 차이 없음) · job `review-mv9mrasq-0k2zdy` — Verdict needs-attention(high 1 · medium 1).

| 지적 | 판단 | 조치 |
|---|---|---|
| [high] 필수 문서 gate가 깨진 커밋 (work/stage1.md:37) — `../eval.md` 링크 대상이 커밋에 없음(E5 dead link) | 사실(커밋 순서) | `eval.md` 를 커밋(`check_gates` 첫 측정이 원장을 만든 뒤) — `check_ddd_docs` · `check_gates` 다시 PASS |
| [medium] 여러 그룹의 같은 상대가 겹친 DOM id 를 만든다 (app.js:1064-1065) — `peer{id}` 열쇠 · `aria-controls` · 펼침 공유 | 사실 | 관리 줄 열쇠 · id = `peer{id}_{그룹 \| none}`(절마다, 펼침도 줄마다). 문구 시험 `polish_literals` · mock 확인(한 상대 두 그룹 — 겹친 id 0 · `aria-controls` 가 제 줄 · 하나 열면 하나만) |


## 적대적 평가 — 라운드 2/2

- 2026-10-15 · `/codex:adversarial-review --base 7420dba --scope branch` · 세션 codex-review(라운드 1 과 같은 세션) · HEAD `92260ab` · job `review-mv9n66o4-jwwjhf` —
  Verdict needs-attention(medium 2). 고침 커밋 `b3da25b`. **상한 도달 — 재리뷰하지 않는다.**

| 지적 | 판단 | 조치 |
|---|---|---|
| [medium] 연결 목록의 접속 상태가 시간 경과 후 갱신되지 않음 (app.js:1062-1063) — 서명이 같으면 다시 그리지 않아 "방금 통신" 이 남음 | 사실 | 부제를 `peerSub(p)` 로 묶고 `data-peer-sub` 를 달아, `renderPeerList` 가 서명 비교 **앞에서** 부제 글자만 고친다(5초 `refresh` → `loadPeers` → `renderPeerList`). 관리 폼(편집 중 모델 · effort)은 다시 만들지 않는다. 문구 시험 `polish_literals` · mock 확인(부제 "2시간 전 통신" 등 · JS 오류 0) |
| [medium] 남아 있는 검색 필터가 에이전트 편집 진입을 무력화함 (app.js:2742-2745) | 사실 | `editAgent` 가 먼저 `agent_q` 를 비운다. 문구 시험 · mock 확인(검색 "zzz-none" 으로 0줄 → 연필 → 검색 빈칸 · 초점 `agent1_manage` · 펼침) |

- 권고 중 "브라우저 회귀 시험 추가" 는 저장소에 브라우저 시험 틀이 없어(검증 명령 = cargo · node 단위) 문구 시험 + mock 확인으로 갈음했다. `presence` 는 `app.js` 안이라
  node 단위 시험 대상이 아니다.

## 미조치 지적

없음
