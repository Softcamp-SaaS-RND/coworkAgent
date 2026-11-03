# eval.md — 1z-19.tray-guide 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: 42d323bf6b646f8bd794f944a76cb425162bce38

- [x] E1: A1 — 트레이 `매뉴얼` 상수 · 메뉴 literal(열기 · 매뉴얼 · 종료)
  CHECK: cargo test -p coworkagent-app --lib tray && cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.20s

- [x] E2: A1 — 메뉴가 여는 주소가 실재(HTTP 200)
  CHECK: python -c "import re,urllib.request as u;s=open('crates/app/src/tray.rs',encoding='utf-8').read();m=re.search(r'USER_GUIDE: &str =\s*.(https://github[.]com/[^ ;]+user-guide[.]md)',s);print('url=%d status=%d'%(m is not None,u.urlopen(m.group(1),timeout=20).status if m else 0))"
  EXPECT: url=1 status=200
  EVIDENCE: url=1 status=200

- [x] E3: A1 — 실제 앱(Windows 릴리스 빌드 또는 디버그 빌드) 트레이 메뉴 순서 열기 · 매뉴얼 · 종료, `매뉴얼` 클릭 → 기본 브라우저에 GitHub 가이드, 확인할 일 표시 그대로, 오류 모드 메뉴 변화 없음
  EVIDENCE: Claude(구현자) 2026-11-03 11:25 — 메뉴 글자 개정(`매뉴얼`) 뒤 다시: Windows 11, 릴리스 빌드 1.4.4(`python tools/rebuild_run.py --yes` rc 0, `target/release/coworkAgent.exe` FileVersion 1.4.4, 사용자 데이터 루트). 알림 영역 아이콘을 UI Automation 으로 찾아 우클릭 → 메뉴 "열기 · 매뉴얼 · 종료"(화면 캡처 `docs/images/tray-menu.png` 다시 찍음), "매뉴얼" 클릭 → 기본 브라우저(ego) 창 "coworkAgent/docs/manual/user/user-guide.md at main · Softcamp-SaaS-RND/coworkAgent", 앱 로그 "매뉴얼 열기 실패" 0줄. 확인할 일 = 메뉴 길이 `on_seen` 을 부르지 않음(`tray.rs` guide_id 분기), 오류 모드 `run_error` diff 없음. 처음 확인(디버그 빌드 · 글자 `사용자 매뉴얼`) — 근거 [work/stage2](work/stage2.md).

- [x] E4: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

- [x] E5: clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.24s

- [x] E6: 웹UI 로직 시험(회귀 — 웹 원본 무변경)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: # fail 0

- [x] E7: 버전 1.4.4(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v144=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.4' for t in v)))"
  EXPECT: crates=4 v144=4
  EVIDENCE: crates=4 v144=4

- [x] E8: A2·A3 — 가이드 320줄 이하 · 캡처 11장(웹 10 + 트레이 1) 참조 · 참조한 그림 전부 실재 · 보안 · 반출 안내 literal 7개(외부 모델 서비스 · 사내망 · 포트 포워딩 · 셸 명령 실행 허용 · 방화벽 · macOS 격리 해제 · 읽기 전용) 유지
  CHECK: python -c "import re,pathlib as P;f=P.Path('docs/manual/user/user-guide.md');t=f.read_text(encoding='utf-8');n=sorted(set(re.findall(r'\]\((\.\./\.\./images/[^)]+\.png)\)',t)));need=['home','group','pairing','agents','new-conv','workspace','requests','relay','notify','settings','tray-menu'];keys=['Claude 모델 서비스','사내망','포트 포워딩','셸 명령 실행 허용','방화벽','xattr -d com.apple.quarantine','읽기 전용'];print('lines_ok=%d refs=%d exist=%d need=%d keys=%d present=%d'%(len(t.splitlines())<=320,len(n),sum((f.parent/x).exists() for x in n),sum(('../../images/%s.png'%k) in n for k in need),len(keys),sum(k in t for k in keys)))"
  EXPECT: /lines_ok=1 refs=(1[1-9]|[2-9][0-9]) exist=\1 need=11 keys=7 present=7/
  EVIDENCE: lines_ok=1 refs=11 exist=11 need=11 keys=7 present=7

- [x] E9: A3 — 캡처 스크립트(mock) 오류 0 · 11장이 v1.4.4 화면 · 가상 데이터만(실 사용자 · PC 이름 · 경로 없음) — 그림을 눈으로 대조
  EVIDENCE: Claude(구현자) 2026-11-03 09:00 — mock(`design/mock.py`, 라이트) + ego-browser `design/shots-ego.mjs` 1440×900, 마지막 실행 mock stderr "404" 0줄 · `#disconnected` 없음. 10장(home · group · pairing · agents · new-conv · workspace · requests · relay · notify · settings) 머리 "coworkAgent v1.4.4", 인물 = 박지훈(jhpark · PARK-PC) · 이준호 · 최유나 · build(가상), 경로 `D:\work\shop-api`(가상). 확장 프로그램 오버레이(#ds365-convert-panel-host · #crx-mouse-redesign-content-root) 숨김 뒤 캡처. tray-menu.png = 메뉴 상자만 잘라 냄(사용자 이름 · PC 이름 · 다른 앱 없음). 평가자 눈 대조 대기 — 근거 [work/stage2](work/stage2.md).

- [x] E10: A2 — 가이드의 화면 글자(단추 · 메뉴 · 설정 이름)가 앱 원본 literal 과 같다 · 장마다 번호 단계 + 캡처 — 평가자 대조
  EVIDENCE: Claude(구현자) 2026-11-03 — 가이드에 쓴 화면 글자 78개(예: "새로 만들기" · "초대 복사" · "내 응답 에이전트" · "최대 응답 횟수" · "archive 항목 보기" · "기록 삭제" · "내 글만 삭제" · "진단 정보 복사" · "날짜 표시방식" · "매뉴얼" · 실패 문구 3)를 `crates/**/*.{html,js,rs}`(시험 제외)에서 찾아 없는 것 0. 장 1 ~ 9 마다 번호 단계, 캡처 11장(1.3 tray-menu · 1.4 home · 1.5 group · 2 pairing · 3 agents · 4.1 new-conv · 4.2 workspace · 5 requests · 6 relay · 7 notify · 8 settings). 평가자 대조 대기 — 근거 [work/stage3](work/stage3.md).

- [x] E11: A4 — 문서 규약 · README · 코드 정리 · 크레이트 경계 · README 의 가이드 장 번호(`§n`)가 새 가이드 heading 에 전부 있음 · 1.4.4 줄
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_code_tidy.py && python tools/check_crates.py && python -c "import re,pathlib as P;r=P.Path('README.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');refs=set(re.findall(r'§(\d+(?:\.\d+)?)',r));heads=set(re.findall(r'^#{2,4} (\d+(?:\.\d+)?)\.? ',g,re.M));print('readme144=%d sec_refs=%d missing=%d'%(r.count('1.4.4'),len(refs),len(refs-heads)))"
  EXPECT: /readme144=[1-9][0-9]* sec_refs=[1-9][0-9]* missing=0/
  EVIDENCE: readme144=1 sec_refs=16 missing=0


WEAK: E3 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)
WEAK: E9 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)
WEAK: E10 CHECK 없음 — 수동 gate(사람이 적은 EVIDENCE 로 판정)

