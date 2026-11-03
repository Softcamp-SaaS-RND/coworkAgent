# plan — 1z-19.tray-guide

> 근거: [intent](intent.md)(승인 2026-11-03 — 결정 1 ~ 7). **구현자는 이 문서만 보고 일한다.**
> 브랜치 `task/1z-19-tray-guide`(원 작업 폴더 — worktree 없음). 기준 커밋: `f30f75f`(main — v1.4.3 릴리스 뒤).
> 승인: 2026-11-03 — 사용자 "진행."
> 개정 2026-11-03 — 사용자 "화면캡처는 ego-browser 을 사용해서 진행.": stage 2 캡처 수단 `shots.mjs`(Edge headless · playwright-core) → ego-browser(`design/shots-ego.mjs`). 항목 · EXPECT 변경 없음.
> 개정 2026-11-03 — 사용자 "트레이 매뉴의 `사용자 매뉴얼` -> `매뉴얼`로 변경."(intent 결정 1 라운드 2): 메뉴 글자 `사용자 매뉴얼` → `매뉴얼` — 코드 · 시험 literal · 가이드 · 문서 · `tray-menu.png` 다시 찍기. 항목 · EXPECT 변경 없음.
> 보정 2026-11-03 — E2 CHECK 정규식 `= .` → `=\s*.`(상수가 줄을 넘겨 선언됨 — CHECK 보정), E8 EXPECT 의 역참조 `\1` 이 plan 작성 때 제어 문자(U+0001)로 깨져 들어간 것을 의도한 글자로 복구(EXPECT 의미 변경 없음), E3 EVIDENCE 경로의 탭 문자 복구.

## 목표

트레이 메뉴(정상 모드)에 `매뉴얼` 을 더해 GitHub main 의 사용자 가이드를 열고, 사용자 가이드를 기능별 짧은 따라 하기 + 캡처로 다시 쓴다. 버전 1.4.4.

### 성공 기준(intent 사본)

- A1: 정상 모드 트레이 메뉴에 `매뉴얼` 항목이 있고, 누르면 기본 브라우저에 GitHub 사용자 가이드 주소가 열린다(Windows · macOS 같은 코드 길).
- A2: 사용자 가이드가 기능별 따라 하기(단계 + 캡처)로 짧아졌다 — 화면 글자는 앱에 보이는 그대로.
- A3: 가이드의 캡처가 지금 화면(v1.4.3 이후)이고, 가상 데이터만 담는다(실 사용자 · PC 이름 · 경로 없음).
- A4: 가이드를 가리키는 링크(README · 매뉴얼 색인)가 깨지지 않는다 — `python tools/check_readme.py` · `python tools/check_ddd_docs.py` PASS.

## 실측(plan 작성 시)

- 트레이 메뉴 = `MenuItem::new("열기")` · `("종료")`, `build(&[open, quit], …)`, 클릭은 `browser::open(&url)` — `crates/app/src/tray.rs:137-140 · 195-201`. 오류 모드 `run_error` 는 따로(손대지 않음).
- `browser::open` 은 두 OS 공통 길(Windows `ShellExecuteW` · macOS `open` — masterPlan D30). 저장소는 public(`gh repo view` → `PUBLIC`).
- 화면 literal 시험 `crates/app/tests/ux_literals.rs:50-51` 이 트레이 메뉴 글자를 잡는다.
- 가이드 578줄 · 17장 · 캡처 2장(`group.png` · `home.png`). README 는 `docs/images/{workspace,pairing,agents}.png` 를 쓰고 가이드를 `§n` 으로 가리킨다(`README.md:24-33 · 99 · 125`).
- 개발자 가이드(988줄)에 감시 · 재시작 · 자동 업데이트 내부가 이미 있다(`developer-guide.md` 2.4 · 3.1 표) — 결정 4 는 대부분 삭제만.
- 캡처 도구: `docs/archive/1q.request-overview/design/mock.py`(가상 API) · `docs/archive/1p.project-tidy/design/shots.mjs`(Edge headless · playwright-core).
  웹이 부르는 API 경로는 mock 이 모두 응답한다(경로 대조) — 1q 이후 더해진 필드(에이전트 실행 도구 · 아이콘 색 · 그룹 등)는 화면을 띄워 보고 채운다.
  playwright-core 는 `%APPDATA%\npm\node_modules\@playwright\mcp\node_modules\playwright-core`(`PW` 환경 변수로 넘김).

## 설계

### 공통 규칙

새 함수 · 상수는 둘레 규약대로(파일 전용 `static` 함수 없음). 제품 원본 주석에 Task 번호 · 결정 번호 · 날짜 없음. TDD — 시험 먼저. 같은 개념 주석 · 매뉴얼은 함께 고친다.
원 작업 폴더의 다른 세션 미커밋(`PRODUCT.md` 끝 2줄 · 미추적 tech 노트 security-posture)은 커밋에서 뺀다 — `PRODUCT.md` 는 이 Task 줄만 `git apply --cached` 로 올린다.

