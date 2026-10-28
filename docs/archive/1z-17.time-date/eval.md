# eval.md — 1z-17.time-date 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 650e8b7375699984861490eb748337a6e2c3c76a

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E2: A7 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.33s

- [x] E3: A1·A2·A3·A4·A8 — 웹UI 로직 시험(`formatTime` 넷 방식 · 다른 해 · 초, `runSteps` · `convInfoRows` 시각, 에이전트 저장 · 삭제 창 글)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A2 — 설정 저장 · 거절(`ui_date_format` 기본 md · 저장 · 모르는 값 400)
  CHECK: cargo test -p coworkagent-app --test ux_api date_format_setting_persists
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.18s

- [x] E5: A2·A5·A6·A8 — 화면 원본 literal(날짜 표시방식 칸 · 1열 900px · 상태 점 크기 · 에이전트 저장 · 삭제 창) · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s

- [x] E6: A7 — 버전 1.4.2(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v142=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.2' for t in v)))"
  EXPECT: crates=4 v142=4
  EVIDENCE: crates=4 v142=4

- [x] E7: A1·A2·A3·A4·A5·A6·A8 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 날짜 표시방식 넷 즉시 반영 · 유지, 설정 1열 · 900px, 레일 상태 점 큰 · 작은 타일,
  EVIDENCE: Claude(구현자) 2026-10-28 — 디버그 빌드 1.4.2(`target\t17`) + scratchpad `ui1z17\data`(#1z-16 `ui1z16` 사본), ego-browser 1440 · 1000 · 1920 × 900 라이트 · 다크(r01 ~ r05). 날짜 표시방식 select 로 md · relative · ymd · time 바꿈 → 서버 저장 확인, 대화 2 말풍선 · 요청 의견 · 사이드바 · 대화 정보가 즉시 그 방식(md "10.23 10:51" · ymd "2026-10-23 10:51" · time "10:51"), 다시 열기 뒤 유지. 설정 실행 · 화면 · 시작 · 알림 · 디버깅 `.set-tiles` 1열 · 너비 900(1440 · 1920), 디버깅 로그 상자 989. 레일 점 14 × 14 · 작은 타일 12 × 12. 에이전트 저장 alert "에이전트 coder을(를) 저장했습니다." · 없는 폴더 → 실패 alert · 삭제 confirm 취소 → 남음(alert · confirm 을 기록 함수로 바꿔 판독). 좁은 시간 칸(사이드바 272 · 창 1440 · 1000 · md · ymd · 두 테마) 줄바꿈 · 넘침 없음 — ymd 에서 사이드바 제목 칸 62px(사용자 결정 stage2-1). 드롭다운 select 7개 모두 옵션 글이 안쪽 너비 안. 실행 상세 시각은 시험 데이터에 실행 기록이 없어 화면 판독 없음(E3 run_steps 시험이 고정). 콘솔 · 페이지 오류 0. 근거 [work/stage2](work/stage2.md).

- [x] E8: A7 — 문서 규약 · 크레이트 경계 · 코드 정리 · README · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d readme=%d'%(r('docs/manual/dev/web-ui.md').count('ui_date_format'),r('docs/manual/user/user-guide.md').count('날짜 표시방식'),r('PRODUCT.md').count('#1z-17'),r('README.md').count('1.4.2')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* readme=[1-9][0-9]*/
  EVIDENCE: web=3 guide=1 product=1 readme=1

- [x] E9: A9 — 시스템 프롬프트 순서(Claude · Codex 추가 지시 맨 앞, 가드 · 읽기 범위 뒤)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.55s


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

## 측정 기록

- 첫 측정 `bc200c9`(릴리스 전) 9/9 PASS → 사용자 지시로 v1.4.2 릴리스 · push 를 먼저 함.
- 리뷰 라운드 1 · 2 수정(`a2bbe55` · `650e8b7`) 뒤 다시 — 원 작업 폴더에는 #1z-18 미커밋 변경(버전 1.4.3)이 있어 `650e8b7` 을 따로 꺼낸 임시 작업 트리
  (scratchpad `wt17`, `git worktree add --detach`)에서 측정: 첫 실행 E8 만 미충족(원장의 옛 측정 커밋 — 문서 규약 체커), 둘째 실행 9/9 PASS. 원장을 이 파일로 옮김.

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex --model gpt-6.1-sol --effort high` — receipt `launch.effective` 같음, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_c98d5262d013` · Task `task_ebc69048fe65` · Dispatch `ctx_b41d065dee28` · 터미널 `term_9b4e63fb-ad3e-4de6-87fd-c4381145c247`(시작 receipt `input_accepted`).
범위 `742e6e6..94bd4b2`(self-collect — v1.4.2 릴리스 squash), 메타 `요청 1z-17-close · revision 94bd4b294898e814c02afdae8264e0a0e4dcdd8d · 라운드 1/2`, worker_done `msg_3e30a5663443`. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).
(이 Task 는 사용자 지시로 릴리스 · push 를 먼저 했다 — 리뷰는 릴리스 뒤.)

```text
# Codex Adversarial Review

Target: branch diff against 742e6e657e9cdf67edeb6cd76fa9fccdbe6a0392
Verdict: needs-attention

고정 revision의 결정 1~13·E1~E9와 diff를 읽기 전용으로 대조했다. 날짜 표시 정확성을 깨는 자정 캐시와 설정 응답 역전 위험 2건으로 종결 보류 의견이다. 경계 회귀 검증과 수정이 남으며 전체 빌드·브라우저·macOS 실기는 재실행하지 않았다.

Findings:
- [medium] 자정 이후에도 캐시된 시각이 계속 오늘로 표시됨 (crates/app/web/js/logic/format.js:84-91, confidence 0.92)
  [확인] formatTime이 현재 날짜에 의존하도록 바뀌었지만 sidebar.js:162~169의 목록 서명과 conv.js:459~461·511~518의 메시지 서명에는 현재 날짜가 없다. 같은 데이터를 5초마다 조회해도 다시 그리지 않아 relative에서 전날 글이 계속 오늘로 남는다. 고정 revision의 formatTime·unchanged를 조합한 축소 재현에서 10월 29일 00:01에도 오늘 23:50이 유지됐고 직접 계산값은 어제 23:50이었다. [추론] 전체 화면에서도 같은 캐시 분기가 적용되며, md의 연도 생략도 연말을 넘기면 잘못 유지된다; 브라우저 전체 재현은 미실행.
  Recommendation: 시각을 포함한 렌더 서명에 현재 지역 날짜와 날짜 형식의 공통 키를 넣거나 날짜 변경 시 관련 캐시를 무효화하라. 동일 데이터로 자정·연말을 넘기는 회귀 검증을 추가하라.
- [medium] 다른 설정의 늦은 응답이 저장한 날짜 형식을 되돌림 (crates/app/web/js/settings/prefs.js:48-48, confidence 0.86)
  [확인] fillPrefs는 모든 설정 응답의 ui_date_format을 무조건 적용한다; 각 입력만 비활성화하므로 다른 설정 요청은 동시에 가능하다(prefs.js:63~70). [추론] 알림 설정 응답이 md인 상태로 지연된 동안 날짜를 ymd로 저장하고 그 응답을 먼저 받으면, 늦게 도착한 알림 응답이 select·state.settings·formatTime을 md로 되돌린다. 서버는 ymd를 유지하고 캐시 무효화는 날짜 입력 처리에서만 하므로 화면이 저장값과 어긋나며 기존 ymd 시각과 새 md 시각이 섞일 수 있다. HTTP 응답 역전·브라우저 재현은 미실행.
  Recommendation: 초기 로딩 이후 다른 설정의 응답으로 ui_date_format을 덮지 말고, 해당 입력의 최신 요청만 날짜 형식을 적용하도록 하라. 실제 적용값이 바뀌는 공통 경로에서 시각 캐시를 무효화하고 응답 역전 회귀 검증을 추가하라.

Next steps:
- 자정·연말 캐시 무효화와 설정 응답 역전 처리 후 라운드 2/2에서 고정 수정 revision을 검토한다.
- 원격 본문 stdin 경로·가드 문구·도구 제한 유지에는 이번 diff에서 방어 가능한 새 결함을 찾지 못했다; macOS 실기는 별도 검증 공백이다.
```

raw JSON(worker_done 본문 둘째 줄):

```json
{"verdict":"needs-attention","summary":"고정 revision의 결정 1~13·E1~E9와 diff를 읽기 전용으로 대조했다. 날짜 표시 정확성을 깨는 자정 캐시와 설정 응답 역전 위험 2건으로 종결 보류 의견이다. 경계 회귀 검증과 수정이 남으며 전체 빌드·브라우저·macOS 실기는 재실행하지 않았다.","findings":[{"severity":"medium","title":"자정 이후에도 캐시된 시각이 계속 오늘로 표시됨","body":"[확인] formatTime이 현재 날짜에 의존하도록 바뀌었지만 sidebar.js:162~169의 목록 서명과 conv.js:459~461·511~518의 메시지 서명에는 현재 날짜가 없다. 같은 데이터를 5초마다 조회해도 다시 그리지 않아 relative에서 전날 글이 계속 오늘로 남는다. 고정 revision의 formatTime·unchanged를 조합한 축소 재현에서 10월 29일 00:01에도 오늘 23:50이 유지됐고 직접 계산값은 어제 23:50이었다. [추론] 전체 화면에서도 같은 캐시 분기가 적용되며, md의 연도 생략도 연말을 넘기면 잘못 유지된다; 브라우저 전체 재현은 미실행.","file":"crates/app/web/js/logic/format.js","line_start":84,"line_end":91,"confidence":0.92,"recommendation":"시각을 포함한 렌더 서명에 현재 지역 날짜와 날짜 형식의 공통 키를 넣거나 날짜 변경 시 관련 캐시를 무효화하라. 동일 데이터로 자정·연말을 넘기는 회귀 검증을 추가하라."},{"severity":"medium","title":"다른 설정의 늦은 응답이 저장한 날짜 형식을 되돌림","body":"[확인] fillPrefs는 모든 설정 응답의 ui_date_format을 무조건 적용한다; 각 입력만 비활성화하므로 다른 설정 요청은 동시에 가능하다(prefs.js:63~70). [추론] 알림 설정 응답이 md인 상태로 지연된 동안 날짜를 ymd로 저장하고 그 응답을 먼저 받으면, 늦게 도착한 알림 응답이 select·state.settings·formatTime을 md로 되돌린다. 서버는 ymd를 유지하고 캐시 무효화는 날짜 입력 처리에서만 하므로 화면이 저장값과 어긋나며 기존 ymd 시각과 새 md 시각이 섞일 수 있다. HTTP 응답 역전·브라우저 재현은 미실행.","file":"crates/app/web/js/settings/prefs.js","line_start":48,"line_end":48,"confidence":0.86,"recommendation":"초기 로딩 이후 다른 설정의 응답으로 ui_date_format을 덮지 말고, 해당 입력의 최신 요청만 날짜 형식을 적용하도록 하라. 실제 적용값이 바뀌는 공통 경로에서 시각 캐시를 무효화하고 응답 역전 회귀 검증을 추가하라."}],"next_steps":["자정·연말 캐시 무효화와 설정 응답 역전 처리 후 라운드 2/2에서 고정 수정 revision을 검토한다.","원격 본문 stdin 경로·가드 문구·도구 제한 유지에는 이번 diff에서 방어 가능한 새 결함을 찾지 못했다; macOS 실기는 별도 검증 공백이다."]}
```

판정 · 조치:
- 지적 1(자정 캐시) — **사실**. stage 1 에 '한계' 로 적었던 것 + 연도 생략의 해 넘김. `format.js` `timeKey()`(방식 · `dayKey`)를 `core/dom.js` `unchanged` · `needsRender` 와 `messagesSigOf` 서명에 붙임,
  날짜 방식 저장 때 `[data-sig]` 를 비우던 우회는 지움. 시험 `time_key`(RED 먼저 — export 없음 SyntaxError).
- 지적 2(응답 역전) — **사실(추론 — 재현은 리뷰어 축소 재현)**. 설정 변경 처리에서 PUT 응답의 바꾼 키만 `fillPrefs`. → 라운드 2 가 다른 PUT 경로 누락을 지적.
- 수정 커밋 `a2bbe55`. node 181/0 · cargo test 84 ok · clippy · tidy, 디버그 빌드 화면: ymd · time · md 전환 즉시 대화 · 사이드바 반영, ymd 저장 뒤 테마 저장해도 ymd 유지.

## 적대적 평가 — 라운드 2/2

평가자: 같은 방식으로 새 Codex. Run `run_c98d5262d013` · Task `task_f68a700560dc` · Dispatch `ctx_32ca4b957dba` · 터미널 `term_f479d443-d8ef-4035-a965-be4a6e2c3a3f`(`input_accepted`, `gpt-6.1-sol` · `high`).
범위 `f4fdbe6..a2bbe55`(inline-diff — 라운드 1 수정), 메타 `요청 1z-17-close · revision a2bbe55c3b4f6a23e38cdf902335ea67b83eaa3d · 라운드 2/2`, worker_done `msg_6edbc4f64225`. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).

