# eval.md — 1z-17.time-date 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: bc200c9a36dd71c6c4a63393f783d78eabc3c6fc

- [x] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E2: A7 — clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.25s

- [x] E3: A1·A2·A3·A4·A8 — 웹UI 로직 시험(`formatTime` 넷 방식 · 다른 해 · 초, `runSteps` · `convInfoRows` 시각, 에이전트 저장 · 삭제 창 글)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E4: A2 — 설정 저장 · 거절(`ui_date_format` 기본 md · 저장 · 모르는 값 400)
  CHECK: cargo test -p coworkagent-app --test ux_api date_format_setting_persists
  EXPECT: /test result: ok\. 1 passed; 0 failed/
  EVIDENCE: test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 50 filtered out; finished in 0.10s

- [x] E5: A2·A5·A6·A8 — 화면 원본 literal(날짜 표시방식 칸 · 1열 900px · 상태 점 크기 · 에이전트 저장 · 삭제 창) · `:root` 밖 리터럴 0
  CHECK: cargo test -p coworkagent-app --test ux_literals --test css_tokens
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s

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
  EVIDENCE: test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.03s


WEAK: E7 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)


## 적대적 평가

- 미실시 — 사용자 지시("서명 추가했어. gh release 등록 진행해. 수정 내역은 push 진행", 2026-10-28)로 gate 판정 뒤 릴리스 · push 를 먼저 했다.
  codex-peer-review(라운드 상한 2)는 종결 전에 한다 — 지적이 나오면 1.4.2 이후 수정으로 다룬다.
