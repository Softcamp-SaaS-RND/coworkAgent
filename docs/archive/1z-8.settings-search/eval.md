# eval.md — 1z-8.settings-search 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 5aa71248e6233fd51fc6275003b90f9ed8c081ec

- [x] E1: A10 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E2: A10 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

- [x] E3: A3·A4·A5·A9 — 웹UI 로직 시험(검색 표지 · 미리보기 크기 · 설정 메뉴 · 해시 별칭 · 진단 글)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A2 — 검색 스위치 서버(기본 = 닫힌 대화 뺌 · closed=true = 포함 · 끝난 공유 사본도 닫힘)
  CHECK: cargo test -p coworkagent-app --test ux_api search_closed
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 48 filtered out; finished in 0.48s

- [x] E5: A9 — 디버깅 API(토큰 필요 · 정보 칸 · 로그 끝 200줄 · 64 KiB 상한 · 지문/토큰/키 칸 없음)
  CHECK: cargo test -p coworkagent-app --test debug_api
  EXPECT: /test result: ok\. [3-9] passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

- [x] E6: A1·A5·A6·A7·A8·A11 — 화면 원본 literal(버전 칸 · 검색 머리 · 메뉴 8 · 페이지 · 스위치 · 2열 타일 · 목록 · 상세 · 닫기 · 제목 줄 없음 · 설정 끌기 없음)
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

- [x] E7: A10 — 문서 규약 · 크레이트 경계 · 버전 · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('ver=%d webui=%d app=%d core=%d guide=%d'%(sum(r('crates/%s/Cargo.toml'%c).count('version = \x221.3.4\x22') for c in ('agent','app','core','net')),r('docs/manual/dev/web-ui.md').count('(#1z-8'),r('docs/manual/dev/an_app.md').count('/api/debug'),r('docs/manual/dev/an_core.md').count('closed'),r('docs/manual/user/user-guide.md').count('**디버깅**')))"
  EXPECT: /ver=4 webui=[1-9][0-9]* app=[1-9][0-9]* core=[1-9][0-9]* guide=[1-9][0-9]*/
  EVIDENCE: ver=4 webui=6 app=2 core=4 guide=2

- [x] E8: A1 ~ A9 · A11 — 실제 화면: 디버그 빌드(시험 데이터 폴더 — 열린 · 닫힌 대화 시드)에서 상단 바 버전, 검색 스위치 끔/켬 결과 · 칩 색, 미리보기 125 % 2줄 · 비침 없음 · 여백,
  EVIDENCE: Claude(구현자) 2026-10-23 — 디버그 빌드 + `COWORKAGENT_DATA_DIR`=scratchpad `ui1z8\data`(사용자 앱 그대로, 상대 3 · 대화 5 — 나감 1 · 상대가 나감 1 · 그룹 2 시드), ego-browser DOM 판독 · 스크린샷 — 상단 바 v1.3.4, 검색 끔 2 · 2 / 켬 6 · 4(태그 · 유형 칩), 미리보기 125 % 2줄 · 12px · 비침 없음, 세부정보 제목 줄 없음, 설정 첫 화면 연결(aria-current) · 한 화면만, 레일 + · 새로 만들기(초대 생성 · 에이전트 등록) · #settings/peers 진입, 고르기 · ✕ · 닫기 초점, 초안 유지, 그룹 넣기 · 빼기, Space 스위치 저장, 2열 · 900px 1열 · 위아래, 다크, 디버깅 카드 · 로그 — 결함 3건(그룹 상세 아래 줄 위치 · 검색 스위치 폭 · 크기 줄바꿈) 고친 뒤 재확인(관측 전문 = plan.md E8 EVIDENCE).


WEAK: E8 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_476d64ec-8a2f-4350-8844-0a5a9b2b712b`.
Run `run_a14062c3e398` · Task `task_f00dcbf9524f` · Dispatch `ctx_3c9118f095d4`, worker_done(`msg_9d746a9e2a4b`) 발신 handle 일치 · outcome succeeded. 범위 `866be6e..1ced96d`(self-collect),
메타 `요청 1z-8-settings-search-r1 · revision 1ced96d7700dbb4a6aebec7bf3dddbf14d1018e1 · 라운드 1/2`. 리뷰 동안 대상 작업 폴더 HEAD 1ced96d 그대로(그 사이 손댄 것 = masterPlan · PRODUCT 환류 — 범위 밖 미커밋 문서, 코드 아님).
리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal` · 원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