```text
# Codex Adversarial Review

Target: branch diff against f4fdbe65d0039031726258a7dac0268ff4ed13e0
Verdict: needs-attention

고정 범위의 수정·intent·plan을 읽기 전용으로 대조하고 경계 경로를 메모리 내 축소 재현했다. 자정 캐시 수정은 작동하지만 의견 입력 소실과 다른 PUT 경로의 응답 역전이 남아 종결 보류 의견이다. 전체 빌드·실제 자정·HTTP 응답 역전·브라우저·macOS 실기는 재실행하지 않았으며 잔여 2건을 라운드 2/2 결과로 남긴다.

Findings:
- [medium] 다른 설정 PUT 경로는 여전히 날짜 형식을 되돌림 (crates/app/web/js/settings/prefs.js:70-72, confidence 0.93)
  [확인] 변경 키 필터는 PREFS 처리에만 적용된다; shell.js:218·262와 tiles.js:156·275는 전체 PUT 응답을 fillPrefs에 전달하고 실패 복구도 prefs.js:82에서 전체 GET을 적용한다. 고정 소스·모의 응답으로 집중 PUT의 md 응답을 지연시킨 뒤 날짜 PUT의 ymd 응답을 먼저 적용하자, 뒤늦은 집중 응답이 select·state.settings·formatter를 md로 되돌렸다. [추론] 서버에는 ymd가 남아 R1의 저장값/표시값 불일치가 재발한다; 실제 HTTP·브라우저 재현은 미실행.
  Recommendation: 변경 키만 반영하는 공통 함수를 모든 설정 PUT 호출에 적용하라. 실패 복구 GET도 해당 키와 요청 세대에 한정하고, 집중·알림 일시 중지·타일 저장과 날짜 저장의 역전 회귀를 검증하라.
- [medium] 자정 재렌더가 대화 안의 작성 중 의견을 삭제함 (crates/app/web/js/logic/message.js:125-125, confidence 0.9)
  [확인] 추가한 timeKey로 동일 메시지도 자정에 서명이 바뀐다. 소비자인 conv.js:511~517은 입력 보호 없이 messages 전체를 replaceChildren하며, 대화의 noteForm(:295·394~437)은 의견을 textarea에만 보관한다. 고정 소스·모의 DOM 축소 재현에서 같은 날 5초 갱신은 유지됐지만 자정 뒤에는 활성 TEXTAREA가 있어도 하위 DOM이 교체됐다. [추론] 실제 대화에서 작성 중인 요청 의견이 사라진다; needsRender의 입력 보호는 이 경로에 적용되지 않으며 전체 브라우저 재현은 미실행.
  Recommendation: 대화의 날짜 갱신도 의견 입력 중에는 서명을 소비하지 않고 미루거나 시각 노드만 갱신하라. 초안의 값·대상 요청·초점을 보존하며 동일 데이터로 자정·연말을 넘기는 회귀를 추가하라.

Next steps:
- 남은 2건은 라운드 2/2 미조치 지적으로 인계한다; 추가 리뷰 라운드는 시작하지 않는다.
- timeKey의 자정·연말 변경, 같은 날 5초 캐시 유지, needsRender 입력 중 서명 미소비는 축소 검증 통과했다; 제공된 전체 테스트 결과는 독립 재실행하지 않았다.
```

