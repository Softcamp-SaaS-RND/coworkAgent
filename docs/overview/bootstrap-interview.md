# 부트스트랩 인터뷰 — 프로젝트 수준 의도 확정

> sc-workflow-ddd 부트스트랩의 인터뷰 기록. 루트 `PROJECT.md`·`PRODUCT.md` 와
> [masterPlan.md](../masterPlan.md) 의 `인터뷰 Qn` TODO 가 여기 번호를 가리킨다. 라운드마다 답을 이 문서에
> 덧붙이고, 결정은 masterPlan `## 확정 방향`(D) · `## 기능 명세`(S) 와 루트 문서로 옮긴다.
> 사용자의 "공유 이해" 확인 → masterPlan `승인: {날짜}` 로 끝난다.

- **입력**: 의도 문서 `%APPDATA%\orca\floating-workspace\aitray.md`(2026-10-11 판).
- **조사한 사실 — 환경**: GitHub 리포 `tskimS/agentNetwork` 는 private(비인증 API 404) · 툴체인 `rustc 1.95.0`
  `x86_64-pc-windows-msvc` · `python` 3.12.10 · 설치된 cargo 확장은 clippy·fmt 뿐(`cargo-mutants` 없음).
- **조사한 사실 — Claude Code 연동 지점**(2026-10-11, 공식 문서 기준):
  - PC 간 세션 메시징 — `SendMessage` 는 다른 머신 세션에도 닿지만 Anthropic 서버 경유 Remote Control 이고,
    claude.ai 로그인 · **같은 계정의 내 세션**만 대상이다. 계정 간 · LAN 직접 경로 · 제3자 프로그램을 에이전트로
    등록하는 공개 프로토콜은 문서 없음. 세션 소켓 와이어 포맷도 문서 없음
    ([cross-session-messaging](https://code.claude.com/docs/en/cross-session-messaging)).
  - 세션 푸시(Channels, research preview) — MCP 서버가 `claude/channel` capability 를 선언하고 알림으로 실행 중
    세션에 메시지를 넣는다. 대화형 세션 전용 · claude.ai 또는 Console API key 만 · 자체 서버는 개발용 플래그
    또는 조직 관리 설정(Team/Enterprise)의 허용 목록이 필요 ([channels](https://code.claude.com/docs/en/channels)).
  - MCP stdio 서버 — `claude mcp add <name> -- <command> [args]`. stdio 도구 호출의 idle 기본 상한 30분이라
    수신 대기(long-poll) 도구가 가능 ([mcp](https://code.claude.com/docs/en/mcp)).
  - 훅 — `UserPromptSubmit`·`SessionStart` 가 `additionalContext` 로 텍스트를 넣을 수 있다(사람이 입력할 때만 반영)
    ([hooks](https://code.claude.com/docs/en/hooks)).

## 1라운드 — 2026-10-11 (답 수신 — 아래 "1라운드 답")

답 형식: 전부 권장이면 `권장`, 예외만 `Qn=b`.

### 제품

❓ **Q1** - **존재 이유·통신 주체**: 목표는 "Claude 간 통신", 기능 목록은 슬랙형 사람 메신저(내 정보 · 사용자 목록 ·
대화 UI). 그리고 **같은 계정의 내 세션끼리는 Claude Code 기본 기능으로 이미 PC 를 넘는다**(위 사실). agentNetwork 가
메우는 빈칸과 오가는 주체는?
(a) 사람↔사람 메신저(동료 PC 간) — Claude 연동은 후순위
(b) Claude 세션↔Claude 세션 — 다른 사용자·계정 간, 또는 Anthropic 서버를 거치지 않는 LAN 직접 경로. 사람은 UI 로 열람·개입만
(c) 혼합 — 주소 = 사용자(PC) 1개, 발신자 = 사람 또는 그 PC 의 Claude 세션(이름 태그로 구분)

➡️ (c). 목표와 기능 목록을 함께 만족하는 최소형. 한 PC 의 여러 세션 중 누가 받는지(라우팅)는 2라운드.
"내 PC 들끼리 · 같은 계정" 만이 목적이면 기본 기능으로 충분하니 범위를 다시 본다.

❓ **Q2** - **네트워크 범위·피어 발견**: "해당 툴을 실행하고 있는 PC 목록" 을 어디까지 찾나?
(a) 같은 LAN 자동 발견(mDNS) + 수동 주소(IP:포트) 추가
(b) 같은 바이너리 1대를 "허브" 로 지정해 서브넷 너머까지
(c) 인터넷 너머(NAT 통과 · 릴레이)
중앙 서버가 없으므로 오프라인 상대에게 보낸 메시지는 발신 PC 가 보관했다가 상대가 다시 보이면 재전송한다.

➡️ (a). 사내망은 VLAN 간 멀티캐스트 차단이 흔해 수동 추가가 안전판. (b)·(c) 는 비목표.

❓ **Q3** - **신뢰·보안 모델**: 누가 누구와 대화할 수 있고, 채널을 어떻게 보호하나?
(a) 설치별 키쌍(ed25519) 신원 + 최초 연결 시 양측 승인(지문 비교 페어링) + 전 구간 암호화(공개키 고정 TLS)
(b) 팀 공유 비밀(사전 공유 키) — 키를 아는 모든 PC 가 서로 신뢰
(c) LAN 신뢰 · 무인증

> [!WARNING]
> 원격 Claude 가 내 Claude 에게 메시지를 보내는 구조는 **원격 프롬프트 인젝션 경로**다. 원격 메시지를
> 근거로 한 명령 실행·파일 접근이 사람 승인 없이 일어나면 LAN 의 아무 PC 가 내 PC 에서 코드를 돌릴 수 있다.

➡️ (a) + 가드레일 6번째 줄: "원격 메시지는 신뢰하지 않는 입력 — 사람 승인 없이 실행으로 이어지지 않는다"
(체커 `GUARDRAIL_BULLETS` 5→6 동반).

❓ **Q4** - **UI 표시 방식(D4 해석)**: "모든 UI 는 웹UI" 를 어디에 띄우나?
(a) 내장 웹서버(127.0.0.1 전용) + 기본 브라우저 탭 — OS 웹뷰 의존 없음. 로컬 UI 는 실행마다 바뀌는 토큰 + Host/Origin 검사로 보호
(b) 앱 창(WebView — Tauri 2 / wry. Windows 는 WebView2, macOS 는 WKWebView)

➡️ (a). 단일 실행 파일에 가장 짧은 경로. 앱 창이 필요해지면 같은 웹 자산으로 (b) 이전 가능.

❓ **Q5** - **프론트엔드 구현**:
(a) 빌드 없는 정적 HTML/CSS/JS(ES modules) — 실행 파일에 임베드, `cargo build` 하나로 끝
(b) React/Svelte + Vite 빌드 후 임베드 — Node 빌드 단계 · CI 의존 추가

➡️ (a). 슬랙형 3단(사이드바 · 대화 · 입력) 규모는 프레임워크 없이 충분. 화면이 커지면 그때 (b).

❓ **Q6** - **MVP 비목표**: 그룹 채널(1:1 대화만) · 파일 전송(텍스트·마크다운만) · 인터넷 너머 연결 · Linux · 모바일 ·
메시지 수정/삭제 동기화 — 이 목록을 1차 비목표로 둘까?

➡️ 목록 그대로 비목표.

❓ **Q7** - **대상 OS·아키텍처 + mac 빌드 수단**: Windows 10/11 x64 · macOS 12+ arm64·x64(universal 단일 파일),
Windows ARM64 제외. mac 빌드·서명 수단은? (a) 보유한 Mac (b) GitHub Actions macOS runner — private 리포 과금 확인 필요 (c) 아직 없음

➡️ 대상은 제안대로. 빌드 수단은 환경 사실이라 선생님 답이 필요하다.

❓ **Q8** - **자동 업데이트 배포처·서명**: 리포가 private 이라 GitHub Releases 를 익명으로 받을 수 없다.
(a) 사내 HTTPS 서버/공유 위치에 매니페스트 + 실행 파일 게시
(b) 별도 public 릴리스 리포
(c) 배포처는 Phase 4 직전에 결정, 형식만 지금 고정 — HTTPS 매니페스트 + 앱 자체 ed25519 서명 검증 + 실행 파일 교체
코드 서명 인증서(Windows Authenticode · Apple Developer ID) 사용 가능 여부도 답 필요.

➡️ (c). 서명 검증은 배포처와 무관하게 필수 — 변조된 업데이트는 설치된 전 PC 의 원격 코드 실행이다.

❓ **Q9** - **자동 실행 기본값**: (a) 첫 실행 시 기본 ON + 트레이 메뉴 체크 토글(Windows `HKCU\…\Run` · macOS LaunchAgent
— 관리자 권한 불필요) (b) 기본 OFF

➡️ (a). 상주형이라 꺼져 있으면 상대 목록에서 사라진다.

❓ **Q10** - **실행 파라미터 literal**: 의도 문서엔 예시만 있다. 제안:
- `agentNetwork` — 트레이 상주 시작
- `agentNetwork chat [--peer <사용자ID>]` — 대화 UI 로 바로 진입(상대 지정 선택)
- 이미 실행 중이면 새 프로세스는 기존 인스턴스에 요청만 넘기고 종료(단일 인스턴스)

대안: 플래그형 `agentNetwork --open-chat [--peer <사용자ID>]`. 호출하는 "다른 툴" 이 Orca 인지, 넘길 맥락(세션명 ·
작업 폴더)이 있는지도 답 필요.

➡️ 서브커맨드형 — 2라운드의 Claude 연동 서브커맨드와 같은 꼴로 확장된다.

❓ **Q11** - **저장 경로**: (a) Windows `%LOCALAPPDATA%\agentNetwork\` · macOS `~/Library/Application Support/agentNetwork/`
(b) Windows `%APPDATA%\agentNetwork\`(Roaming — Orca 와 같은 쪽)

➡️ (a). Roaming 은 도메인 로밍 프로필에서 다른 PC 로 복사된다 → 신원 키가 두 PC 에 동시 존재(ID 충돌) + SQLite 동기화 손상 위험.

### 프로세스

❓ **Q12** - **스캐폴딩 범위**: (a) 지금은 DDD 문서 골격까지(완료). Cargo 골격은 masterPlan 승인 후 Phase 0 첫 Task 로
(b) 지금 `cargo new` 최소 골격도 생성

➡️ (a). D5(DDD)는 의도 승인 전에 코드를 쓰지 않는다.

❓ **Q13** - **개발 환경 수단**(루트 `PROJECT.md` 개발 환경 · 검증 명령):
- Git 브랜치 — (a) Task 별 `task/NN-이름` → main merge (b) main 직접 커밋
- 의존 그래프 실측 — `cargo tree`(graphify 는 선택)
- 의존 경계(`check_layers.py` 대체) — (a) Cargo workspace 크레이트 분리: 역방향·순환을 컴파일러가 차단, 첫 모듈 Task 에서
  `cargo metadata` 간선 검사 추가 (b) 단일 크레이트 + 리뷰
- 임계값 경계(`check_thresholds.py` 대체) — 경계·±1 단위 테스트. `cargo-mutants` 도입은 첫 임계값 Task 에서 판단
- 검증 — `cargo test --workspace` · `cargo clippy --workspace --all-targets -- -D warnings`
- TDD · 리뷰 — superpowers 3종(현 기록안)

➡️ 브랜치 (a) — 적대적 평가의 `--scope branch` 와 맞물린다 · 의존 경계 (a) · 나머지 제안 그대로.

### 1라운드 답 — 2026-10-11

사용자 원문:

> Q1. 다른 사용자 및 PC 간 에이젼트 대화 용도임.
> Q8. 개발 완료 후 서명 추가 검토.

- 안내한 형식("예외만 `Qn`")에 따라 **Q1 · Q8 외 11건은 권장안 수용으로 기록**한다 — 사용자가 `권장` 을 따로 적지
  않았으므로 잠정이며, masterPlan `승인:` 이 최종 관문이다.
- 권장안이 없는 사실 질문(Q7 mac 빌드 수단 · Q10 호출 툴·맥락)은 미답 → 2라운드에서 같은 번호로 다시 묻는다.
- Q1 은 (b) 에 해당한다. 단 (b) 보기에 붙였던 "사람은 열람·개입만" 은 결정으로 옮기지 않았다 — Q16 · Q17.
- Q8 은 배포처 (c) 수용 + 서명은 개발 완료 후 검토. 업데이트 파일 서명 검증까지 미루는지는 Q8 확인.

| Q | 기록한 결정 | 반영 위치 |
|---|---|---|
| Q1 | 다른 사용자·PC 간 에이전트 대화 | masterPlan D6 · 문제 · S1 |
| Q2 | (a) LAN mDNS + 수동 IP:포트, 발신 PC 보관 후 재전송 | D7 · 비목표 |
| Q3 | (a) + 가드레일 6행 | D8 · 루트 `PROJECT.md` 가드레일 |
| Q4 | (a) 127.0.0.1 웹서버 + 기본 브라우저 | D9 |
| Q5 | (a) 빌드 없는 정적 자산 임베드 | D10 |
| Q6 | 목록 전부 비목표 | 비목표 |
| Q7 | 대상 OS 수용 · mac 빌드 수단 미답 | D11 · 2라운드 |
| Q8 | (c) + "개발 완료 후 서명 추가 검토" | D12 · 2라운드 Q8 확인 |
| Q9 | (a) 기본 ON + 트레이 토글 | D13 |
| Q10 | 서브커맨드형 literal · 호출 툴 미답 | D14 · 2라운드 |
| Q11 | (a) `%LOCALAPPDATA%` / Application Support | D15 |
| Q12 | (a) Cargo 골격은 승인 후 Phase 0 첫 Task | 루트 `STATE.md` |
| Q13 | 브랜치 (a) · 의존 경계 (a) · 나머지 제안대로 | 루트 `PROJECT.md` 개발 환경 · 검증 명령 |

## 2라운드 — 2026-10-11 (답 수신 — 아래 "2라운드 답")

답 형식: 전부 권장이면 `권장`, 예외만 `Qn=b`. 사실 질문(Q7 · Q10 · Q15 의 계정 형태)은 값으로.

❓ **Q14** - **에이전트 주소 단위**: D6 으로 대화 주체는 에이전트다.
(a) 사용자(PC) 당 에이전트 1개 — 그 PC 에서 수신 대기 중인 세션 하나가 사용자 명의로 받는다
(b) 세션 단위 — 한 PC 에 에이전트 여럿, `사용자/에이전트명` 으로 주소. 이름은 세션 등록 시 지정(기본: 작업 폴더명).
연결 목록은 사용자 아래 에이전트 트리
(b) 는 세션마다 MCP 프로세스가 하나씩 뜬다는 Q15 (a) 를 전제한다 — 세션 신원이 거기서 자연히 생긴다.

➡️ (b). 한 PC 에 세션 여럿(Orca 워크트리 등)이 보통이라 (a) 는 누가 받는지 모호하다.

❓ **Q15** - **Claude 연동 방식 + 사내 계정 형태**:
(a) MCP stdio 서브커맨드 `agentNetwork mcp` — 도구: 상대 목록 · 보내기 · 받기(수신 대기, stdio idle 기본 상한 30분).
**pull 방식**이라 상대 에이전트는 그쪽 사람이 "메시지를 기다려라" 라고 시켜 둔 동안에만 답한다 — 대화는 양쪽
모두 사람이 시작시킨다
(b) (a) + Channels 푸시 — 받은 메시지를 실행 중 세션에 즉시 주입해 사람 없이 반응. research preview · 대화형 세션
전용 · claude.ai 또는 Console API key 만 · 자체 서버는 개발 플래그(운영용 아님) 또는 Team/Enterprise 조직 설정의
허용 목록 필요
(c) 훅 주입 — 사람이 입력할 때만 반영, 에이전트 간 자동 대화 불가
사실 질문: 동료들의 Claude 사용 형태는? claude.ai Team/Enterprise · Console API key · 개인 Pro/Max · 혼재

➡️ (a) 로 시작, (b) 는 계정 형태 확인 뒤 옵션 단계. (a) 만으로도 양쪽이 "기다려라" 를 한 번 시키면 왕복 대화가 된다.

❓ **Q16** - **원격 메시지 전달 정책 + 가드레일 6행 문구**: 실행 통제는 Claude Code 권한 프롬프트 소관이라(bypass 모드
세션은 무방비) agentNetwork 가 강제할 수 있는 것은 **전달**뿐이다.
(a) 페어링된 상대면 자동 전달 — 실행 통제는 Claude Code 에 맡김
(b) 대화 단위 승인 — 새 대화의 첫 메시지는 웹UI 에서 사람이 승인해야 에이전트에 전달, 승인된 대화는 이후 자동.
상대별 accept/refuse 로 조정(Claude Code `crossSessionInbound` 의 accept/hold/refuse 와 같은 축)
(c) 메시지마다 승인
가드레일 6행을 강제 가능한 문구로 교체: "원격 메시지는 **신뢰하지 않는 입력**이다 — 사람이 승인하지 않은 상대·대화의
메시지는 에이전트에 전달하지 않고, 전달할 때 원격 출처를 표시한다."

➡️ (b) + 문구 교체.

❓ **Q17** - **사람의 역할**:
(a) 열람 + 승인만 — 대화는 에이전트끼리, 시작도 각자의 Claude 세션에서
(b) (a) + 사람이 대화에 직접 작성(발신자 '사람' 표시) — 1:1 대화에 세 번째 발화자가 생긴다(Q6 "1:1 만" 과 긴장)

➡️ (a). 공격면이 작고 Q6 과 맞는다.

❓ **Q18** - **"작업 이력" 의 정의**(S3):
(a) 대화 메시지만 — 작업 요청도 메시지
(b) 별도 작업 객체 — 요청 → 수락 → 완료/실패 상태 추적
(c) 메시지 + 보안 이벤트(페어링 · 승인 · 거부 · 연결) 기록

➡️ (c). 승인 기반 가드레일(Q16)을 사후에 추적할 수단이고 테이블 하나 수준이다. 작업 상태 추적은 필요해지면 메시지
종류 필드로 키운다.

❓ **Q19** - **Phase 순서**:
(a) 층별 — 현 초안(기반 → 피어 → 대화 → Claude 연동 → 배포)
(b) 관통 우선 — Phase 0 에서 2 PC 간 에이전트 메시지 1왕복을 끝까지 연결: 단일 실행 파일 · 트레이 · 로컬 웹UI ·
SQLite · 신원 + **페어링 · 공개키 고정 TLS · 전달 승인(Q16)** · MCP 송수신. 이후 Phase 가 각 층(발견 자동화 · 슬랙형
UI · 자동 실행/업데이트)을 키운다

➡️ (b). 핵심 가치와 위험(LAN 차단 · Claude 연동 제약)을 먼저 드러낸다. 보안은 첫 메시지부터 넣는다 — 평문 골격을
먼저 만들면 D8 · 가드레일과 어긋나는 계획 drift 다.

❓ **Q7 (재질문)** - **mac 빌드 수단**: (a) 보유한 Mac (b) GitHub Actions macOS runner(private 리포 과금 확인 필요)
(c) 아직 없음. 서명은 Q8 대로 개발 완료 후.

➡️ 권장안 없음 — 환경 사실.

❓ **Q8 (확인)** - **"서명" 의 범위**:

> [!WARNING]
> 업데이트 파일 서명 검증 없이 자동 업데이트를 켜면, 업데이트 배포처를 장악한 누구나 설치된 모든 PC 에 코드를
> 밀어 넣을 수 있다.

(a) 코드 서명 인증서(Authenticode · Developer ID — OS 경고 제거)만 개발 완료 후 검토. 업데이트 파일 서명 검증(앱 자체
ed25519)은 S6 구현에 포함
(b) 둘 다 개발 완료 후 — 그때까지 S6(자동 업데이트)는 꺼진 채 배포(수동 교체). 명세 기능이 비활성으로 나가는 것을
수용해야 한다
(c) 둘 다 개발 완료 후, 자동 업데이트는 서명 없이 먼저 — 권장하지 않음

➡️ (a). 앱 자체 서명 검증은 키 한 쌍 + 검증 몇 줄이라 인증서 구매·절차와 무관하다.

❓ **Q10 (재질문)** - **호출 툴 · 맥락**: 서브메뉴를 붙이는 "다른 툴" 이 Orca 인가? 서브메뉴가 세션/워크트리 단위라면
"내 에이전트" 를 지정하는 인자가 필요하다 — 제안 `agentNetwork chat [--agent <이름>] [--peer <사용자ID>]`(D14 literal
변경이라 승인 필요, Q14 (b) 와 짝).

➡️ 호출 툴이 Orca 면 `--agent` 추가. 아니면 툴 이름과 넘길 수 있는 값을 알려 달라.

### 2라운드 답 — 2026-10-11

사용자 원문:

> Q15. claude -p 의 해드리스 명령어로 연동.
> Q16. 자동으로 동작.
> Q17. b 사람은 직접 대화를 작성. Q6답변 번복.
> Q18. 요청/응답에 대한 전체 이력. 해당 이력은 채팅 이력 모양으로 보여주며, 나중에 해당 이력에 이어서 작업 가능해야해함.
> Q7. 보유한 mac 에서 빌드.

- Q15 · Q16 · Q18 은 보기 밖의 답이라 **원문을 결정으로** 옮겼다.
- 무응답 4건:
  - Q14 — 전제(세션마다 MCP 프로세스)가 Q15 답으로 사라졌다 → 권장 수용으로 보지 않고 Q20 으로 다시 묻는다.
  - Q19 — 권장 (b) 관통 우선 수용(잠정). 골격의 Claude 연동은 MCP 에서 `claude -p` 로 바꿨다.
  - Q8 확인 — 권장 (a) 수용(잠정): 코드 서명 인증서만 개발 완료 후, 업데이트 파일 서명 검증은 S6 에 포함.
  - Q10 — 두 번 미답 → S8 Task 착수 인터뷰로 이월(D14 literal 은 유지).
- **충돌 3건**(3라운드로 넘김):
  1. Q16 "자동으로 동작" ↔ 가드레일 6행(Q3 수용) "사람 승인 없이 실행으로 이어지지 않는다" → Q21.
  2. Q17 "Q6답변 번복" 의 범위 — "1:1 대화만" 한 항목인가 Q6 전체인가 → Q22. 우선 "1:1 대화만" 만 비목표에서 뺐다.
  3. Q15 `claude -p` → Q14 (b) 의 전제 소멸 → Q20.

| Q | 기록한 결정 | 반영 위치 |
|---|---|---|
| Q15 | `claude -p` 헤드리스 실행 | masterPlan D16 · S1 · 루트 `PRODUCT.md` 경계 |
| Q16 | 자동 처리 | D17 · 3라운드 Q21 |
| Q17 | 사람 직접 작성 + "1:1 대화만" 철회 | D18 · 비목표 · 3라운드 Q22 |
| Q18 | 요청/응답 전체 · 채팅 모양 · 이어서 작업(resume) | D19 · S3 · 루트 `PRODUCT.md` 저장 |
| Q7 | 보유한 Mac 에서 빌드 | D11 |
| Q8 확인 | (a) 잠정 | D12 |
| Q19 | (b) 잠정 | Phase 로드맵 |
| Q10 | S8 Task 로 이월 | D14 TODO |
| Q14 | Q20 으로 대체 | — |

- **조사한 사실 — `claude -p`**(로컬 Claude Code 2.1.296 `--help` 실측): `--output-format json|stream-json` ·
  `--resume <id>` · `--session-id <uuid>` · `--fork-session` · `--no-session-persistence` · `--permission-mode`(acceptEdits ·
  auto · bypassPermissions · manual · dontAsk · plan) · `--permission-prompts host|none` · `--allowedTools` ·
  `--disallowedTools` · `--tools` · `--restricted` · `--max-budget-usd` · `--mcp-config` · `--strict-mcp-config` ·
  `--append-system-prompt` · `--bare`. `-p` 는 작업 폴더 신뢰 확인을 건너뛴다("Only use this in directories you trust").
  Windows 네이티브 설치는 `~/.local/bin/claude.exe`.
- **조사한 사실 — `claude -p` 의미**(공식 문서, 2026-10-11 — 3라운드 송부 뒤 도착해 보충으로 반영):
  - 결과 JSON: `session_id` · `result` · `is_error` · `subtype` · `num_turns` · `total_cost_usd` · `permission_denials` 등. 오류
    subtype 은 `error_max_turns` · `error_max_budget_usd` · `error_during_execution` 이고 이때 `result` 가 없다. 실패 시 종료
    코드 non-zero ([agent-sdk/typescript](https://code.claude.com/docs/en/agent-sdk/typescript#sdkresultmessage)).
  - resume: ID 재개는 cwd 와 무관하지만 재개 시 도구 작업 폴더는 문서 없음 → 폴더를 고정한다. `-p` 세션도 대화형
    `claude --resume <id>` 로 이어진다. 같은 세션 동시 재개는 "messages from both interleave" — 잠금 없음. 재개 때
    `--mcp-config` · `--settings` · `--add-dir` 와 권한 모드를 매번 다시 준다 ([sessions](https://code.claude.com/docs/en/sessions)).
  - 권한: 승인이 필요한 호출은 응답할 호스트가 없으면 거부, `--permission-prompts none` 은 대기 없이 거부. 일부 환경
    (서드파티 provider · telemetry off)에선 `-p` 시작 모드가 `auto`. `--permission-prompt-tool` 은 입출력 형식이 문서에 없고
    응답까지 무기한 대기 — 문서화된 대안은 `PermissionRequest` 훅
    ([headless](https://code.claude.com/docs/en/headless) · [permission-modes](https://code.claude.com/docs/en/permission-modes)).
  - 신뢰: `-p` 는 신뢰 확인 없이 그 폴더의 `.claude/settings.json` 훅 · `env` · `apiKeyHelper` 와 `.mcp.json` 서버를 실행한다.
    `--restricted` 는 사용자·프로젝트·로컬 설정을 읽지 않고, `--strict-mcp-config` 를 더하면 MCP 서버도 건너뛴다
    ([permissions](https://code.claude.com/docs/en/permissions#what-runs-before-you-trust-a-folder)).
  - 한도: `--max-turns`(`--help` 에 없지만 지원) · `--max-budget-usd`(추정 비용 기준)
    ([cli-reference](https://code.claude.com/docs/en/cli-reference#cli-flags)).
  - 인증: `ANTHROPIC_API_KEY` 가 있으면 `-p` 는 그 키를 쓴다. `--bare` 는 구독 OAuth 를 읽지 않으며 문서에 "will become the
    default for `-p` in a future release" ([authentication](https://code.claude.com/docs/en/authentication) ·
    [headless](https://code.claude.com/docs/en/headless#start-faster-with-bare-mode)).
  - 정책(원문): "Anthropic does not permit third-party developers to offer Claude.ai login into their own applications, or to
    route requests through Free, Pro, or Max plan credentials on behalf of their users. Moreover, developers may not collect,
    store, or intermediate Claude.ai credentials or session tokens" · "Advertised usage limits for Pro and Max plans assume
    ordinary, individual usage of Claude Code and the Agent SDK."
    ([legal-and-compliance](https://code.claude.com/docs/en/legal-and-compliance))
  - Windows: Bash 도구는 Git Bash 필요(없으면 PowerShell 도구) · 비셸 부모 프로세스의 PATH 해석은 문서 없음 → 전체 경로를
    지정 · `-p` 는 남은 백그라운드 작업을 idle 10분까지 기다린다 ([setup](https://code.claude.com/docs/en/setup)).

## 3라운드 — 2026-10-11 (답 수신 — 아래 "3라운드 답(부분)" · "종료")

답 형식: 전부 권장이면 `권장`, 예외만 `Qn=b`.

❓ **Q20** - **에이전트의 정의(Q14 대체)**: D16 으로 에이전트 = 수신 PC 에서 도는 `claude -p` 실행이다. `-p` 는 작업 폴더
신뢰 확인을 건너뛰고, 실행 폴더가 곧 맥락(CLAUDE.md · 설정 · 세션 저장 위치)이자 **에이전트가 읽어 답으로 내보낼 수
있는 범위**다. 또 `-p` 는 그 폴더의 훅 · `env` · `.mcp.json` 을 신뢰 확인 없이 실행한다(Q21 플래그로 차단).
(a) 프로필 등록제 — 수신 PC 사용자가 웹UI 에서 에이전트를 등록(이름 · 작업 폴더 · 허용 도구 · 추가 지시문). 원격은
`사용자/에이전트명` 으로 등록된 것만 부른다
(b) PC 당 기본 에이전트 1개 — 설정한 작업 폴더 하나
(c) 원격이 작업 폴더를 지정 — 원격이 내 PC 의 아무 폴더나 대상으로 삼는다. 권장하지 않음

➡️ (a). 어느 폴더를 어떤 권한으로 열지는 그 PC 사람이 정해야 한다. 처음엔 1개만 등록해 써도 같은 구조다.

❓ **Q21** - **헤드리스 실행 권한(가드레일 6행 충돌 해소)**: Q16 "자동으로 동작" 과 가드레일 6행 "사람 승인 없이
실행으로 이어지지 않는다"(Q3 수용)가 부딪힌다. 수단(2.1.296 `--help` 실측): `--allowedTools` · `--disallowedTools` ·
`--tools` · `--restricted`(명령·코드 실행 도구와 WebFetch 제외, 사용자·프로젝트 설정 무시, 파일 도구를 작업 폴더로 한정,
bypassPermissions 거부) · `--permission-prompts none`(물어야 할 동작은 자동 거부) · `PermissionRequest` 훅(`-p` 에서도
결정권, `--settings` 로 주입). `--permission-prompt-tool` 은 형식 미문서화 · 무기한 대기라 쓰지 않는다.
(a) 에이전트별 허용 도구만 자동 실행, 그 밖은 즉시 거부 — 사람 개입 0. 6행 → "원격 메시지는 **신뢰하지 않는
입력**이다 — 에이전트에 허용하지 않은 동작으로 이어지지 않는다."
(b) (a) + 허용 밖 동작은 수신 PC 사람에게 웹UI 승인 요청, 무응답이면 거부 — 6행 유지. 수단은 `PermissionRequest` 훅,
대기 상한은 agentNetwork 가 건다
(c) 전부 자동(`bypassPermissions`) — 원격 피어가 내 PC 에서 임의 명령을 돌릴 수 있다 = D8 · 6행 철회. 권장하지 않음

➡️ (a). 기본 플래그 `--restricted --strict-mcp-config --permission-prompts none` + 권한 모드 명시(일부 환경의 `auto` 시작
방지) + 에이전트별 `--tools`. 폴더의 훅 · MCP 가 신뢰 확인 없이 도는 것도 이것으로 막는다. "자동" 과 가드레일을 함께 지킨다.

❓ **Q22** - **대화 구성(Q6 번복 범위)**:
(a) "1:1 대화만" 한 항목만 철회 — 대화 = 두 PC 사이, 발화자 = 양쪽 사람 + 에이전트. 3 PC 이상 다자 채널과 나머지 Q6
비목표는 유지
(b) 다자 채널까지 — 여러 PC 의 사람·에이전트가 한 채널. 중앙 서버가 없어 메시지를 참여 PC 전부에 복제하고 오프라인
PC 의 따라잡기가 필요
(c) Q6 전체 번복 — 파일 전송 · 인터넷 너머 · Linux · 모바일 · 수정/삭제 동기화도 범위 안

➡️ (a). (b) 는 P2P 복제가 붙어 한 Phase 를 넘는다 — 필요해지면 그때.

❓ **Q23** - **에이전트 간 자동 왕복의 시작·종료·한도**: D17 자동 처리라 두 에이전트의 답이 서로를 계속 깨우면 끝없이
돌고, 비용은 각 수신 측 계정이 낸다.
- 시작: (a) 사람이 대화를 열 때 두 에이전트를 지정하고 첫 메시지를 쓴다 — 이후 한쪽 답이 다른 쪽 입력으로 자동 전달
  (b) 내 에이전트가 agentNetwork 가 붙여 준 송신 도구(`--mcp-config`)로 스스로 상대를 부른다
  (c) 없음 — 사람 → 원격 에이전트 대화만 하고, 에이전트 간 자동 왕복은 비목표(아래 종료·한도도 불필요)
- 종료·한도: 대화당 자동 왕복 상한(기본 10, 설정 가능) 도달 시 멈추고 사람 입력 대기 + 상대별 시간당 수신 처리 상한 +
  실행 1회당 `--max-turns` · `--max-budget-usd`

➡️ 시작 (a) — 추가 도구 없이 `-p` 출력만으로 된다. (b) 는 후속. 종료·한도는 제안대로.

❓ **Q24 (보충)** - **Claude 계정·인증(D16 전제)**: 원격 요청을 수신 PC 의 Claude 로그인으로 처리하는 구조라 위 정책 원문에
걸린다 — Free·Pro·Max 자격으로 "route requests ... on behalf of their users" 금지, Pro/Max 사용 한도는 "ordinary, individual
usage" 전제.
(a) 각 PC 는 회사가 발급한 Console API key(`ANTHROPIC_API_KEY`)로 `claude -p` 실행 — 정책상 가장 명확, 키 소유자 과금,
`--bare` 기본화에도 안전
(b) 회사 Team/Enterprise 플랜 로그인 — 금지 문장의 대상(Free · Pro · Max)은 아니지만 "ordinary use" 범위인지 Anthropic 확인 필요
(c) 개인 Pro/Max 로그인 — 정책 위반 소지. 권장하지 않음
어느 경우든 agentNetwork 는 Claude 자격 증명·세션 토큰을 수집·저장·중개하지 않는다(정책 원문 귀결).

➡️ (a). (b) 로 가려면 Anthropic 영업 문의로 사용 사례 확인이 먼저다.

### 3라운드 답(부분) — 2026-10-11

사용자 원문:

> claude max 구독이 설정되어있는 pc 에 해당 프로젝트의 바이너리를 배포할 거야.
> 해당 툴은 목적 문서에 작성한것과 같이 서버/클라이언트/슬랙(UI)를 모두 제공하고 다른 사용자에의 PC 에 요청은 claude -p 로 실행할 거야.

- Q24 → (c) 각 PC 의 Max 구독 로그인. 정책 위험을 고지(보충 질문)한 뒤의 결정 → masterPlan D20.
- 둘째 문장은 D3 · D4/S4 · D16 의 재확인 — 변경 없음.
- Q20~Q23 은 이 답에 없다 → masterPlan 승인 확인 때 함께 묻는다.

## 성공 기준 초안 (3라운드와 함께 제시)

- 프로젝트 수준 성공 기준 — 3라운드가 권장안대로면:
  - 서로 다른 사용자의 PC 2대(Windows · macOS 각 1대 이상)가 같은 LAN 에서 중앙 서버 없이 페어링하고, 한쪽 사람이 쓴
    메시지에 상대 PC 의 등록 에이전트가 `claude -p` 로 답한다.
  - 페어링하지 않은 PC 의 메시지는 처리되지 않고, 에이전트에 허용하지 않은 도구는 원격 메시지로 실행되지 않는다.
  - 대화의 요청/응답 전체가 양쪽 SQLite 에 남고 채팅 모양으로 보이며, 나중에 같은 대화에 이어 쓰면 에이전트가 이전
    맥락으로 답한다.
  - 두 에이전트의 자동 왕복이 상한에서 멈춘다.
  - 배포물은 OS 별 실행 파일 1개 — 설치 없이 실행해 트레이 상주 · 자동 실행 · 서명 검증된 자동 업데이트 ·
    `agentNetwork chat` 진입이 동작한다.
- Phase 0 Task 분할(채팅으로 제안): 00.skeleton · 01.pairing · 02.agent-roundtrip.

## 종료 — 2026-10-11

사용자 원문:

> 공유 이해했고 다음 단계 진행.

- 프런티어 종료 → masterPlan `승인: 2026-10-11`.
- Q20~Q23 은 답이 없었고 "공유 이해" 확인으로 권장안 수용 → masterPlan D21~D24. 가드레일 6행은 Q21 (a) 문구로 교체.
- 성공 기준은 위 초안 그대로 → masterPlan SC1~SC5. Phase 0 은 제안한 3분할.
- 남은 열린 것: D14 의 호출 툴 · `--peer` 값 형식 → S8 Task 착수 때.
