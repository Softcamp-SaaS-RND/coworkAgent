# PRODUCT.md — coworkAgent

**무엇을 만들었는가**의 단일 위치 — 구조 · 규칙 · 설계. 프로세스·환경은 루트 `PROJECT.md`,
진행 현황은 루트 `STATE.md`. 계약은 [etc/doc-conventions.md](docs/etc/doc-conventions.md) "PRODUCT.md — 필수 6절".
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
  app 은 `ShellExecuteW` — 브라우저 · 파일 열기, macOS `open` 과 같은 동작 · 레지스트리(`RegGetValueW` · `RegSetKeyValueW` · `RegDeleteKeyValueW`) —
  로그인 자동 실행 `HKCU\…\Run`, macOS LaunchAgent 파일과 같은 동작 · an-agent 는 Job Object — 자손 프로세스 종료, 프로세스 그룹과
  같은 동작) · futures-util 0.3.34(SSE 스트림) · rustls 0.23.45(`ring`, TLS 1.2 미포함) · tokio-rustls 0.26.6 · ring 0.17.14(ed25519 ·
  SHA-256) · serde 1.0.229 · serde_json 1.0.152(피어 프레임 · API).
- **웹UI 글꼴**(#1p): IBM Plex Sans KR 1.1.0 woff2 3종(SIL OFL 1.1 — `/fonts/LICENSE.txt`)을 실행 파일에 넣음(+1.25 MiB).
- **빌드**: `cargo build --release --workspace` → Windows `target/release/coworkAgent.exe`(단일 파일). macOS universal 은
  `lipo` 절차(Phase 4, D25) — [archive/00.skeleton/plan.md](docs/archive/00.skeleton/plan.md) "macOS universal".
- **웹UI 테스트**: `node --test` — 화면 로직 모듈(`crates/app/web/view.js`)만, Node 22+ 는 개발 PC 에만(제품 의존 아님).
- **테스트**: `cargo test --workspace` — **경고 baseline**: 없음 <!-- 외부 의존 경고만 여기 고정. 우리 코드 경고는 0 -->

## 모듈 Layer 구조

| Layer | 크레이트 | 담는 것 |
|---|---|---|
| base | `an-core`(`crates/core`) | 데이터 경로 · SQLite · 로그 회전 · 단일 인스턴스 · 실행 옵션 · 화면 설정 검증 |
| comm | `an-net`(`crates/net`) | 신원 · RPK TLS · 피어 프로토콜 · 페어링 · 초대 |
| domain | `an-agent`(`crates/agent`) | `claude -p` 실행기 · 대화 왕복 조율 · DB 대기열 |
| app | `coworkagent-app`(`crates/app`) | 웹UI 서버 · 내 정보 · 피어 · 에이전트 · 대화 API · 핸드오프 · AI 에이전트용 CLI · 트레이 · 브라우저 · 시작 순서 |

<!-- 의존 방향 app → domain → comm → base 단방향. 비-Python 이라 `check_layers.py` 는 두지 않는다 — 이 절의
     표를 검사하는 수단은 루트 PROJECT.md "검증 명령" 의 "의존 경계" 행. -->

**의존 경계 규칙**: 간선은 app → domain → comm → base 방향만(`tools/check_crates.py` 의 `LAYER` 가 정본). base 는 UI · HTTP ·
트레이를 모른다. 새 크레이트는 `LAYER` 와 이 표에 함께 등록한다.

## 도메인 규칙

지켜야 하는 제품 규칙만 둔다 — 흐름 · 세부는 [개발자 가이드](docs/manual/dev/developer-guide.md) 의 절, 화면 규칙은 [web-ui](docs/manual/dev/web-ui.md). 결정 번호는 [masterPlan.md](docs/masterPlan.md) `## 확정 방향`.

### 신원 · 보안

- **신원**: 설치별 ed25519 키쌍(D8) — `identity.key`(PKCS#8). 비밀키는 사용자 데이터 경로(D15) 밖으로 내보내지 않는다 — 리포·로그·UI
  금지(D8 귀결). 손상 키는 재생성하지 않는다(기존 페어링 보호). 지문 = SHA-256(SPKI) 앞 16바이트.
- **페어링**(D8 개정): 한쪽 "초대 생성" → 초대 텍스트(주소 · 초대 코드 · 지문)를 상대에게 → 상대가 "초대 수락" 에 붙여 넣으면 지문으로
  서버 키를 고정해 연결 · 추가 · 내 승인. 초대 생성 측은 그 초대 코드가 맞는 처음 보는 키 1건(5분)을 사람 승인 없이 등록하고(Hello 응답
  `approved`) "새 연결 등록" 알림 · 그룹설정에서 바로 삭제 — 위험 수용(초대가 새면 창 안에서 제3자가 등록, 주소+코드 직접 추가는 지문을
  고정하지 않아 LAN MITM 도 수용 — 사내망 전용, #1j 부터 웹UI 칸은 없고 API 만). 아는 키의 다시 추가는 사람 승인. approve 는 같은 세대만. 미페어링 상대는 hello · approve ·
  remove 외 거부(SC2).
- **로컬 웹UI 인증**(#1a): 쿠키를 쓰지 않는다 — 쿠키는 포트를 가리지 않아 같은 PC 의 다른 `127.0.0.1` 포트로 실린다. 토큰은 탭 메모리 +
  `Authorization: Bearer`(알림 스트림만 `?t=`), 정적 자산은 공개(Host 검사), 모든 응답에 CSP(`frame-ancestors 'none'`) · `X-Frame-Options: DENY` ·
  `nosniff` · `no-referrer`. 위협 모델은 **한 PC = 한 사용자**(다른 로그인 사용자 · 같은 사용자 악성 프로세스는 밖).
- **웹UI 다시 읽기**(D39 ② · #1u): 알림 스트림(`/api/events`)에 알림과 별개인 변경 신호 `event: change`(`{res: [{kind, id?}]}`)가 함께 흐른다 — 알림 설정 · 일시 중지 ·
  집중과 무관. 웹UI 는 그 자원을 쓰는 로더만 다시 읽고(`view.js` `loadersFor`), 다시 읽기는 한 번에 하나(`OneAtATime`). 5초 전체 새로 고침은 그대로 — 화면 규칙 [web-ui](docs/manual/dev/web-ui.md) "알림".
- **데이터 폴더 권한**(#1a): 주 인스턴스 시작마다 소유자 전용(Windows 보호 DACL 사용자 · SYSTEM / Unix 0700). OS 전용 암호화(DPAPI 등)로 키를
  감싸지 않는다 — Windows · macOS 가 같은 방식.
- **원격 실패 · 프롬프트**(#1a): 상대에게는 정해진 실패 문구만(stderr · 경로 · DB 오류는 내 쪽만) · 실행 입력은 늘 구분자 틀 · 원격 대화 id 는 UUID 꼴만.
- **원격 메시지**: 신뢰하지 않는 입력(루트 `PROJECT.md` 가드레일). 웹UI 에 그릴 때 이스케이프·마크다운 sanitize 를
  거친다 — 원격 내용이 로컬 UI 에서 스크립트로 돌면 UI 토큰이 털린다(D8 · D9 귀결). 처리는 자동(D17).
- **두 OS 같은 UI**(D30): 화면 · 알림은 웹UI 와 tray-icon 으로만 — OS 전용 UI API(WinRT · 메시지 상자 · AppleScript)를 쓰지 않는다.

### Claude 실행

- **Claude 실행**: `claude -p` 는 수신 PC 사용자의 계정·Claude 로그인으로 돈다 — 원격 요청이 수신 측 사용량을 쓰고,
  에이전트가 읽을 수 있는 것은 답에 실려 원격으로 나갈 수 있다(D16 · D17 귀결). 실행 폴더는 에이전트 프로필(D21), 권한은
  허용 도구만 — 기본 `--restricted --strict-mcp-config --permission-prompts none`(D22), 한도는 D24. 같은 대화(세션)는
  직렬로 실행한다 — 동시 resume 은 잠금 없이 섞인다(공식 문서). 도구는 프로필과 무관하게 읽기 전용 3개(`Read,Grep,Glob` —
  #1d), 권한 모드 `dontAsk` +
  `--allowedTools` = `--tools`. 원격 텍스트는 표준 입력으로만. 시간 제한 30분이 넘으면 자손 프로세스까지 종료. `--model` · `--effort` 는
  그 상대 덮어쓰기 → 전역 설정 → `sonnet` · `high`(D22 보강, 모델은 `opus` · `sonnet` · `haiku` 만 — #1d) — 설정을 못 읽으면 실행하지 않고 실패 답. 세션은 에이전트 폴더에 묶인다 —
  폴더를 바꾸면 그 에이전트 대화의 세션을 비우고, 실행 도중 바뀌었으면 끝난 실행의 세션을 남기지 않는다. 에이전트별 **상위 한단계 탐색 허용**(#05-2, 기본 끔) —
  켜면 바로 위 폴더를 `--add-dir` 로 더 연다(드라이브 루트 · 사용자 홈 · 데이터 루트를 품는 위는 열지 않음), 바꾸면 세션을 비운다.
  에이전트별 **셸 명령 실행 허용**(#05-3, 기본 끔) — 켜면 도구에 `Bash` · `PowerShell`(macOS 는 `Bash`)을 더하고 셸용 가드(이 PC 에 설정된 workflow 수행 · 조회용 — workflow 산출물은 예외, 소스 · 문서 · 설정 직접 삭제 · 편집 · 저장소 변경 금지),
  `claude -p` 에 앱 환경 변수를 넘기지 않는다(기본 변수 목록만). 셸에는 폴더 · 상위 읽기 · 데이터 루트 제한이 적용되지 않는다 — 켤 때 위험 확인 창, 압축 요약은 셸 없이.
- **문의 응답 전용**(D31): 원격 실행은 수신 PC 프로젝트 · 제품에 대한 문의에 답만 한다 — 가드 시스템 프롬프트 항상(재개 포함),
  도구는 읽기 전용 목록(`Read` · `Grep` · `Glob`)만, Claude Code 2.1.267 미만이면 실행하지 않는다.
- **Claude 자격 증명**: coworkAgent 는 Claude 자격 증명·세션 토큰을 수집·저장·중개하지 않는다 — `claude` 가 그 PC 에
  설정된 인증을 스스로 쓴다(Anthropic 정책 원문 귀결). 계정은 각 PC 의 Max 구독 로그인(D20) — 구독 OAuth 를 읽지 않는
  `--bare` 는 쓰지 않는다.
- **실행 한도**(D24 개정 #1b): PC 전체 동시 실행 상한(기본 2 · 1 ~ 8, 설정 `max_runs`) — 실행 문(`an_agent::gate`)이 사람 요청 · 자동 왕복을 한 줄(FIFO)로
  묶고, 넘는 요청은 거절하지 않고 기다린다. 흐름 · 진행 알림은 개발자 가이드 3.2.

### 대화 · 통신

- **양쪽 발화**(D18 · D23 개정): 양쪽 사람 모두 쓸 수 있고, 새 글은 모두 대화 에이전트에게 실행 요청이다(사람에게 보내기 없음 — 웹UI ·
  CLI · API `to: "human"` 400). 실행 없는 사람 글은 이전 이력으로 보이고 받기도 남는다. 대화의 에이전트는 받은 쪽에 하나.
  양쪽 이력은 같은 순서(램포트 시계 `lc`, `(lc, uuid)` 정렬). 실행 때 그 사이 사람 글을 맥락으로 앞붙임.
- **대화 왕복**: 대화 UUID 를 양쪽이 공유, Claude 세션 ID 는 에이전트 쪽만. 보내기는 대화별 직렬, 전송 오류는 실패로 확정하지 않는다 — 개발자 가이드 6.4.
- **보내기 보장**(D7 실현 #1c): 상대에게 보내는 대화 프레임은 먼저 SQLite `outbox` 에 적고 상대별 줄이 id 순으로 보낸다(24시간 닿지 않으면 실패 보관) —
  개발자 가이드 6.6.
- **피어 주소**(D7 보강 #1b): 47100 을 못 열면 임의 포트, 페어링된 키의 `Ping{port}` 로 주소를 따라간다 — 개발자 가이드 5.1.
- **자동 왕복**(D24): 상한(기본 10 · 1 ~ 50 왕복)에서 멈추고, 어느 쪽 사람이든 쓰거나 멈추면 멈춘다 — 개발자 가이드 8.3.
- **질의 · 요청**(S1 개정 · #1d): **요청은 실행하지 않는다** — 받은 쪽 사람만 수용 · 반려 · 완료, 바뀐 상태는 `ReqState` 로 쓴 쪽에 — 개발자 가이드 8.6.
- **@멘션**(D19 보강 · #18): 내가 연 대화(자동 왕복 아님)에서 글의 첫 `@이름` 이면 그 상대 PC 에이전트를 부른다 — 개발자 가이드 8.4.
- **집중 · 접속 상태**(D17 개정 · #18): 집중 동안 상대의 새 실행 요청은 받는 순간 `focus` 로 거부한다. 접속 = 마지막 통신 시각(주기 확인 없음) —
  개발자 가이드 8.5 · 8.8.
- **이력 정리**(D34 · #1f): 편집 · 삭제는 **끝난 내 사람 글**만(`Chat::tidy_ok` 한 곳), 실행 · 알림 없음 — 개발자 가이드 8.9. 웹UI 편집 모드는 없다(#1v — CLI · API 만).
- **요청 보드 · 의견 말풍선**(D39 ① · #1v · #1z): '요청' 화면 목록 / 보드, 받은 요청 카드를 손잡이 포인터 끌기로 다른 열에 놓으면 그 열로 옮겨진 채 의견 칸 → 확인 시 `req:{상태}` 그대로(상태는 네 상태 사이 어디로든 — `ReqState::can_become`, 2026-10-19). 요청 아래 의견 말풍선(지금 상태 · 의견).
  의견은 여러 줄 200자(#1z — 저장 `\n`, 보낼 때 U+2028 · 받을 때 `\n` 으로 되돌림 — 이전 버전 `clean_label` 통과, `chat::note_to_wire` · `note_from_wire`).
- **요청 archive · 보기 기간**(D35 보강 · #1y): 완료 요청은 완료 7일 뒤 자동 · 손으로 archive(이 PC 에만 — `req_archived` 스키마 23, 상태가 바뀌면 풀림), 기본 숨김 ·
  "archive 항목 보기" 로 따로. 주고받은 메시지 "기간" 1 ~ 12주(`ui_peer_weeks` 기본 4 — 보기만), 시작 때 DB VACUUM(빈 공간 4 MiB · 1/4 이상) — 개발자 가이드 8.6.
- **공유 대화**(D38 · #1o): 페어링된 상대 5명까지 읽기 전용 — `Share*` 프레임, 24시간 닿지 않으면 끝 — 개발자 가이드 8.11.
- **대화 나가기 · 기록 삭제**(D40 · #1w · #1x): 모든 대화 ⋯ 메뉴 — 나가기 = 확인("대화방이 목록에서 삭제됩니다.") 뒤 이 PC 에서 삭제(상대에 `Leave` — 이전 버전엔 거절
  `left`만 · 묘비 · 번호 재사용 없음), 기록 삭제 ▸ 내 글만(이 PC 글 비움) · 모든 사용자(상대 PC · 관람자 사본도 — `Clear{conv, id}` · `Pong.clear`, 대화는 남음 · 보내는 중인 글이 있으면 막음), 관람자는 나가기만,
  도는 실행 · 대기 글이 있으면 막음, 지운 글 uuid `gone_msgs` · 글 번호 재사용 없음(스키마 22) — 개발자 가이드 8.12.
- **세션 생성 · 화면**(#1x · S4 · D35): + 세션 생성 = 글 없이 대화만(`POST /api/conversations` 빈 본문, 첫 글은 대화방 입력창 · 자동 왕복은 첫 글부터), 사이드바 대화 줄 손잡이
  포인터 끌기(☆ 없음 — 즐겨찾기는 대화 머리), 타일 크기 `ui_tile_size`, 기본 테마 라이트, 받은 요청 알림 묶음(`request` — 같은 tag · 요청 보드) · 효과음 `notify_sound` —
  [web-ui](docs/manual/dev/web-ui.md).
- **사람 ↔ 에이전트 구분 · 레일 끌기**(#1z · S4 · D35 · D37): 말풍선 A(에이전트 글 = 흰 면 · 테두리 · "에이전트" 칩 · 로봇 아바타 양쪽), 대화 목록 카드 · 발화자 아이콘,
  입력창 여백 `--chat-gutter` · 스크롤 밖 24px, 레일 폭 손잡이 끌기로 타일 크기(놓으면 72 · 168px, 120px 기준), 레일 타일 순서 포인터 끌기, 설정 "그룹설정" 목록 + 상세 — [web-ui](docs/manual/dev/web-ui.md).
- **에이전트 응답 · 그룹설정 B**(#1z-2 · D35 · D37 · S4): '에이전트 응답'(옛 에이전트 대화) '나' 에만 · 상대 타일 배지에서 에이전트 대화 제외, 대화 줄 한 줄(유형 아이콘 · 제목),
  레일 "+ 사용자" · 놓는 자리 = 가장 가까운 타일, 그룹설정 = 그룹 카드 격자 + 사용자 표(줄 펼침 · 대화 줄을 카드에 포인터로 넣기 — #1z-8 이 목록 + 상세로 대체), 주고받은 메시지 원형 아바타 줄 — [web-ui](docs/manual/dev/web-ui.md).
- **대화 유형 색 · 내 에이전트 상세**(#1z-7 · D21 · D35 · S4): 내 에이전트 응답(`in`) = 청록 · 로봇, 내가 문의(`out`) = 파랑 · 사람, 공유 = 회색 점선 — 대화 머리 · 사이드바 줄 ·
  개요 카드 · 목록 줄 같은 규칙(`data-kind`). 설정 '내 에이전트' 상세 한 격자 · 아이콘 색 6개(스키마 26 `agents.color`, 이 PC 에만) · 닫기 — [web-ui](docs/manual/dev/web-ui.md).
- **설정 시안 B · 검색 스위치 · 디버깅**(#1z-8 · S4 · D35 · D37 · D40): 설정 = 왼쪽 메뉴 8개(연결 · 그룹설정 · 에이전트 · 실행 · 화면 · 시작 · 알림 · 디버깅) · 고른 화면만 ·
  처음 = 연결, 연결 · 그룹설정 · 에이전트 = 목록 + 상세(줄 고름 · ✕ · 닫기), 나머지 = 2열 타일, 체크박스 = 스위치. 디버깅 = `GET /api/debug`(버전 · OS · 데이터 폴더 · DB · 로그
  경로 · 크기) · `GET /api/debug/log`(끝 200줄 · 64 KiB) · 진단 정보 복사(키 · 토큰 · 지문 없음). 통합 검색 `closed` 기본 거짓 = 닫힌 대화 뺌(`chat::search` · `closed_sql`) · 스위치
  "나간 대화 포함" · 유형 칩, 상단 바 버전, 미리보기 말풍선 2줄, 대화 정보 `제목` 줄 없음 — [web-ui](docs/manual/dev/web-ui.md) · [an_app](docs/manual/dev/an_app.md).
- **개요 미리보기**(#1z-4 · #1z-5 · #1z-6 · S4 · D35): 개요 기본 보기 = 상대마다 절 + 대화방마다 축소 채팅 카드(2차 시안 A — 상대마다 한 줄, 열 수 = 브라우저 폭, 넘치면 "+n개",
  상대 타일은 카드 전부 · 세로 스크롤), "목록 | 미리보기" 는 설정 `ui_home_view` 에 기억, "미리보기 크기" 100 ~ 200%(카드 폭 · 말풍선 칸 높이, 말풍선은 2줄 — #1z-8, 글자 그대로, `ui_home_zoom`), 글은 `?limit=` 마지막 몇 개만(SQL 에서 자름) · 바뀐 대화 · 변경 신호 때만 다시 읽음 · 읽음은 바꾸지 않음 — [web-ui](docs/manual/dev/web-ui.md).
- **컨텍스트**(D19 보강 · #1m): 압축(요약 후 새 세션) · 비우기(세션만) · 80% 자동 압축 — 개발자 가이드 6.3.

### 실행 · 배포

- **실행 구조 · 자동 시작**(S7 · D13 · #1e · #1n): 인자 없는 실행 = 감시 프로세스 + 앱 자식(비정상 종료면 다시 띄움), 감시 · 앱은 OS 사용자당 하나, 인자
  있는 실행(CLI)은 감시를 거치지 않는다 — 개발자 가이드 3.1.
- **자동 업데이트**(S6 · D12 · D36 개정 · #1y · #30): 트레이 모드 감시가 앱을 띄우기 전 GitHub 정식 릴리스(`Softcamp-SaaS-RND/coworkAgent` `releases/latest`)를 보고, 새 버전이면
  받아 회사 인증서 Authenticode 서명(서명자 `SOFTCAMP CO.,LTD.`)을 확인한 뒤 묻지 않고 바꿔 다시 뜬다 — 실패면 지금 버전, Windows 만. 릴리스 절차는 개발자 가이드 2.4.
- **시작 실패**: 트레이 오류 모드(사유 파일 = 임시 폴더 `coworkAgent-start-error.txt`) — 개발자 가이드 3.1.
- **실행 파라미터**: D14 literal 이 계약 — 다른 툴이 부르므로 바꾸면 호환이 깨진다(변경은 사용자 승인).
- **AI 에이전트용 CLI**(D14 보강 · #15 `say` 삭제): `coworkAgent peers` · `ask` · `history` — 떠 있는 주 인스턴스의 로컬 API 를 `instance.json` 토큰으로
  부른다(웹UI 와 같은 신뢰 경계, 앱을 띄우지 않는다). 종료 코드 · 트랜잭션(`cli_tx`)은 개발자 가이드 8.7.
- **부모 도구 메뉴**(#1n · D14 보강): `coworkAgent start` · `open` · `stop` — 어떤 실행 경로로도 앱 · 감시는 하나, 트레이는 많아야 하나 — 개발자 가이드 8.10.
- **CLI 도움말**(#1q · D14 보강): `coworkAgent /?`(`-h` · `--help`, 첫 인자만) — 앱 없이 명령 · 출력 형식 · 종료 코드를 표준 출력 · 0, 다른 AI 도구 연동용(README).
- **내 활동 범위**(#1q · D35): 받은 요청 · 내 요청(상대가 상태를 바꾸면 `relay_events.kind = req_state` 로 안 본 것) · 받은 답 · 실패만 — 화면 규칙은 [web-ui](docs/manual/dev/web-ui.md).
- **웹UI 다듬기**(#1r · S4 · D35): 세션 생성 "내 응답 에이전트"(= 자동 왕복 상대 에이전트, `relay_agent`) + 하위 줄 "최대 응답 횟수"(`relay_cap`) · 메뉴 아이콘 상시 ·
  연필 통일 · `.peer-fold` 단계 들여쓰기 · 설정 "초대 생성" 메뉴 = 초대 만들기(`press`, 한 번에 하나) · 초대 수락 칸은 라우터가 여닫음 — API 변경 없음, [web-ui](docs/manual/dev/web-ui.md).

### 저장

- **저장**: D15 경로 아래 SQLite — 요청/응답 전체 + 대화별 Claude 세션 ID(이어서 작업용, D19). 대화 제목 · 타일 표시 이름 · 아이콘 · 그룹 ·
  상대별 model/effort · ★ · 저장 · 읽음 · 내 활동 본 표시 · 알림 일시 중지 · 집중 · 접속 상태는 이 PC 에만(상대에 보내지 않는다). 질의 · 요청 · 완료 ·
  멘션 이름은 양쪽에. 답의 실행 model · effort 는
  실행한 쪽이 답과 함께 보내 양쪽에 남는다.
- **스키마 올리기**: 단계마다 한 트랜잭션 · 올리기 전 사본, 이전 실행 파일은 더 새 스키마를 열지 않는다 — 개발자 가이드 7.5.
- **레일 그룹**(D37 개정 · #1w): 이 PC 에만 있는 **대화** 묶음(상대 단위 소속 없음) — 상대에 보내지 않는다. 저장은 개발자 가이드 7.2, 화면은 [web-ui](docs/manual/dev/web-ui.md).

## 개발된 모듈 / 매뉴얼

| Layer | 모듈 | 의존(실측) | 매뉴얼 |
|---|---|---|---|
| base | `an_core` | 없음 | [manual/dev/an_core.md](docs/manual/dev/an_core.md) |
| comm | `an_net` | `an_core` | [manual/dev/an_net.md](docs/manual/dev/an_net.md) |
| domain | `an_agent` | `an_core` · `an_net` | [manual/dev/an_agent.md](docs/manual/dev/an_agent.md) |
| app | `an_app` · bin `coworkAgent` | `an_core` · `an_net` · `an_agent` | [manual/dev/an_app.md](docs/manual/dev/an_app.md) |

## 설계 문서

- [manual/README.md](docs/manual/README.md) — 모듈 매뉴얼 라우팅(`dev/` 개발자·AI · `user/` 쓰는 사람)
- [manual/dev/developer-guide.md](docs/manual/dev/developer-guide.md) — 프로세스 · 서비스 역할 · 세션 관리 · 저장소 · 워크플로우(모듈을 가로지른 실행 구조)
- [manual/dev/web-ui.md](docs/manual/dev/web-ui.md) — 웹UI 화면 규칙(구조 · 말풍선 · 설정 · 알림 · 공유 대화 화면 · 접속 표시 · 디자인 토큰 · 문구 규칙 · 용어 표) — 화면을 고칠 때만
- `docs/tech/` — 횡단 기술 노트 · 환경 함정(개발 전 필독). 생기면 한 줄씩 링크로 등록한다.
  - [tech/windows-crates-io-tls-revocation.md](docs/tech/windows-crates-io-tls-revocation.md) — Windows 에서 crates.io
    TLS 폐기 확인 실패 → cargo `check-revoke` 설정