raw JSON(worker_done 본문 둘째 줄):

```json
{"verdict":"needs-attention","summary":"고정 범위의 수정·intent·plan을 읽기 전용으로 대조하고 경계 경로를 메모리 내 축소 재현했다. 자정 캐시 수정은 작동하지만 의견 입력 소실과 다른 PUT 경로의 응답 역전이 남아 종결 보류 의견이다. 전체 빌드·실제 자정·HTTP 응답 역전·브라우저·macOS 실기는 재실행하지 않았으며 잔여 2건을 라운드 2/2 결과로 남긴다.","findings":[{"severity":"medium","title":"다른 설정 PUT 경로는 여전히 날짜 형식을 되돌림","body":"[확인] 변경 키 필터는 PREFS 처리에만 적용된다; shell.js:218·262와 tiles.js:156·275는 전체 PUT 응답을 fillPrefs에 전달하고 실패 복구도 prefs.js:82에서 전체 GET을 적용한다. 고정 소스·모의 응답으로 집중 PUT의 md 응답을 지연시킨 뒤 날짜 PUT의 ymd 응답을 먼저 적용하자, 뒤늦은 집중 응답이 select·state.settings·formatter를 md로 되돌렸다. [추론] 서버에는 ymd가 남아 R1의 저장값/표시값 불일치가 재발한다; 실제 HTTP·브라우저 재현은 미실행.","file":"crates/app/web/js/settings/prefs.js","line_start":70,"line_end":72,"confidence":0.93,"recommendation":"변경 키만 반영하는 공통 함수를 모든 설정 PUT 호출에 적용하라. 실패 복구 GET도 해당 키와 요청 세대에 한정하고, 집중·알림 일시 중지·타일 저장과 날짜 저장의 역전 회귀를 검증하라."},{"severity":"medium","title":"자정 재렌더가 대화 안의 작성 중 의견을 삭제함","body":"[확인] 추가한 timeKey로 동일 메시지도 자정에 서명이 바뀐다. 소비자인 conv.js:511~517은 입력 보호 없이 messages 전체를 replaceChildren하며, 대화의 noteForm(:295·394~437)은 의견을 textarea에만 보관한다. 고정 소스·모의 DOM 축소 재현에서 같은 날 5초 갱신은 유지됐지만 자정 뒤에는 활성 TEXTAREA가 있어도 하위 DOM이 교체됐다. [추론] 실제 대화에서 작성 중인 요청 의견이 사라진다; needsRender의 입력 보호는 이 경로에 적용되지 않으며 전체 브라우저 재현은 미실행.","file":"crates/app/web/js/logic/message.js","line_start":125,"line_end":125,"confidence":0.9,"recommendation":"대화의 날짜 갱신도 의견 입력 중에는 서명을 소비하지 않고 미루거나 시각 노드만 갱신하라. 초안의 값·대상 요청·초점을 보존하며 동일 데이터로 자정·연말을 넘기는 회귀를 추가하라."}],"next_steps":["남은 2건은 라운드 2/2 미조치 지적으로 인계한다; 추가 리뷰 라운드는 시작하지 않는다.","timeKey의 자정·연말 변경, 같은 날 5초 캐시 유지, needsRender 입력 중 서명 미소비는 축소 검증 통과했다; 제공된 전체 테스트 결과는 독립 재실행하지 않았다."]}
```