### stage 1 — 트레이 메뉴 · 버전

1. `crates/app/src/tray.rs`:
   - `pub const USER_GUIDE: &str = "https://github.com/Softcamp-SaaS-RND/coworkAgent/blob/main/docs/manual/user/user-guide.md";`
   - `run` — `MenuItem::new("매뉴얼", true, None)` 를 열기 · 종료 사이에(`build(&[open, guide, quit], …)`). 클릭 = `browser::open(USER_GUIDE)`, 실패는 `log::error!("매뉴얼 열기 실패: {e}")`.
     `on_seen` 은 부르지 않는다(확인할 일 표시는 웹UI 열기로만 지움). 모듈 머리 주석 "메뉴 "열기" · "종료"" 도 함께.
   - 시험 `user_guide_url` — 상수 literal 단정(주소 전체 · `https://github.com/` 시작 · `/blob/main/docs/manual/user/user-guide.md` 끝).
2. `crates/app/tests/ux_literals.rs` — `("tray.rs", r#"MenuItem::new("매뉴얼""#, …)` 추가.
   **바꾸는 시험(의도된 변경 — 약화 아님)**: 같은 파일 834행 `MenuItem::new(` 개수 4 → 5(정상 모드 열기 · 매뉴얼 · 종료 + 오류 모드 오류 보기 · 종료).
3. 버전 1.4.3 → 1.4.4: `crates/{core,net,agent,app}/Cargo.toml` + `Cargo.lock`.

### stage 2 — 캡처 도구 · 캡처

1. `design/mock.py`(1q mock 사본) · `design/shots-ego.mjs`(ego-browser — 개정)를 이 Task 폴더에 두고 지금 화면(v1.4.4)에 맞춘다 — 가상 인물 · PC · 경로만, 콘솔 · 페이지 오류 0(`ERRORS []`).
2. 라이트 테마로 찍어 `docs/images/` 에 둔다. 장마다 한 장 이상 — 이름 고정:
   `home` · `group` · `pairing` · `agents` · `new-conv` · `workspace` · `requests` · `relay` · `notify` · `settings` (10장, 웹UI).
   README 가 쓰는 `workspace` · `pairing` · `agents` 도 같은 파일이라 함께 바뀐다.
3. `tray-menu.png` — 실제 앱(`python tools/rebuild_run.py`, Windows)의 트레이 메뉴(열기 · 매뉴얼 · 종료). 트레이 메뉴는 포커스를 잃으면 닫히는 임시 창이라
   headless 로 못 찍는다 — PowerShell(UI Automation 으로 알림 영역 아이콘 우클릭 → 화면 캡처)로 먼저 시도하고, 안 되면 **사용자에게 Win + Shift + S 캡처를 요청**한다(그림 파일만 받음).
   사용자 이름 · PC 이름이 보이면 잘라 낸다. 이 그림이 없으면 E8 은 미충족으로 남긴다(조용히 빼지 않음).

### stage 3 — 사용자 가이드 · 문서

**옛 장 → 새 장**(결정 3 의 장 목록에 없는 옛 장은 가까운 새 장의 절로 넣는다 — 빠뜨리는 장 없음):

| 옛 장 | 새 자리 | 비고 |
|---|---|---|
| 1 무엇을 하는 앱인가 | 1 시작하기 | 외부 모델 서비스 · LAN 전용 · 포트 포워딩 금지 문구 유지 |
| 2 준비 | 1 시작하기 | OS · 설치(macOS `xattr`) · 방화벽 · Claude Code 버전 유지 |
| 3 실행 · 열기 · 종료 | 1 시작하기 | 트레이 메뉴 셋 · 자동 실행 · 자동 업데이트는 결과만, 재시도 간격 · 재시작 규칙은 삭제(개발자 가이드 3.1 · 2.4 에 있음) |
| 4 화면 둘러보기(4.1 ~ 4.3) | 1 시작하기의 절 | 레일 · 타일 · 그룹 · 자리 옮기기 |
| 5 PC 연결 | 2 PC 연결 | |
| 6 내 에이전트 등록 | 3 에이전트 등록 | "셸 명령 실행 허용" 위험 경고 유지 |
| 7 대화하기(7.1 ~ 7.8) | 4 대화 | 질의 · 요청 구분, 컨텍스트, 메시지 작업, 공유, 기록 삭제 |
| 8 @멘션 | 4 대화의 절 | |
| 9 요청 목록 | 5 요청 | |
| 10 자동 왕복 | 6 자동 왕복 | |
| 11 집중 · 접속 상태 | 7 알림의 절 | |
| 12 알림 | 7 알림 | |
| 13 실행 설정 | 8 설정 | 날짜 표시방식 등 설정 화면 전체 |
| 14 AI 에이전트용 CLI(14.1) | 9 CLI | |
| 15 내 데이터 | 10 문제 해결의 절 | 데이터 경로 · 지우기만, 파일 구조는 개발자 가이드 |
| 16 문제 해결 | 10 문제 해결 | |
| 17 알아 둘 점 | 10 문제 해결의 절 | |