고정 범위와 intent·plan을 읽기 전용 검토했다. 삭제한 에이전트의 초안이 새 프로필에 적용되어 출하 보류가 필요하다. 초안 식별 경계 수정과 회귀 검증이 남았다.

Findings:
- [high] 삭제한 에이전트의 초안을 재사용된 id의 새 프로필에 적용함 (crates/app/web/app.js:1442-1446)
  agentDrafts는 숫자 id로만 저장되며 삭제 성공·목록 갱신 때 제거되지 않는다. agents 테이블은 INTEGER PRIMARY KEY이고 add_with는 id 없이 INSERT하므로, 가장 큰 id의 에이전트를 삭제하고 등록하면 같은 id를 재사용한다. A의 폴더·추가 지시·상위 탐색 초안을 남긴 채 A를 삭제하고 B를 등록하면 agentDetail(B)가 A의 초안을 표시한다. 이후 저장하면 B의 등록 폴더 대신 A의 폴더와 지시가 저장되고 parent_read도 다시 켜질 수 있어 원격 요청에 허용한 읽기 범위가 달라진다. SQLite :memory:의 id 재사용과 고정 revision에서 추출한 agentDetail 초기화 코드 실행으로 이전 폴더·prompt·parent_read 적용을 확인했다. 실제 브라우저·두 PC 연동은 미실행이며 저장 이후 영향은 agentPatch와 PATCH 호출 흐름에 근거한 추론이다.
  Recommendation: 삭제 성공 시 해당 초안을 제거하고, 목록 갱신 시 사라지거나 프로필 식별자가 달라진 항목의 초안도 폐기한다. 초안에 이름 또는 별도 등록 식별자를 함께 보관해 동일 숫자 id의 다른 프로필에는 복원하지 말고, 최대 id 삭제→새 등록→저장 회귀에서 폴더·prompt·parent_read·shell이 이전 초안에서 유입되지 않는지 검증한다.