판정 · 조치(라운드 2 수정 — 상한이라 재리뷰 없음, 라운드 1 · 2 지적 4 건 모두 사실 · 고침 — 미조치 없음):
- 지적 1(다른 PUT 경로) — **사실**(`shell.js` 집중 · 알림 일시 중지, `tiles.js` '나' 타일 저장 · 그림 지우기, 실패 복구 GET). `prefs.js` `saveSettings(body)` — 보낸 키만 반영하는 공통 함수로 모두 바꿈,
  실패 복구도 그 키만. `ux_literals` `literals_absent` 에 `fillPrefs(await api("PUT"` · `fillPrefs(await api("GET"` 넣어 되돌아오지 않게(RED 먼저 — 2 FAILED).
- 지적 2(자정 재렌더가 쓰던 의견 지움) — **사실**(데이터가 바뀌어도 같은 일이 났다 — 자정이 하나 더 늘린 경로). `conv.js` `openConversation` — 대화에 의견 칸(`.req-note`)이 열려 있으면 다시 그리기를 미룸
  (닫거나 보내면 다음 새로 고침에 그린다). `ux_literals` PRESENT.
- 수정 커밋 `650e8b7`. cargo test 84 ok · clippy · tidy · node 181/0. 평가 원장은 `650e8b7` 을 따로 꺼낸 임시 작업 트리에서 다시 측정(원 작업 폴더에는 #1z-18 미커밋 변경).

## 미조치 지적

없음
