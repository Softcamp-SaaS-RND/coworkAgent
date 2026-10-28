# plan — 1z-17.time-date

> 근거: [intent](intent.md)(승인 2026-10-28 — 결정 1 ~ 9). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-17-time-date`(원 작업 폴더 — worktree 없음). 기준 커밋: `742e6e6`(main — v1.4.1 릴리스 뒤).
> 승인: 2026-10-28 — 사용자 "나머지는 결정사항 승인. 개발 진행해"(intent 승인과 함께 plan 승인 질문 없이 구현 — #1z-16 과 같은 구현자 해석, report 에 적음).
> 개정 2026-10-28 — 개발 중 추가 요청 2 · 3(intent 결정 10 ~ 12 · A8 · A9): stage 1 의 7 · 8, 평가 항목 E3 · E5 · E7 보강 · E9 추가. E7 에 좁은 시간 칸 확인(첫 보고에서 약속한 것)을 더함.

## 목표

화면 시각에 날짜를 붙이고(설정 > 화면 '날짜 표시방식' 4가지, 기본 `M.D HH:MM`), 설정 실행 · 화면 · 시작 · 알림 · 디버깅 타일을 1열 · 최대 너비 900px 로,
레일 상태 점을 키운다(큰 14px · 작은 12px). 에이전트 상세 저장 · 삭제에 창을 띄우고, 에이전트 시스템 프롬프트에서 추가 지시를 맨 앞에 둔다. 버전 1.4.2.

### 성공 기준(intent 사본)

- A1: `formatTime` 을 쓰는 모든 화면 시각이 '날짜 표시방식' 을 따른다 — 기본 `M.D HH:MM`(다른 해 `YYYY.M.D HH:MM`). 말풍선 위 · 에이전트 답 · 요청 카드 · 요청 의견 ·
  목록 · 활동 · 피어 기록 · 그룹 포함.
- A2: 설정 > 화면에 '날짜 표시방식' 칸 — ① `M.D HH:MM`(기본) ② `오늘 · 어제 HH:MM`, 그 전은 ①과 같음 ③ `YYYY-MM-DD HH:MM` ④ `HH:MM`. ①②는 다른 해면 `YYYY.M.D`.
  고르면 저장되고(다시 열어도 그대로) 열린 화면 시각이 바로 바뀐다. 모르는 값은 서버가 거절한다.
- A3: 실행 상세 타임라인 요청 · 응답 시각 = 고른 방식 + `:SS`(① `10.28 14:03:21` 등). 디버깅 로그 시각은 그대로 `MM-DD HH:MM:SS`.
- A4: 세부정보 '대화 정보' 기간 · 압축 시각 = 고른 방식, 날짜가 겹치지 않는다(① `10.28 16:05`, ② `오늘 16:05`).
- A5: 설정 실행 · 화면 · 시작 · 알림 · 디버깅 타일이 창 너비와 무관하게 1열, 타일 묶음 너비 ≤ 900px(넓은 창에서 이름과 옵션 칸 간격이 900px 안). 디버깅 로그 상자 너비는 그대로.
- A6: 레일 상태 점 = 큰 타일 14px · 작은 타일 12px, 색(대기 · 온라인 · 집중) · 위치(오른쪽 아래 모서리) · 테두리 그대로.
- A7: 그 밖 기존 동작 · 문구 그대로(날짜 구분선 포함), 검증 명령 PASS, 매뉴얼 반영, 버전 1.4.2.
- A8: 에이전트 상세 '저장' 성공 = "에이전트 {이름}을(를) 저장했습니다." 창, 실패 = "에이전트 {이름} 저장 실패: {사유}" 창. '삭제' = 확인 창(이름 · 세션 끊김 안내),
  취소면 지우지 않고 확인이면 지운다.
- A9: 추가 지시가 있으면 시스템 프롬프트(Claude `--append-system-prompt` · Codex `developer_instructions`)가 `[수신자 추가 지시]\n{지시}` 로 시작하고 가드 · 읽기 범위가 뒤에
  그대로 온다. 없으면 가드로 시작(지금과 같음).

## 설계

### 공통 규칙

새 함수는 둘레 규약대로(`export` · `pub` — 파일 전용 static 함수 없음). 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저. 같은 개념 주석 · 매뉴얼은 함께 고친다.

### stage 1 — 설정 저장소 · 화면(web) · 버전

1. **서버 설정 키**(`ui_tile_size` 와 같은 길):
   - `crates/core/src/run_opts.rs` — `DATE_FORMAT_KEY = "ui_date_format"` · `DATE_FORMATS = ["md", "relative", "ymd", "time"]`(첫 값 기본) · `date_format(conn)`(없거나 틀리면 `md`).
   - `crates/app/src/notify.rs` — 패치 필드 `ui_date_format`, 모르는 값 `invalid("ui_date_format 는 md · relative · ymd · time")`, 저장 목록 · `settings_json` 에 `ui_date_format`.
   - 시험 `crates/app/tests/ux_api.rs` `date_format_setting_persists` — 기본 `md`, `relative` 저장 · 다시 읽음, `"huge"` 400.
2. **`js/logic/format.js`**:
   - `DATE_FORMATS`(값 넷) · `setDateFormat(f)`(모르는 값 = `md`, 바뀌었으면 `true`) · 모듈 변수 기본 `md`.
   - `dateText(ts, now)` — 올해 `M.D` · 다른 해 `YYYY.M.D`. `dayLabel` 의 오늘 · 어제 밖 분기가 이것을 쓴다(구분선 글자 그대로).
   - `formatTime(unix, { seconds = false, fmt = 현재 방식, now = 지금 } = {})` — `md` = `dateText` + 시각, `relative` = `dayLabel` + 시각,
     `ymd` = `YYYY-MM-DD` + 시각, `time` = 시각만. `seconds` 면 `HH:MM:SS`. `null` → `""`.
   - `formatClock` 은 그대로(디버깅 로그가 "MM-DD " 를 붙여 쓴다).
3. **호출처**: `logic/message.js` `runSteps` 요청 · 응답 시각 = `formatTime(t, { seconds: true })`. `logic/conv.js` `convInfoRows` 의 `when` = `formatTime(t, { now })`
   (`dayLabel` 앞붙임 뺌). 그 밖 `formatTime(x)` 호출은 그대로(함수가 방식을 따른다).
4. **설정 화면**: `index.html` 화면 절에 타일 '날짜 표시방식'(`<select id="ui_date_format">`, 아이콘 `clock`, 옵션 넷 — 글자에 보기 예시).
   `js/settings/prefs.js` `PREFS` 에 `ui_date_format`, `fillPrefs` 에서 `setDateFormat`. 바꿔 저장하면 목록 · 대화의 `data-sig` 를 비우고 `refresh()` —
   이미 그린 시각이 새 방식으로 다시 그려진다(`needsRender` · `messagesSig` 가 같은 값이면 건너뛰므로).
5. **`app.css`**:
   - `.set-tiles` = 1열(`grid-template-columns: minmax(0, 1fr)`) + `max-width: 900px`. 960px 미디어 질의에서 `.set-tiles` 를 뺀다(`.md-cols` 는 그대로). 디버깅 로그 상자(`.debug-log`)는 그대로.
   - `.rail-dot` 14px, `#app.tile-small .rail-dot` 12px — 위치 · 테두리 · 색 그대로.