1. [user-guide.md](../../manual/user/user-guide.md) 다시 쓰기(결정 3 의 장 순서). 장마다 "무엇" 1 ~ 2줄 + 번호 단계 + 캡처, 화면 글자는 앱 그대로(웹 `index.html` · js literal 과 대조).
   내부 세부는 결과만(결정 4), macOS 차이는 한 줄(결정 6). 목표 길이 320줄 이하.
2. 빼낸 세부 중 개발자 가이드에 없는 것만 [developer-guide](../../manual/dev/developer-guide.md) 제자리로.
3. README — 기능 표 · 본문의 `§n` 을 새 장 번호로, 1.4.4 줄(트레이 `매뉴얼` · 가이드 재작성). [manual/README](../../manual/README.md) 가이드 한 줄 요약을 새 장으로.
4. `PRODUCT.md` #1z-19 줄(트레이 메뉴 · 버전 1.4.4).
5. 트레이 메뉴를 "열기" · "종료" 로 적은 같은 개념 설명 함께(실측 grep): `crates/app/src/tray.rs:1` 머리 주석 · [an_app](../../manual/dev/an_app.md) 168행 ·
   [developer-guide](../../manual/dev/developer-guide.md) 193행 표 · [web-ui](../../manual/dev/web-ui.md) 229행. masterPlan S4 는 종결 환류에서.

## 평가 항목

- [ ] E1: A1 — 트레이 `매뉴얼` 상수 · 메뉴 literal(열기 · 매뉴얼 · 종료)
  CHECK: cargo test -p coworkagent-app --lib tray && cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A1 — 메뉴가 여는 주소가 실재(HTTP 200)
  CHECK: python -c "import re,urllib.request as u;s=open('crates/app/src/tray.rs',encoding='utf-8').read();m=re.search(r'USER_GUIDE: &str =\s*.(https://github[.]com/[^ ;]+user-guide[.]md)',s);print('url=%d status=%d'%(m is not None,u.urlopen(m.group(1),timeout=20).status if m else 0))"
  EXPECT: url=1 status=200
  EVIDENCE: pending
- [x] E3: A1 — 실제 앱(Windows 릴리스 빌드 또는 디버그 빌드) 트레이 메뉴 순서 열기 · 매뉴얼 · 종료, `매뉴얼` 클릭 → 기본 브라우저에 GitHub 가이드, 확인할 일 표시 그대로, 오류 모드 메뉴 변화 없음
  EVIDENCE: Claude(구현자) 2026-11-03 11:25 — 메뉴 글자 개정(`매뉴얼`) 뒤 다시: Windows 11, 릴리스 빌드 1.4.4(`python tools/rebuild_run.py --yes` rc 0, `target/release/coworkAgent.exe` FileVersion 1.4.4, 사용자 데이터 루트). 알림 영역 아이콘을 UI Automation 으로 찾아 우클릭 → 메뉴 "열기 · 매뉴얼 · 종료"(화면 캡처 `docs/images/tray-menu.png` 다시 찍음), "매뉴얼" 클릭 → 기본 브라우저(ego) 창 "coworkAgent/docs/manual/user/user-guide.md at main · Softcamp-SaaS-RND/coworkAgent", 앱 로그 "매뉴얼 열기 실패" 0줄. 확인할 일 = 메뉴 길이 `on_seen` 을 부르지 않음(`tray.rs` guide_id 분기), 오류 모드 `run_error` diff 없음. 처음 확인(디버그 빌드 · 글자 `사용자 매뉴얼`) — 근거 [work/stage2](work/stage2.md).
- [ ] E4: 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E5: clippy 경고 0
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E6: 웹UI 로직 시험(회귀 — 웹 원본 무변경)
  CHECK: node --test --test-reporter=tap "crates/app/web/*.test.mjs"
  EXPECT: /# fail 0/
  EVIDENCE: pending
- [ ] E7: 버전 1.4.4(크레이트 넷)
  CHECK: python -c "import pathlib as P;v=[P.Path('crates/%s/Cargo.toml'%c).read_text(encoding='utf-8') for c in ('core','net','agent','app')];print('crates=%d v144=%d'%(len(v),sum(t.split('version = ',1)[1][1:6]=='1.4.4' for t in v)))"
  EXPECT: crates=4 v144=4
  EVIDENCE: pending