Next steps:
- agentDrafts의 삭제·프로필 교체 경계를 수정하고 id 재사용 회귀를 검증한다.
- 수정 후 고정 revision으로 라운드 2/2 리뷰를 요청한다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R1-1 high 삭제한 에이전트의 초안을 재사용된 id 의 새 프로필에 적용 | 사실 — `agentDrafts` 는 숫자 id 만 열쇠, 삭제 · 목록 갱신 때 안 지움, `agents` 는 `INTEGER PRIMARY KEY` 라 가장 큰 id 를 지우고 등록하면 같은 id. 같은 꼴이 `gsDrafts`(상대 id — 모델 · effort)에도 있음(#1z-2 부터) | `view.draftOf` — 초안에 주인(에이전트 이름 · 상대 지문)을 함께 두고 다르면 버림, `view.pruneDrafts` — 목록에 없는 id 의 초안 버림(`renderAgentList` · `renderConnPage`), 삭제 성공이면 그 초안 지움(에이전트 · 상대). 시험 `view.test.mjs` `draft_owner`(node 170 pass). 실제 화면: `coder`(id 1) 폴더 `C:\Users` · 추가 지시 초안 → 삭제 → `fresh` 등록(같은 id 1) → 상세 폴더 `C:\Windows\Temp` · 추가 지시 빈 칸 · 상위 탐색 끔(초안 유입 없음). `cargo test -p coworkagent-app` 통과 |

## 적대적 평가 — 라운드 2/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex`, 작업 폴더 `C:/Users/tskim/.claude/skills`), handle `term_7bb12baf-d094-41e4-83f0-711bf21806a1`.
Run `run_a14062c3e398` · Task `task_f72db4b92523` · Dispatch `ctx_1c1e3c3df131`, worker_done(`msg_a5ee5fb88f6d`) 발신 handle 일치 · outcome succeeded. 범위 `866be6e..08b8195`(self-collect),
메타 `요청 1z-8-settings-search-r2 · revision 08b81956c5b44304373f0a3f924d954623d91ffd · 라운드 2/2`. 리뷰 동안 대상 작업 폴더 HEAD 08b8195 그대로(그 사이 손댄 것 없음 — masterPlan · PRODUCT 미커밋 문서는 리뷰 전부터).
리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal` · 원문 transcript `captured`). 원문(render):

```text
# Codex Adversarial Review

Target: branch diff against main
Verdict: needs-attention

고정 범위와 명세를 읽기 전용으로 검토했다. 같은 이름 재등록에서 R1 초안 격리가 실패하므로 출하 보류다. 등록 세대 구분과 해당 회귀 검증이 남았다.

Findings:
- [medium] 같은 이름으로 재등록하면 이전 프로필의 초안이 다시 적용됨 (crates/app/web/app.js:1448-1459)
  agentDetail은 초안 소유자를 a.name으로만 비교한다. agents.id는 AUTOINCREMENT 없는 INTEGER PRIMARY KEY이고 agents::add_with는 같은 이름 재등록을 허용하므로, 다른 탭이나 CLI에서 최대 id 프로필을 삭제하고 같은 이름으로 다시 만들면 id와 이름이 모두 재사용된다. 기존 탭이 삭제 중간 목록을 보지 않고 다른 에이전트 저장 후 loadAgents 등으로 새 목록을 받으면 pruneDrafts도 이를 남긴다. 고정 revision의 실제 draftOf/pruneDrafts 실행과 SQLite 메모리 DB에서 재현했다. 정적 경로 추적상 이후 새 프로필 상세에 옛 folder/prompt/parent_read/shell이 복원되고 저장 시 새 프로필을 덮어써 실행 범위와 지시가 오염된다. 브라우저 전체 경로는 미실행이다.
  Recommendation: 재사용 가능한 이름 대신 등록 세대를 구분하는 식별자를 API에 제공해 초안 소유자에 사용하라(기존 agents.uid 또는 별도 등록 UUID). 같은 id·같은 이름·다른 세대로 재등록한 뒤 목록을 한 번만 갱신하는 회귀 시험을 추가하라.

Next steps:
- 같은 이름 재등록의 초안 격리를 수정하고 회귀 시험으로 검증한다.
- 라운드 2/2 종료: 위 미조치 지적을 인계하며 R3는 시작하지 않는다.
- 기존 workspace/clippy/E8 통과는 제공된 증거이며 이번에 재실행하지 않았다. macOS·실제 두 PC 연동은 미검증이다.
```

판정 · 조치:

| 지적 | 판정 | 조치 |
|---|---|---|
| R2-1 medium 같은 이름으로 재등록하면 이전 프로필의 초안이 다시 적용 | 사실 — R1 주인 = 이름이라 다른 탭 · CLI 에서 가장 큰 id 를 지우고 같은 이름으로 다시 등록하면(이 탭이 그 사이 목록을 못 봄) id · 이름 모두 같음 | 에이전트 JSON 에 등록 세대 `uid`(이미 있는 `agents.uid` — 등록마다 새 값, 폴더 · 상위 · 셸 저장이면 새 값)를 내보내고 초안 주인 = `uid`. 시험 `chat_api` `agents_crud_api` — 같은 이름 다시 등록 = 같은 id · 다른 `uid`(9 passed). 실제 화면: 고친 `app.js` 제공 확인 · 다른 화면 갔다 와도 초안 유지(A7 회귀 없음). 다른 탭이 폴더 · 상위 · 셸을 저장하면 이 탭의 초안은 버려진다(새 세대 — 옛 값으로 덮지 않음) |

상한 2/2 — 재리뷰 없음. next step 중 "workspace · clippy · E8 재실행 안 함" 은 리뷰어 쪽 사정(coordinator 가 종결 검증에서 다시 돌림), "macOS · 실제 두 PC 연동 미검증" 은 기존 조건 대기(19 acceptance · Mac 실기 — D28).

R1-1 · R2-1 모두 반영. 수정 뒤 재리뷰는 하지 않았다(상한) — R2-1 수정(`uid` 주인)은 리뷰어가 보지 않았다.

## 미조치 지적

없음