6. **버전** 1.4.1 → 1.4.2: `crates/{core,net,agent,app}/Cargo.toml` + `Cargo.lock`.
7. **에이전트 상세 창**: `js/logic/agent.js` `agentSavedText(name)` · `agentSaveFailText(name, reason)` · `agentDeleteAsk(name)`(글만 — 시험 가능, 조사 `josa`).
   `js/settings/agents.js` 저장 = 성공 `window.alert(agentSavedText)` · 실패 `window.alert(agentSaveFailText)` 뒤 다시 던져 상세 아래 글자도 그대로,
   삭제 = `window.confirm(agentDeleteAsk(a.name)) && act(…)` — 취소면 보내지 않는다.
8. **시스템 프롬프트 순서**: `crates/agent/src/runner.rs` `extra_prompt(agent)`(`[수신자 추가 지시]\n{지시}\n\n`, 없으면 빈 글자)를 맨 앞에 —
   Claude = 추가 지시 + 가드 + 읽기 범위, Codex = 추가 지시 + `CODEX_GUARD_PROMPT` + 읽기 범위. 시험 `crates/agent/tests/runner.rs` `guard_prompt_always` · `codex_args_follow_decisions`.

### 바꾸는 시험(의도된 변경 — 약화 아님)

- `view.test.mjs` "시간 표기" — `formatTime` 기본 = `M.D HH:MM`, 넷 방식 · 다른 해 · `seconds` 를 고정 시각으로. `convInfoRows` 기대값 `10.21 10:50 시작 · 마지막 10.23 16:05` · `압축 1회 · 10.22 09:00`.
  `runSteps` 시각 기대값 = `formatTime(t, { seconds: true })`.
