# PRODUCT.md — coworkAgent

**무엇을 만들었는가**의 단일 위치 — 구조 · 규칙 · 설계. 프로세스·환경은 루트 `PROJECT.md`,
진행 현황은 루트 `STATE.md`. 계약은 [etc/ddd-workflow.md](docs/etc/ddd-workflow.md) "PRODUCT.md — 필수 6절".
**빈 절은 없다** — 실내용이 없으면 `없음`, 미확정이면 `미확정` + TODO 주석.

## 제품 개요

- **무엇**: 트레이에 상주하는 단일 실행 파일이 서버·클라이언트를 겸하여, 다른 사용자·PC 의 Claude 에이전트끼리
  대화하게 하는 도구 — UI 는 슬랙형 웹UI.
- **경계**(결정 번호는 [masterPlan.md](docs/masterPlan.md) `## 확정 방향`):
  - 피어 간 — 같은 LAN 자동 발견(mDNS) + 수동 주소, 설치별 ed25519 신원 · 페어링 · 공개키 고정 TLS(D7 · D8).
  - 로컬 UI — 127.0.0.1 전용 웹서버 + 기본 브라우저(D9).
  - Claude — 메시지를 받은 PC 에서 `claude -p` 를 자식 프로세스로 실행하고 결과를 답으로 보낸다(D16 · D17).
    외부 전제: 각 PC 에 Claude Code 설치·로그인.
  - 다른 툴 → 실행 파라미터(D14).
- 전체 의도·범위·제외는 [masterPlan.md](docs/masterPlan.md) `## 의도`, 기능 명세는 `## 기능 명세`(복제 금지).

## 기술 스택

- **언어 / 런타임**: Rust stable — 검증 환경 `rustc 1.95.0` · `x86_64-pc-windows-msvc`(Windows 11). 대상은
  Windows 10/11 x64 · macOS 12+ universal, mac 빌드는 보유한 Mac(D11).
- **외부 실행 전제**: Claude Code CLI — 검증 환경 `2.1.296`, Windows 네이티브 설치는 `~/.local/bin/claude.exe`.
  자기 업데이트 시 이전 exe 를 `claude.exe.old.*` 로 남긴다(실측). Bash 도구는 Git Bash 필요(없으면 PowerShell 도구),
  비셸 부모 프로세스의 PATH 해석은 문서에 없어 전체 경로로 실행한다.
- **핵심 의존**(Windows 에서 빌드·실행 실측): tray-icon 0.26.1 · tao 0.37.1(트레이 · 이벤트 루프) · axum 0.8.9 · tokio 1.53.2
  (로컬 웹UI) · rusqlite 0.40.2 `bundled`(SQLite 내장) · dirs 7.0.0 · getrandom 0.4.3 · log 0.4.34 · windows-sys 0.61.2(Windows 전용 —
  app 은 `ShellExecuteW` 하나 — 브라우저 · 파일 열기, macOS `open` 과 같은 동작 · an-agent 는 Job Object — 자손 프로세스 종료, 프로세스 그룹과
  같은 동작) · futures-util 0.3.34(SSE 스트림) · rustls 0.23.45(`ring`, TLS 1.2 미포함) · tokio-rustls 0.26.6 · ring 0.17.14(ed25519 ·
  SHA-256) · serde 1.0.229 · serde_json 1.0.152(피어 프레임 · API).
- **빌드**: `cargo build --release --workspace` → Windows `target/release/coworkAgent.exe`(단일 파일). macOS universal 은
  `lipo` 절차(Phase 4, D25) — [archive/00.skeleton/plan.md](docs/archive/00.skeleton/plan.md) "macOS universal".
- **웹UI 테스트**: `node --test` — 화면 로직 모듈(`crates/app/web/view.js`)만, Node 22+ 는 개발 PC 에만(제품 의존 아님).
- **테스트**: `cargo test --workspace` — **경고 baseline**: 없음 <!-- 외부 의존 경고만 여기 고정. 우리 코드 경고는 0 -->

## 모듈 Layer 구조

| Layer | 크레이트 | 담는 것 |
|---|---|---|
| base | `an-core`(`crates/core`) | 데이터 경로 · SQLite · 로그 회전 · 단일 인스턴스 |
| comm | `an-net`(`crates/net`) | 신원 · RPK TLS · 피어 프로토콜 · 페어링 · 초대 |
| domain | `an-agent`(`crates/agent`) | `claude -p` 실행기 · 대화 왕복 조율 · DB 대기열 |
| app | `coworkagent-app`(`crates/app`) | 웹UI 서버 · 내 정보 · 피어 · 에이전트 · 대화 API · 핸드오프 · 트레이 · 브라우저 · 시작 순서 |