- [ ] E8: A2·A3 — 가이드 320줄 이하 · 캡처 11장(웹 10 + 트레이 1) 참조 · 참조한 그림 전부 실재 · 보안 · 반출 안내 literal 7개(외부 모델 서비스 · 사내망 · 포트 포워딩 · 셸 명령 실행 허용 · 방화벽 · macOS 격리 해제 · 읽기 전용) 유지
  CHECK: python -c "import re,pathlib as P;f=P.Path('docs/manual/user/user-guide.md');t=f.read_text(encoding='utf-8');n=sorted(set(re.findall(r'\]\((\.\./\.\./images/[^)]+\.png)\)',t)));need=['home','group','pairing','agents','new-conv','workspace','requests','relay','notify','settings','tray-menu'];keys=['Claude 모델 서비스','사내망','포트 포워딩','셸 명령 실행 허용','방화벽','xattr -d com.apple.quarantine','읽기 전용'];print('lines_ok=%d refs=%d exist=%d need=%d keys=%d present=%d'%(len(t.splitlines())<=320,len(n),sum((f.parent/x).exists() for x in n),sum(('../../images/%s.png'%k) in n for k in need),len(keys),sum(k in t for k in keys)))"
  EXPECT: /lines_ok=1 refs=(1[1-9]|[2-9][0-9]) exist=\1 need=11 keys=7 present=7/
  EVIDENCE: pending
- [x] E9: A3 — 캡처 스크립트(mock) 오류 0 · 11장이 v1.4.4 화면 · 가상 데이터만(실 사용자 · PC 이름 · 경로 없음) — 그림을 눈으로 대조
  EVIDENCE: Claude(구현자) 2026-11-03 09:00 — mock(`design/mock.py`, 라이트) + ego-browser `design/shots-ego.mjs` 1440×900, 마지막 실행 mock stderr "404" 0줄 · `#disconnected` 없음. 10장(home · group · pairing · agents · new-conv · workspace · requests · relay · notify · settings) 머리 "coworkAgent v1.4.4", 인물 = 박지훈(jhpark · PARK-PC) · 이준호 · 최유나 · build(가상), 경로 `D:\work\shop-api`(가상). 확장 프로그램 오버레이(#ds365-convert-panel-host · #crx-mouse-redesign-content-root) 숨김 뒤 캡처. tray-menu.png = 메뉴 상자만 잘라 냄(사용자 이름 · PC 이름 · 다른 앱 없음). 평가자 눈 대조 대기 — 근거 [work/stage2](work/stage2.md).
- [x] E10: A2 — 가이드의 화면 글자(단추 · 메뉴 · 설정 이름)가 앱 원본 literal 과 같다 · 장마다 번호 단계 + 캡처 — 평가자 대조
  EVIDENCE: Claude(구현자) 2026-11-03 — 가이드에 쓴 화면 글자 78개(예: "새로 만들기" · "초대 복사" · "내 응답 에이전트" · "최대 응답 횟수" · "archive 항목 보기" · "기록 삭제" · "내 글만 삭제" · "진단 정보 복사" · "날짜 표시방식" · "매뉴얼" · 실패 문구 3)를 `crates/**/*.{html,js,rs}`(시험 제외)에서 찾아 없는 것 0. 장 1 ~ 9 마다 번호 단계, 캡처 11장(1.3 tray-menu · 1.4 home · 1.5 group · 2 pairing · 3 agents · 4.1 new-conv · 4.2 workspace · 5 requests · 6 relay · 7 notify · 8 settings). 평가자 대조 대기 — 근거 [work/stage3](work/stage3.md).
- [ ] E11: A4 — 문서 규약 · README · 코드 정리 · 크레이트 경계 · README 의 가이드 장 번호(`§n`)가 새 가이드 heading 에 전부 있음 · 1.4.4 줄
  CHECK: python tools/check_ddd_docs.py && python tools/check_readme.py && python tools/check_code_tidy.py && python tools/check_crates.py && python -c "import re,pathlib as P;r=P.Path('README.md').read_text(encoding='utf-8');g=P.Path('docs/manual/user/user-guide.md').read_text(encoding='utf-8');refs=set(re.findall(r'§(\d+(?:\.\d+)?)',r));heads=set(re.findall(r'^#{2,4} (\d+(?:\.\d+)?)\.? ',g,re.M));print('readme144=%d sec_refs=%d missing=%d'%(r.count('1.4.4'),len(refs),len(refs-heads)))"
  EXPECT: /readme144=[1-9][0-9]* sec_refs=[1-9][0-9]* missing=0/
  EVIDENCE: pending
