# 보안 태세 — 외부 공격자 대비 안전조치 · 점검 이력 · 남은 위험

> 횡단 기술 노트. **새 기능이 외부 입력(피어 프레임 · 원격 본문 · 로컬 API)을 늘릴 때 · 보안 점검을 다시 할 때** 읽는다.
> 기준: 2026-10-19 · 브랜치 `task/1w-conv-leave` `03eb318`(main `b21b325` + #1w plan · stage 1 · 2 의 3커밋 — `Leave` 프레임은 main 미합류).
> 위치는 **심볼 이름**으로 적는다(줄 번호는 개정마다 어긋난다). 근거 없는 항목은 `확인 필요`.
> 출처: [1a 보안 점검](../archive/1a.security-hardening/audit.md) · [1a report](../archive/1a.security-hardening/report.md) ·
> [미조치 원장](../overview/ledger.md) · [masterPlan](../masterPlan.md) D8 · D9 · D22 · D24 · D31 · D33 + 이 노트 작성 시 소스 대조.

## 1. 위협 모델 (D33)

| 구분 | 주체 | 범위 |
|---|---|---|
| **외부 — 대상** | 같은 LAN 의 **미페어링 호스트**(포트 스캔 · 초대 코드 추측 · 연결 고갈) | 안 |
| | **페어링된 상대가 악의적이거나 탈취됨**(조작 프레임 · 프롬프트 주입 · 대량 전송) | 안 |
| | 원격 본문이 수신 PC 의 `claude -p` 를 조종(프롬프트 주입 → 파일 유출) | 안 |
| | 원격 본문이 로컬 웹UI 에서 스크립트로 실행(XSS → UI 토큰 탈취) | 안 |
| | 같은 PC 의 **다른 `127.0.0.1` 포트 웹 페이지**(악성 로컬 서버 · 요청을 기록하는 개발 서버) | 안 |
| | 같은 사용자가 띄운 샌드박스 · 서비스 계정(상속 그룹) | 안 |
| | 공급망(crates.io 의존 · DLL 심기) | 안 |
| **범위 밖** | 다른 로그인 사용자 · 같은 사용자 권한의 악성 프로세스(이미 모든 사용자 파일을 읽는다) | 밖 — D33 |
| | OS 전용 암호화(DPAPI 등)로 키 감싸기 | 기각 — Windows · macOS 상호 통신 |

**보호 자산**: 설치 신원 `identity.key`(PKCS#8 평문 — 탈취 시 모든 상대에게 이 PC 로 위장) · 로컬 UI 토큰(`instance.json` — 로컬 API 전권) ·
대화 DB(`%LOCALAPPDATA%\coworkAgent`) · 에이전트 폴더 내용(원격 문의의 읽기 범위) · 수신 PC 의 Claude 사용량.

**수용된 위험(D8 개정)** — 지적 대상이 아니다: ① 초대 생성 측은 5분 창 안 첫 맞는 키 1건을 사람 승인 없이 등록(초대 텍스트가 새면 제3자 등록) ·
② 주소 + 코드 직접 추가는 지문을 고정하지 않음(LAN MITM — 사내망 전용, 웹UI 칸 없음 · API 만).

```text
[미페어링 LAN 호스트] ─TCP 47100(0.0.0.0)─┐
[페어링된 상대]       ─RPK TLS 1.3──────────┤→ an-net node ─(paired 게이트)→ an-agent chat ─stdin→ claude -p (--restricted, 읽기 전용 5 도구)
                                             │                                   │
[같은 PC 다른 포트 페이지] ─HTTP─X(Host/Origin/Bearer)→ app server 127.0.0.1:동적 ← 웹UI(탭 메모리 토큰) · CLI(instance.json 토큰)
```

## 2. 경계별 안전조치

### 2.1 피어 네트워크 (`an-net`)

| 조치 | 위치 | 막는 것 |
|---|---|---|
| TLS 1.3 만 · 서명 ED25519 만 · RPK(원시 공개키) · TLS 1.2 서명 검증 거부 | `tls::server_config` · `client_config_pin` · `tls12_refused` · `supported_verify_schemes` | 다운그레이드 · 약한 서명 |
| 클라이언트 인증서 필수 + 실제 서명 검증 · 페어링 뒤 송신은 저장된 SPKI 고정 | `tls` `verify_client_cert` · `verify_server_cert`(`Pin::Key`) | 위장 · MITM |
| 미페어링 키는 `Hello` · `Approve` · `Remove` 외 전부 `not_paired` | `node` 요청 분기(대화 · `Share*` · `Leave` 묶음) | 미인증 기능 호출(SC2) |
| 페어링 상대라도 id 꼴 검사 — 원격 대화 id 는 UUID 꼴만(`invalid`) | `Request::ids_ok` · `proto::is_uuid` | 조작 id 주입 |
| `Approve` · `Remove` 는 인증된 **자기** 기록에만 · `approve` 는 같은 세대만 | `node` `Request::Approve` · `Request::Remove` | 남의 페어링 조작 |
| 초대 창 5분 · 코드 상수 시간 비교 · 틀린 코드는 창을 닫지 않음(선점 방지) · **IP 당 10회 틀리면 그 IP 창 끝까지 거절** | `PAIRING_WINDOW` · `ct_eq` · `MAX_CODE_MISSES` | 초대 코드 추측 |
| 프레임 길이 검사가 할당보다 먼저(64 KiB) · 본문 16 KiB | `proto::MAX_FRAME` · `MAX_TEXT` | 메모리 고갈 |
| 연결 전역 32 · 출처 IP 당 4 · 인증 전 3초 · 입출력 10초 | `MAX_CONNS` · `MAX_CONNS_PER_IP` · `INBOUND_DEADLINE` · `IO_DEADLINE` | 연결 슬롯 장기 점유 |
| 원격 이름의 서식 문자(Cf — U+202E RLO · U+200B 등) 제거 · 상대 거절 사유는 정해진 꼴만 | `proto::clean_name` · `is_format` · `refused`(`clean_code`) | 이름 위장 · 오해 유발 문구 |
| 대화는 `(uuid, peer)` 로 찾는다 — 남의 대화에 끼어들 수 없다 | `chat::find` · `find_own` | 교차 대화 조작 |
| 공유(#1o) 프레임은 그 상대가 보낸 살아 있는 사본에만 · 관람자 수 ≤ 5 · 끝은 그 상대 관람자 행만 | `inbound_share_post` · `live_copy` · `inbound_share_notice`(`MAX_WATCHERS`) · `inbound_share_end` | 남의 공유 사본 오염 |
| 나가기(#1w) 는 그 상대와의 **내 소유** 대화에만 · 없는 대화도 `Ok`(존재 비노출) | `inbound_leave`(`find_own`) | 남의 대화 상태 조작 · 존재 탐지 |

### 2.2 원격 → 에이전트 실행 (`an-agent`)

| 조치 | 위치 | 막는 것 |
|---|---|---|
| 원격 텍스트는 **표준 입력으로만**(argv 주입 없음) · `claude.exe` 전체 경로 직접 실행(`.cmd` 경유 BatBadBut 없음) | `runner::run_process` · `find_claude` | 명령 주입 |
| `--restricted --strict-mcp-config` · 권한 `dontAsk` · 도구 = 읽기 전용 5개(`Read` · `Grep` · `Glob` · `LS` · `NotebookRead`, 프로필 무관) | `runner::claude_args` · `agents::READ_ONLY_TOOLS` | 쓰기 · 셸 · MCP · 작업 폴더 밖 읽기 · 프로젝트 훅 · 스킬 |
| 문의 응답 전용 가드 시스템 프롬프트를 매 실행(재개 포함) · Claude Code `2.1.267` 미만이면 실행 안 함 | `GUARD_PROMPT` · `MIN_CLAUDE` | 가드 없는 실행(D31) |
| 실행 입력은 늘 머리말 + 구분자 틀 · 원격 글 안의 구분자 글자 제거 · 상대 이름은 `상대(…)` · 메모 20개 · 32 KiB | `with_notes` · `unframe` · `MAX_NOTES` · `MAX_NOTES_BYTES` | 틀 위장 프롬프트 주입 · 프롬프트 폭주 |
| 원격은 세션 · 모델 · 옵션을 정하지 못함(내 설정 → `sonnet` · `high`, 모델 3종만) | `chat` 실행 옵션 해석 | 사용량 증폭 · 세션 하이재킹 |
| **요청(S1)은 실행하지 않음** — 받은 쪽 사람만 수용 · 반려 · 완료 | `inbound` 요청 분기(#1d) | 원격이 일을 시킴 |
| 실패 답은 정해진 문구 7종만 — stderr · 경로 · DB 오류 · `is_error` 결과는 내 쪽만 · 빈 답도 실패 | `Failure` · `RUN_FAILED` · `RESULT_ERROR` · `EMPTY_ANSWER` · `FOLDER_BLOCKED` | 실패 유도 정보 수집(1a S3) |
| 시간 제한 30분 → Job Object(`KILL_ON_JOB_CLOSE`)로 자손까지 종료 | `RUN_TIMEOUT` · `runner` Job Object | 고아 프로세스 · 무한 실행 |
| PC 전체 동시 실행 상한(기본 2 · 1 ~ 8) · FIFO · 넘으면 대기 | `gate` · `max_runs` | 동시 실행으로 사용량 소진 |
| 자동 왕복 상한(기본 10 · 1 ~ 50) · 사람이 쓰거나 멈추면 멈춤 · 집중 중 새 실행 요청 `focus` 거부 | `default_relay_cap` · `FOCUS` | 에이전트끼리 무한 대화 |
| 에이전트 폴더가 데이터 루트를 품거나 네트워크 경로면 등록 · 수정 · **실행 직전** 거부 | `paths::agent_folder_problem` · `folder_blocked` | 원격 문의로 키 · DB 읽기(1a S7) |

### 2.3 로컬 웹UI · CLI (`coworkagent-app`)

| 조치 | 위치 | 막는 것 |
|---|---|---|
| `127.0.0.1` 동적 포트만 바인드 | `server::bind` | 원격 접근 |
| 토큰 32바이트 난수(실행마다) · 상수 시간 비교 · **쿠키 없음** — 탭 메모리 + `Authorization: Bearer`(알림 스트림 `/api/events` 만 `?t=`) | `new_token` · `ct_eq` · `bearer` · `check` | 다른 로컬 포트로 쿠키 유출(1a S2) |
| Host 정확 일치(`localhost` 도 거부) · Origin 있으면 정확 일치 | `check` | DNS rebinding · CSRF |
| 모든 응답 CSP(`frame-ancestors 'none'` · `object-src 'none'` · `script-src 'self'`) · `X-Frame-Options: DENY` · `nosniff` · `no-referrer` · 페이지 `no-store` | `CSP` · `harden` · `index` | 클릭재킹 · MIME 혼동 · 토큰 Referer 유출 |
| 원격 텍스트는 `textContent` · `setAttribute` 만 · `innerHTML` 은 고정 SVG 상수만 · HTML 본문은 `sandbox=""` iframe(`srcdoc`) | `app.js` `el` · `svgButton` · `iconButton` · 본문 뷰어 | XSS → 토큰 탈취 |
| CLI 는 `instance.json` 토큰으로 같은 신뢰 경계 · 원격 답의 C0 · C1 제어 문자 제거(`\n` `\t` 제외) | `cli` · `printable` | 터미널 ANSI · OSC 조작 |

### 2.4 저장 · 프로세스 · 빌드

| 조치 | 위치 | 막는 것 |
|---|---|---|
| 데이터 루트를 주 인스턴스 시작마다 소유자 전용(Windows 보호 DACL 사용자 · SYSTEM / Unix 0700 · 0600 · `umask 077`) | `owner_only::restrict` · `Paths::restrict` · `main` `umask` | 상속 그룹의 키 · 토큰 읽기(1a S1) |
| 비밀키는 데이터 경로 밖으로 내보내지 않음(리포 · 로그 · UI 금지) · 손상 키는 재생성 안 함 · 키 생성 원자적 | `identity` | 키 유출 · 페어링 파괴 |
| SQL 전부 바인드 · LIKE 이스케이프 · 로그에 본문 · 토큰 · 초대 코드 · 키 없음 | `an-core` `db` · `chat` | SQL 주입 · 로그 유출 |
| 오염된 잠금 이어 쓰기(panic 1회로 기능 정지 안 함) | `sync::lock` | 가용성 |
| 정적 C 런타임 · `/DEPENDENTLOADFLAG:0x800` · `SetDefaultDllDirectories(SYSTEM32)` | `.cargo/config.toml` · `main` | DLL 심기(1a S11) |
| Claude 자격 증명을 수집 · 저장 · 중개하지 않음 — `claude` 가 그 PC 인증을 스스로 사용 · `--bare` 안 씀 | `runner` | 자격 증명 유출 |

## 3. 점검 이력

| 일자 · 기준 | 범위 | 결과 |
|---|---|---|
| 2026-10-13 · main `35d5758`(#1a) | 읽기 전용 4갈래(로컬 웹 API · 피어 네트워크 · 원격 → 실행 · 저장 · 프로세스) + `claude 2.1.296` 실측 | Critical · High **0**(High 후보 1 = 작업 폴더 밖 읽기 → 실측 **반박**). D33 전제 뒤 Medium 1(S3) · Low 13 · 제외 1(S6) · Info 9. 조치 · 수용 내역 = [1a report](../archive/1a.security-hardening/report.md) |
| 2026-10-13 · #1a | 로컬 API gRPC 전환 검토 | **기각** — 보안 이득 없음 · 공격면 증가(HTTP/2 · protobuf) · 응답 1.4 ~ 6.6 ms 실측 · D10 충돌 |
| 2026-10-13 · #1a 적대적 평가 | 2 라운드(상한) | 라운드 1 high 1(`is_error` 결과 원문 → 상대) · 고침. 미조치 15 → 원장 |
| 2026-10-19 · `03eb318`(이 노트) | 2절 조치의 **존재 대조**(심볼 grep) · #1o 공유 · #1w 나가기 수신 처리기 열람 | 심볼로 적은 조치는 현존 확인. 심볼 없는 행(SQL 바인드 · LIKE 이스케이프 · 로그에 비밀 없음 · 키 생성 원자성 · `approve` 같은 세대 · `innerHTML` 호출자 전수)은 **1a 결과 인용**(이번에 다시 재지 않음). **#1a 이후 기능(아래 4.2)의 전용 보안 점검은 없다** |

**`--restricted` 실측(1a, claude 2.1.296)** — 작업 폴더 밖 절대 경로 Read · Grep · Glob, 작업 폴더 안 junction → 밖, 사용자 · 프로젝트 스킬,
프로젝트 `UserPromptSubmit` 훅 모두 **거부**(대조 실행 = 플래그 없으면 실행됨). 이 방어는 **CLI 플래그 의미에 기댄다** — Claude Code 업데이트로 조용히
사라질 수 있어 canary(5.1)로 다시 잰다.

## 4. 남은 위험 · 향후 대비

### 4.1 원장의 보안 관련 행 (상태는 [원장](../overview/ledger.md) 이 정본)

| 원장 출처 | 요지 | 상태 | 다시 볼 때 |
|---|---|---|---|
| `#01 R2-F2` · 1a S15 | 출처별 속도 제한 없음 — 여러 IP 로 32 슬롯 반복 점유(LAN DoS) | 조건 대기 | 수신 정책 재검토 · 공용망 사용 시 |
| `#1a A-I1` | 피어 포트 `0.0.0.0` 수신 · 방화벽 규칙은 매뉴얼 안내뿐(공용 Wi-Fi · VPN 에서도 수신) | 수용 | 사내망 밖 배포 시 |
| `#1a A-I3` | `claude -p` 가 앱 환경 변수 상속(셸에서 띄우면 그 셸 비밀) | 수용 | 셸 도구 허용 시 즉시 `env_clear` |
| `#1a A-I4` | 원격 문의 세션을 사용자가 그 폴더에서 `--resume` 하면 가드 없이 이어받음 | 조건 대기 · 확인 필요 | Claude Code 실측 |
| `#1a A-I7` | `cargo audit`(RUSTSEC) 미실행 — 2026-10-19 에도 `no such command: audit` | 조건 대기 | **배포(S6) 전 필수** |
| `#1a A-I8` | 허용 도구 이름 `LS` · `NotebookRead` 가 현행 CLI 에 있는지 | 조건 대기 · 확인 필요 | Claude Code 업데이트마다 |
| `#1a A-I5` | 마이그레이션 `.vN.bak` 영구 보존(지운 대화가 사본에) | 조건 대기 | 보존 정책 결정 시 |
| `#1a A-S12` | 받는 쪽이 건 자동 왕복 멈춤을 연 쪽이 다시 켬 | 수용 | 확실한 차단 = 집중 · 연결 삭제 |
| `#1a S2-C` | Cf 밖 위장 문자(U+2028/2029 · 한글 채움 문자)가 이름에 남음 | 수용 | 이름이 `textContent` 밖에서 쓰일 때 |
| `#1a R1-A1f` | 데이터 폴더 권한 맞추기 실패 시 경고만 · 시작 계속(fail-open) | 수용 | — |
| `#1a R1·R2-U` · `#1n R2-F1u` | Unix 권한 · 강제 종료 소유권을 실기에서 확인 못 함 | 조건 대기 | Phase 4 macOS |
| `#1b S2-EX` | 같은 PC 프로세스가 `127.0.0.1:47100` 선점 가능(`SO_EXCLUSIVEADDRUSE` 없음) — 키 고정이라 방해만 | 수용 | D33 개정 시 |
| `#1b S2-Ping` | `Ping` 이 저장 주소를 연결 출처 IP 로 확인 없이 덮음(다중 NIC · VPN · NAT) | 수용 | 다중 경로 지원 시 |
| `#1n R2-F1w` | Windows 강제 종료가 실행 파일 이름만 비교 | 수용 | — |

### 4.2 #1a 이후 늘어난 외부 입력 — 전용 점검 없음

#1a 점검 이후 아래 피어 프레임 · 경로가 생겼다. 2.1 의 게이트(페어링 · `ids_ok` · 프레임 길이)는 모두 거친다. 이 노트는 **처리기 열람까지만** 했다.

| 기능 | 표면 | 열람 관찰 |
|---|---|---|
| #1b 피어 주소 따라가기 | `Ping{port}` | 페어링된 자기 키 행만 갱신(원장 `#1b S2-Ping`) |
| #1c 보내기 보장 | outbox · 24시간 재전송 | 재전송 대상은 내가 적은 행만 |
| #1d 질의 · 요청 | `ReqState` | 요청은 실행 안 함 |
| #1m 컨텍스트 | `Context`(압축 · 비우기) | `CTX_OPS` 목록 · `MAX_TEXT` 검사 · `conv` · `msg` UUID 꼴 |
| #1o 공유 대화 | `Share` · `SharePost` · `ShareDelete` · `ShareEnd` · `ShareNotice` | 그 상대 사본에만 · 관람자 ≤ 5 · 같은 uuid 가 다른 대화면 거절. **개수 상한 없음**(소스 확인): `inbound_share` · `share::open_copy` · `share::upsert` 는 페어링 상대 하나가 만들 수 있는 사본 수 · 사본당 글 수를 세지 않는다 — 글당 `MAX_TEXT` 만. 1a S8(페어링 상대 대량 전송 → DB 증가)과 같은 부류의 **남은 위험**(원장 미등록) |
| #1w 나가기 | `Leave` | 내 소유 대화만 · 존재 비노출(main 미합류) |

→ **다음 보안 점검의 1순위 범위**는 이 표다. 특히 페어링 상대 기준 **총량 상한**(대화 · 공유 사본 · 글 수 · DB 크기)은 1a S8 이후 대화 본문 · 메모 · 동시 실행만 막았다.

## 5. 다시 점검하는 법

### 5.1 회귀 확인 명령

```powershell
# --restricted canary — 실제 claude · 사용량 사용, Claude Code 업데이트마다
cargo test -p an-agent --test restricted_canary -- --ignored --nocapture
# DLL 심기 방어 — 릴리스 빌드마다. 기대: vcruntime=0 dependent_load_flag=0x800
cargo build --release --workspace
python tools/pe_imports.py target/release/coworkAgent.exe
# 데이터 폴더 권한 — 기대: 사용자 · SYSTEM 2 항목, 상속 없음
icacls "$env:LOCALAPPDATA\coworkAgent"
# 의존 취약점 — 도구 설치 필요(이 PC 미설치)
cargo install cargo-audit; cargo audit
```

`RUSTFLAGS` 환경 변수가 있으면 `.cargo/config.toml` rustflags 가 통째로 무시된다(원장 `#1a S1-F`) — `pe_imports.py` 결과로 판단한다.

### 5.2 새 기능 검토 질문 (외부 입력을 늘리는 Task 의 intent · plan 에서)

1. 새 피어 프레임이 미페어링 상대에게 열리는가 — `node` 요청 분기의 `not_paired` 묶음에 넣었는가.
2. 원격 id 는 `ids_ok` 로 꼴을 거르는가 · 대상은 `(uuid, peer)` 로 그 상대 것만 찾는가.
3. 길이 · 개수 · 누적 **상한**이 있는가(본문 `MAX_TEXT` 만으로는 개수 폭주를 못 막는다).
4. 원격 텍스트가 실행 입력에 들어가면 `with_notes` 틀 · `unframe` 을 거치는가 · stdin 으로만 가는가.
5. 실패 응답이 내부 상세(stderr · 경로 · DB 오류)를 싣지 않는가 — `Failure` 의 정해진 문구.
6. 원격 이름 · 문구가 화면에 닿으면 `clean_name` · `textContent` 를 거치는가 · `innerHTML` 에 원격 값이 없는가.
7. 새 로컬 API 라우트가 `guard`(Host · Origin · Bearer · 보안 헤더) 뒤에 있는가 · GET 은 부작용이 없는가.
8. 원격 실행 도구 · 플래그를 바꾸면 `READ_ONLY_TOOLS` · `claude_args` · canary 를 함께 보는가.
9. 위협 모델(D33)을 바꾸는 결정이면 이 노트 1절 · 4절과 masterPlan D33 을 같이 개정한다.
