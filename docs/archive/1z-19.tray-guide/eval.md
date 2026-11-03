# eval.md — 1z-19.tray-guide 적대적 평가

## 평가 원장

<!-- tools/check_gates.py 산출 — 손으로 고치지 않는다. 단 CHECK 가 없는 수동 gate 의
     EVIDENCE 와 ABANDON 줄은 사람이 적고, 재실행해도 보존된다.
     WEAK 줄은 기계가 매 실행 다시 단다 — 약한 gate 를 셀 수 있게 하는 표시이며
     판정에는 영향이 없다. -->

측정 커밋: d3c14568cdcaa8a8aeb0ad40c24cfc804d883313

- [x] E1: A1 — 트레이 `매뉴얼` 상수 · 메뉴 literal(열기 · 매뉴얼 · 종료)
  CHECK: cargo test -p coworkagent-app --lib tray && cargo test -p coworkagent-app --test ux_literals
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.19s

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
  EVIDENCE: Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.26s

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

## 측정 기록

- 첫 측정 `f30f75f`(미커밋) E11 만 미충족(측정 커밋 뒤 코드 변경 — 문서 규약 체커) → 커밋 `42d323b` 뒤 첫 실행 E11 만 미충족(원장의 옛 측정 커밋), 둘째 실행 11/11 PASS.
- 메뉴 글자 개정(`사용자 매뉴얼` → `매뉴얼` — intent 결정 1 라운드 2)은 `42d323b` 에 포함 — E3 은 릴리스 빌드로 다시 확인.

## 적대적 평가 — 라운드 1/2

평가자: Orca 가 새로 띄운 Codex(`worker-start --agent codex --model gpt-6.1-sol --effort high`, 작업 폴더 `C:/Users/tskim/.claude/skills` — 대상 저장소 밖, `git -C` 로 읽기만).
Run `run_204e8c825ad9` · Task `task_461daf7eda3d` · Dispatch `ctx_978e464674d9` · 터미널 `term_e3e57a32-07f5-44c2-902b-97b557105fb9`.
범위 `f30f75f..d3c1456`(self-collect — main 에 squash push 된 v1.4.4), 메타 `요청 #1z-19.tray-guide · revision d3c14568cdcaa8a8aeb0ad40c24cfc804d883313 · 라운드 1/2`, worker_done `msg_006bfd0e9c87`. 리뷰어 Dispatch `worker-release` → `released`(`closed_agent_terminal`).

```text
# Codex Adversarial Review

Target: branch diff against f30f75f88df682456ecbce7666da585b161ffea4
Verdict: needs-attention

라운드 1/2: 배포 보류. 가이드 재작성으로 삭제 후 잔존 데이터와 원격 삭제 지연 안내가 사라졌다. 트레이 변경·버전·README 장 번호에서는 차단할 결함을 찾지 못했다.

Findings:
- [medium] 초기화 뒤에도 남는 Claude Code 기록 안내를 복원해야 한다 (docs/manual/user/user-guide.md:293-295, confidence 0.94)
  기존 §15의 초기화 안내에는 Claude Code 기록(%USERPROFILE%\.claude\projects\… 또는 CLAUDE_CONFIG_DIR)이 별도로 남는다고 명시했지만, 새 안내는 coworkAgent 폴더 삭제만 제시한다. runner.rs:217-218은 --session-id/--resume을 사용하며, developer-guide.md:511-519도 앱 데이터를 지워도 세션 파일은 남는다고 명시한다. [추론] 민감한 질의·응답을 정리하려는 사용자가 이 절차만 따르면 로컬 대화 사본을 남긴 채 초기화가 끝났다고 오인할 수 있다; 실제 삭제 실험은 하지 않았다.
  Recommendation: 초기화는 앱 데이터만 지우며 Claude Code 세션 기록은 별도로 남는다는 경고와 해당 기록 위치/개발자 가이드 §7.4 링크를 복원한다. 다른 프로젝트 기록을 일괄 삭제하도록 안내하지 않는다.
- [medium] 모든 사용자 글 삭제의 원격 처리 지연 조건을 다시 안내해야 한다 (docs/manual/user/user-guide.md:180-184, confidence 0.94)
  기존 §7.8에 있던 '상대가 꺼져 있으면 이 PC만 먼저 지우고 상대는 켜질 때 삭제' 안내가 없어졌다. chat/lifecycle.rs의 clear_conversation은 상대 연결 실패를 허용한 뒤 로컬 기록을 지우고 Clear를 pending outbox에 넣어 반환하며, conv-menu.js:142-143은 그 시점에 '모든 사용자의 대화 기록을 지웠습니다'를 표시한다. [추론] 오프라인 상대가 있는 상황에서 새 가이드와 완료 문구만 따르면 상대 사본까지 이미 삭제됐다고 믿고 민감한 기록 정리를 끝낼 수 있다; 네트워크 재현은 하지 않았다.
  Recommendation: 이 PC 삭제와 원격 삭제 완료를 구분해 설명한다. 상대 오프라인·실행 중에는 원격 삭제가 지연되고, 실제 원격 반영을 확인하기 전에는 전체 삭제 완료로 간주하지 말라는 주의를 복원한다.

Next steps:
- §10.2의 세션 기록 잔존 경고와 §4.8의 원격 삭제 지연 조건을 복원한다.
- 고정 범위 정적 검토·캡처 11장 대조만 수행했다. 빌드·테스트·Windows/macOS 실기는 재실행하지 않았다.
```

판정 · 조치:
- 지적 1(초기화 뒤 남는 Claude Code 기록 안내 빠짐) — **사실**. 옛 가이드 §15 "초기화" 에 있던 `%USERPROFILE%\.claude\projects\…` · `CLAUDE_CONFIG_DIR` 안내가 재작성 때 빠졌다(결정 4 "결과만 남김" 을
  보안 · 반출 안내까지 넓혀 적용한 실수 — 개발자 가이드 7장에는 있음). 가이드 10.2 에 ⚠ 줄 복원(이 PC 의 에이전트 기록 · 작업 폴더 해당 기록만 · 다른 프로젝트 기록은 건드리지 않음).
- 지적 2("모든 사용자 글 삭제" 원격 지연 안내 빠짐) — **사실**. 옛 §7.8 "상대가 꺼져 있으면 이 PC 는 바로 지우고, 상대 PC 는 켜질 때" 가 빠졌다. 가이드 4.8 에 복원 + 완료 문구만으로 상대 사본 삭제를 단정하지 말 것.
- 둘 다 문서 수정 — 코드 · 시험 변경 없음. E8(가이드 ≤ 320줄 · 보안 literal 7) 다시 판정.
- 라운드 2 는 돌리지 않음 — 수정이 옛 가이드 문구 복원(원문 대조)뿐이라 1 라운드로 끝냈다(상한 2 는 상한이지 최소가 아니다).

## 미조치 지적

없음