<!-- 의존 방향 app → domain → comm → base 단방향. 비-Python 이라 `check_layers.py` 는 두지 않는다 — 이 절의
     표를 검사하는 수단은 루트 PROJECT.md "검증 명령" 의 "의존 경계" 행. -->

**의존 경계 규칙**: 간선은 app → domain → comm → base 방향만(`tools/check_crates.py` 의 `LAYER` 가 정본). base 는 UI · HTTP ·
트레이를 모른다. 새 크레이트는 `LAYER` 와 이 표에 함께 등록한다.

## 도메인 규칙

- **신원**: 설치별 ed25519 키쌍(D8) — `identity.key`(PKCS#8). 비밀키는 사용자 데이터 경로(D15) 밖으로 내보내지 않는다 — 리포·로그·UI
  금지(D8 귀결). 손상 키는 재생성하지 않는다(기존 페어링 보호). 지문 = SHA-256(SPKI) 앞 16바이트.
- **페어링**: 받는 쪽 "페어링 받기" → 초대 텍스트(주소 · 초대 코드 · 지문)를 메신저로 → 보내는 쪽이 붙여 넣으면 지문으로 서버 키를
  고정해 연결 · 추가 · 내 승인 → 받는 쪽 승인 → 페어링. 처음 보는 키는 초대 코드가 맞는 1건만(5분), approve 는 같은 세대만.
  미페어링 상대는 hello · approve · remove 외 거부(SC2).
- **두 OS 같은 UI**(D30): 화면 · 알림은 웹UI 와 tray-icon 으로만 — OS 전용 UI API(WinRT · 메시지 상자 · AppleScript)를 쓰지 않는다.
- **인바운드 알림**(S9): 기본 OFF, 켜면 페어링 요청 · 상대 승인을 웹UI 탭이 열려 있으면 브라우저 알림(SSE → `Notification`, OS 알림
  정책 D27), 탭과 무관하게 트레이 "새 요청" 표시. 웹UI 연결(`POST /api/seen`) · 트레이 열기 성공 · 끄기로 지운다. 켜짐 · 건수는 한
  잠금으로 직렬화. 종류별(페어링 · 받은 요청 · 받은 답 · 자동 왕복) 켜기 · 끄기와 본문 미리보기 끄기. 자동 왕복은 기본 시작 1 + 끝 1
  (저절로 멈춘 것만)로 묶고, "글마다" · "알리지 않음" 을 고를 수 있다. 자동 왕복 상한 기본값도 설정(1 ~ 50, 처음 10).
- **시작 실패**: 트레이 오류 모드(사유 파일 = 임시 폴더 `coworkAgent-start-error.txt`), 트레이도 못 띄우면 그 파일을 OS 기본 앱으로.
- **웹UI 구조**(D26): 시안 A 3단(PC 레일 · 사이드바 · 대화) 기본 + C 요청 타임라인 + 레일 하단 설정. 레일에서 PC 를 고르면 그 상대 대화만,
  다른 PC 를 고르면 열린 대화를 닫는다. 보내기는 해시의 대화 · 레일 상대가 맞을 때만. 늦게 온 조회 응답은 버린다.
- **로컬 웹UI CSRF**: 쿠키 인증의 상태 변경(GET/HEAD 밖) 요청은 Origin 필수 — 쿠키가 포트를 구분하지 않는 위험(#00 개정 1)을 막는다.
- **원격 메시지**: 신뢰하지 않는 입력(루트 `PROJECT.md` 가드레일). 웹UI 에 그릴 때 이스케이프·마크다운 sanitize 를
  거친다 — 원격 내용이 로컬 UI 에서 스크립트로 돌면 UI 토큰이 털린다(D8 · D9 귀결). 처리는 자동(D17).
- **Claude 실행**: `claude -p` 는 수신 PC 사용자의 계정·Claude 로그인으로 돈다 — 원격 요청이 수신 측 사용량을 쓰고,
  에이전트가 읽을 수 있는 것은 답에 실려 원격으로 나갈 수 있다(D16 · D17 귀결). 실행 폴더는 에이전트 프로필(D21), 권한은
  허용 도구만 — 기본 `--restricted --strict-mcp-config --permission-prompts none`(D22), 한도는 D24. 같은 대화(세션)는
  직렬로 실행한다 — 동시 resume 은 잠금 없이 섞인다(공식 문서). 기본 도구는 읽기 전용(`Read,Grep,Glob`), 권한 모드 `dontAsk` +
  `--allowedTools` = `--tools`. 원격 텍스트는 표준 입력으로만. 시간 제한 30분이 넘으면 자손 프로세스까지 종료.
- **대화 왕복**: 대화 UUID 를 양쪽이 공유, Claude 세션 ID 는 에이전트 쪽만(첫 `--session-id` → 이후 `--resume`). `message` 는 저장
  (= DB 대기열 등록) 뒤 바로 `ok`, 실행 결과는 `reply`(고정 키)로. 실행 후 기록은 한 트랜잭션(실패 → 오류 답). 답은 `reply_to` 가 답
  없는 내 메시지일 때만. 보내기는 대화별 직렬, 전송 오류는 실패로 확정하지 않는다(ACK 손실 시 중복 실행 방지). 재시작 시 `queued` 는 실행, `running`(실행 도중 종료)은 다시 돌리지 않고 오류 답. 본문 16 KiB.
- **문의 응답 전용**(D31): 원격 실행은 수신 PC 프로젝트 · 제품에 대한 문의에 답만 한다 — 가드 시스템 프롬프트 항상(재개 포함),
  도구는 읽기 전용 목록(`Read` · `Grep` · `Glob` · `LS` · `NotebookRead`)만, Claude Code 2.1.267 미만이면 실행하지 않는다.
- **Claude 자격 증명**: coworkAgent 는 Claude 자격 증명·세션 토큰을 수집·저장·중개하지 않는다 — `claude` 가 그 PC 에
  설정된 인증을 스스로 쓴다(Anthropic 정책 원문 귀결). 계정은 각 PC 의 Max 구독 로그인(D20) — 구독 OAuth 를 읽지 않는
  `--bare` 는 쓰지 않는다.
- **실행 파라미터**: D14 literal 이 계약 — 다른 툴이 부르므로 바꾸면 호환이 깨진다(변경은 사용자 승인).
- **양쪽 발화**(D18 · D23): 양쪽 사람 모두 글마다 "에이전트에게(실행) / 사람에게(실행 없음)". 대화의 에이전트는 받은 쪽에 하나.
  양쪽 이력은 같은 순서(램포트 시계 `lc`, `(lc, uuid)` 정렬). 실행 때 그 사이 사람 글을 맥락으로 앞붙임.
- **자동 왕복**(D24): 대화를 여는 쪽이 내 에이전트를 함께 고르면 상대 에이전트 답 → 내 에이전트 → 상대 에이전트 실행 요청을 상한(기본 10,
  1 ~ 50 왕복 — 내 에이전트가 한 번 답하면 1)까지 반복. 연 쪽이 구동하고 재개(카운트 0)한다. 어느 쪽 사람이든 쓰거나 멈추면 멈추고,
  받은 쪽이 멈추면 그 PC 는 에이전트발 요청을 `stopped` 로 거절. 실패 답 · 전달 거절이면 멈춤. 에이전트발 요청도 페어링 · 대화 소유 ·
  가드(D31) 그대로.
- **스키마 올리기**: 한 트랜잭션, 올리기 전 `{db}.v{버전}.bak` 사본(롤백용). 이전 실행 파일은 더 새 스키마를 열지 않는다.
- **저장**: D15 경로 아래 SQLite — 요청/응답 전체 + 대화별 Claude 세션 ID(이어서 작업용, D19).

## 개발된 모듈 / 매뉴얼

| Layer | 모듈 | 의존(실측) | 매뉴얼 |
|---|---|---|---|
| base | `an_core` | 없음 | [manual/dev/an_core.md](docs/manual/dev/an_core.md) |
| comm | `an_net` | `an_core` | [manual/dev/an_net.md](docs/manual/dev/an_net.md) |
| domain | `an_agent` | `an_core` · `an_net` | [manual/dev/an_agent.md](docs/manual/dev/an_agent.md) |
| app | `an_app` · bin `coworkAgent` | `an_core` · `an_net` · `an_agent` | [manual/dev/an_app.md](docs/manual/dev/an_app.md) |

## 설계 문서

- [manual/README.md](docs/manual/README.md) — 모듈 매뉴얼 라우팅(`dev/` 개발자·AI · `user/` 쓰는 사람)
- `docs/tech/` — 횡단 기술 노트 · 환경 함정(개발 전 필독). 생기면 한 줄씩 링크로 등록한다.
  - [tech/windows-crates-io-tls-revocation.md](docs/tech/windows-crates-io-tls-revocation.md) — Windows 에서 crates.io
    TLS 폐기 확인 실패 → cargo `check-revoke` 설정