- `ux_literals.rs` — `.set-tiles` 2열 · 960px 1열 literal(#1z-8)을 1열 · 900px 로, 새 PRESENT(날짜 표시방식 select · `.rail-dot` 크기 · 에이전트 저장 · 삭제 창).
- `crates/agent/tests/runner.rs` — "프로필 지시는 그 뒤에" 기대(가드로 시작 · 추가 지시로 끝)를 결정 12 순서(추가 지시로 시작 · 가드 뒤)로.

### stage 2 — 실제 화면 · 문서

- 디버그 빌드(`python tools/rebuild_run.py`) · ego-browser 두 테마 — 설정 화면 1열 · 넓은 창 900px, 날짜 표시방식 넷 전환(말풍선 · 요청 카드 · 사이드바 · 실행 상세 · 대화 정보 즉시 반영, 다시 열어도 유지),
  레일 상태 점(큰 · 작은 타일), 콘솔 오류 0.
- 문서: [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-17 줄) · README 1.4.2 줄 · masterPlan 환류(종결).

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1·A2·A3·A4·A8 — 웹UI 로직 시험(`formatTime` 넷 방식 · 다른 해 · 초, `runSteps` · `convInfoRows` 시각, 에이전트 저장 · 삭제 창 글)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E4: A2 — 설정 저장 · 거절(`ui_date_format` 기본 md · 저장 · 모르는 값 400)
  CHECK: cargo test -p coworkagent-app --test ux_api date_format_setting_persists
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: A2·A5·A6·A8 — 화면 원본 literal(날짜 표시방식 칸 · 1열 900px · 상태 점 크기 · 에이전트 저장 · 삭제 창) · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E6: A7 — 버전 1.4.2(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v142=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.2' for t in v)))"
  EXPECT: crates=4 v142=4
  EVIDENCE: pending
- [ ] E7: A1·A2·A3·A4·A5·A6·A8 — 실제 화면(디버그 빌드 · ego-browser) 두 테마 — 날짜 표시방식 넷 즉시 반영 · 유지, 설정 1열 · 900px, 레일 상태 점 큰 · 작은 타일,
  에이전트 저장 완료 창 · 삭제 확인 창(취소 = 남음), 사이드바 · 개요 목록 · 미리보기 시간 칸 줄바꿈 · 넘침 없음(1440 · 1000, md · ymd, 두 테마), 콘솔 오류 0
  EVIDENCE: pending
- [ ] E8: A7 — 문서 규약 · 크레이트 경계 · 코드 정리 · README · 매뉴얼 반영
  CHECK: python tools/check_ddd_docs.py && python tools/check_crates.py && python tools/check_code_tidy.py && python tools/check_readme.py && python -c "import pathlib as P;r=lambda f:P.Path(f).read_text(encoding='utf-8');print('web=%d guide=%d product=%d readme=%d'%(r('docs/manual/dev/web-ui.md').count('ui_date_format'),r('docs/manual/user/user-guide.md').count('날짜 표시방식'),r('PRODUCT.md').count('#1z-17'),r('README.md').count('1.4.2')))"
  EXPECT: /web=[1-9][0-9]* guide=[1-9][0-9]* product=[1-9][0-9]* readme=[1-9][0-9]*/
  EVIDENCE: pending
- [ ] E9: A9 — 시스템 프롬프트 순서(Claude · Codex 추가 지시 맨 앞, 가드 · 읽기 범위 뒤)
  CHECK: cargo test -p an-agent --test runner
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
