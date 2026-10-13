# 보안 점검 — 1a.security-hardening

> [intent](intent.md) `## 왜` 의 사실 근거. 2026-10-13 main `35d5758` 기준, 읽기 전용 점검 4갈래(로컬 웹 API · 피어 네트워크 ·
> 원격 → 에이전트 실행 · 로컬 저장 · 프로세스) + 실측. 코드는 고치지 않았다.

## 판정 요약

- **Critical · High 0건.** 점검 중 High 로 올라온 1건(작업 폴더 밖 비밀 읽기)은 실측으로 **반박** — `--restricted` 가 막는다(아래 "실측").
- 확인된 결함은 **Medium 3 · Low 12 · Info 9** — 한 PC 한 사용자 전제(아래 "전제 개정") 뒤 **Medium 1(S3) · Low 13 · 제외 1(S6)**. 무게는 "같은 PC 의 다른 주체" 와 "원격 실패 정보 노출" 에 몰려 있다.
- 수용된 위험(D8 개정 — 초대 생성 측 승인 생략 · 주소+코드 직접 추가의 지문 미고정)은 지적에서 뺐다.

## 실측 (claude 2.1.296, 앱과 같은 인자 — `runner.rs` `claude_args`)

| 시도 | `--restricted` 있음 | 대조(없음) | 판정 |
|---|---|---|---|
| 작업 폴더 밖 파일 Read(절대 경로) | 거부(`permission_denials`) | — | 막힘 |
| 작업 폴더 밖 Grep · Glob(절대 경로) | 거부 | — | 막힘 |
| 작업 폴더 안 junction → 밖 파일 Read | 거부 | — | 막힘 |
| 사용자 스킬 `/grill-me` 를 stdin 첫 줄로 | 실행 안 됨 | — | 막힘 |
| 작업 폴더의 프로젝트 스킬 `/pwn` | 실행 안 됨 | **실행됨** | 막힘 |
| 작업 폴더 `.claude/settings.json` 의 `UserPromptSubmit` 훅 | 실행 안 됨 | **실행됨** | 막힘 |
| 데이터 폴더 ACL(`icacls`) | 상속 ACL 만 — 이 PC 는 추가 그룹(샌드박스 사용자)이 `identity.key` · `instance.json` 에 `(RX)` | — | **결함 S1** |

`--help` 원문(2.1.296): `--restricted` "… ignores user, project and local settings files … Also confines the file tools to the working
directories (--add-dir included) …". 방어가 **CLI 플래그 의미에 기댄다** — CLI 가 바뀌면 조용히 사라진다(G1).

## 전제 개정 (2026-10-13 사용자 답)

- **한 PC = 한 사용자** — 원문 "윈도우 다중로그인은 미지원. 즉, 한 pc에서 여러 사용자 환경은 배제함". 다른 로그인 사용자는 위협 모델 밖이다.
  같은 사용자 권한 프로세스도 밖(이미 모든 사용자 파일을 읽는다). 남는 로컬 주체 = 같은 사용자가 띄운 **샌드박스 · 서비스 계정**(이 PC 의 상속 그룹처럼)
  과 **악의 없는 로컬 서버**(요청을 기록하는 개발 서버).
- **Windows 전용 암호화 배제** — 원문 "윈도우, 맥 사용자간에도 통신해야하므로 윈도우 전용 암호화방식은 배제." DPAPI 기각.
- 재평가: S1 Medium → **Low**(샌드박스 · 서비스 주체만) · S2 Medium → **Low**(다른 로컬 리스너 = 샌드박스 또는 기록하는 개발 서버) ·
  S4 그대로 Low · **S6 → 제외**(남은 포트를 차지할 다른 사용자가 없다). Medium 은 **S3 하나**(원격 상대 — 전제와 무관).

## gRPC 검토 — 로컬 API (2026-10-13 사용자 요청 "로컬api 서버 통신을 grpc 적용 검토. 보안과 성능 같이 검토")

로컬 API 의 클라이언트는 **브라우저 웹UI**(라우트 26개 · `fetch` 1곳 · SSE 1곳)와 **CLI** 둘이다.

| 축 | 지금(HTTP/1.1 + JSON · SSE) | gRPC 로 바꾸면 |
|---|---|---|
| 브라우저 지원 | `fetch` · `EventSource` 기본 | 브라우저는 gRPC(HTTP/2 trailer)를 직접 못 쓴다 → **gRPC-Web** 필요: 서버 `tonic-web` + 브라우저 protobuf 런타임 · 생성 코드 → D10 "빌드 없는 정적 JS" 와 충돌 |
| 인증 | 토큰(쿠키 · `Bearer`) + Host · Origin 검사 | gRPC 에 인증은 없다 — 토큰은 metadata(= HTTP 헤더). gRPC-Web 도 `fetch` 라 **Host · Origin · DNS rebinding 검사를 똑같이** 해야 한다. S2 의 해법(헤더 토큰)은 프로토콜과 무관 |
| 전송 암호화 | 루프백 평문(같은 PC) | 루프백에 TLS 를 걸어도 브라우저가 자체 서명 인증서를 믿지 않아 이점 없음 |
| 공격면 | hyper HTTP/1 · serde_json | **늘어난다** — HTTP/2 스택(h2 — 현 `Cargo.lock` 에 없음, HTTP/2 Rapid Reset 류 DoS 표면) · protobuf 파서 · gRPC-Web 변환층 |
| 성능(실측) | 응답 평균 **1.4 ~ 6.6 ms**(새 연결 포함, 최대 21 ms) · 크기 **115 B ~ 9.5 KB**(`status` · `peers` · `conversations` · `search` · `activity`, 실행 중 앱) | 직렬화 수십 µs · 수 KB 절약 = 체감 0. 종단 지연은 `claude -p` 가 지배(가장 단순한 질의도 3.6 s 실측) |
| 실시간 | SSE 서버 푸시 이미 있음 | 서버 스트리밍 — 같은 기능 |
| 빌드 · 배포 | `cargo build` 하나 | `prost-build` 는 `protoc` 필요(이 PC 에 없음, `protox` 로 우회하면 의존 더 추가) · tonic · prost · h2 추가 · 실행 파일(현 7.6 MB) 증가 · 라우트 26개 + 웹UI 통신부 재작성 |

**권장: 기각.** 보안 이득 없음(같은 검사를 다시 해야 하고 공격면은 커진다) · 성능 이득 측정 범위 밖 · D10(빌드 없는 프런트 · `cargo build` 하나)과 충돌.
대신 HTTP + JSON 유지 + 헤더 토큰(S2) + 보안 헤더(S4). 다시 볼 때: 다른 언어 · 툴이 로컬 API 를 많이 부르게 되어 스키마 계약이 필요할 때(S8) —
그때도 OpenAPI 문서가 더 가볍다. 피어 프로토콜(4바이트 길이 + JSON over TLS)은 이 검토 밖.

## 지적

심각도는 "같은 사용자 권한 프로세스는 이미 모든 것을 읽는다" 를 전제로 매겼다. 위치는 점검 시점 줄.

| ID | 심각도 | 요지 | 위치 | 공격 시나리오 | 최소 수정안 |
|---|---|---|---|---|---|
| S1 | Medium | 데이터 폴더(`identity.key` 평문 PKCS#8 · `instance.json` 토큰 · DB · `.bak`)가 **상속 ACL** 에만 기댄다 | `core/paths.rs:115` · `net/identity.rs:84` · `core/instance.rs:58` | 상속된 추가 그룹 · 다른 로컬 주체가 키를 읽어 → 페어링된 모든 상대에게 이 PC 로 위장, 토큰으로 → 로컬 API 전권 | 데이터 루트 생성 시 상속 끊은 DACL(소유자 · SYSTEM) 명시, Unix 는 0700/0600 |
| S2 | Medium | 인증 쿠키 `an_token` 이 **포트를 가리지 않는다**(쿠키 · SameSite 모두 포트 무시) | `app/server.rs:100` | 사용자가 같은 PC 의 다른 `127.0.0.1:포트` 페이지(악성 로컬 서버 · 요청을 기록하는 개발 서버)를 열면 쿠키가 그쪽으로 실린다 → 토큰 획득 → 실행 중 내내 로컬 API 전권. 다른 포트가 같은 이름 쿠키를 심으면 401(UI 막힘) | 쿠키 대신 JS 가 토큰을 헤더로(`Authorization`), 진입 `?t=` 는 주소창에서 지움, SSE 만 쿼리 허용 |
| S3 | Medium | 실행 실패 시 **claude stderr · DB 오류 원문**이 답 본문으로 상대에게 간다 | `agent/chat.rs:1327` ← `runner.rs:330` | 상대가 실패를 유도해 로컬 경로 · 사용자명 · 인증 상태 · 버전을 반복해 얻는다 | 상대에게는 일반 문구, 상세는 로컬 로그 · 내 화면에만 |
| S4 | Low | 보안 헤더 없음(`X-Frame-Options` · CSP `frame-ancestors` · `nosniff`) | `app/server.rs:156-191` | 같은 사이트(다른 로컬 포트) 페이지가 웹UI 를 iframe 으로 띄워 승인 버튼 클릭재킹 | 전 응답에 헤더 3종(본문 HTML iframe 의 `srcdoc` 이 부모 CSP 를 물려받는 점 유의) |
| S5 | Low | 초대 코드 **실패 횟수 무제한** — 틀린 코드는 창을 닫지 않는다(선점 방지 설계) | `net/node.rs:257-273` | 같은 LAN 이 창(5분) 동안 재연결로 코드 추측 — 31^8 공간이라 창당 성공 확률 ≈ 3.5×10⁻⁶(추정, 초당 1만 회 가정). 성공하면 사람 승인 없이 등록(D8 수용 범위 밖) | 출처 IP 별 실패 N 회 뒤 그 IP 거절(창은 유지) |
| S6 | Low | CLI 가 `instance.json` 만 믿는다 — 앱이 비정상 종료해 파일이 남으면 그 포트를 차지한 다른 주체에게 질문을 보낸다 | `app/cli.rs:338-361` · `handoff.rs:29` | 질문 본문 유출 + 가짜 답으로 CLI 를 부른 AI 에이전트에 주입 | CLI 가 `instance.lock` 을 먼저 시도 — 잡히면 "실행 중 아님" |
| S7 | Low | 에이전트 폴더 검증 = "있는 절대 경로" 뿐 | `app/chat_api.rs:274-280` | 홈 · `C:\` 를 폴더로 등록하면 데이터 루트(다른 상대와의 대화 DB · 키)가 원격 문의의 읽기 범위에 들어간다(`--restricted` 는 작업 폴더 **안** 은 막지 않는다) | 데이터 루트를 포함하는 폴더 등록 거부 |
| S8 | Low | 원격 입력 총량 상한 없음 — `conv`/`msg`/`reply_to` 길이 · 형식 미검증(본문만 16 KiB), 실행 전 모으는 메모 수 무제한, 받는 쪽 동시 `claude -p` 수 무제한 | `agent/chat.rs:659,768,1166,1629-1670` · `core/chat.rs:805` | 페어링된 상대가 글 · 대화를 대량으로 → DB 증가 · 프롬프트 폭주 · 동시 실행으로 Max 사용량 소모 | uuid 형식 고정 · 메모 개수/바이트 상한 · (D24 재검토 시) 동시 실행 상한 |
| S9 | Low | 원격 이름의 **서식 문자**(U+202E RLO · U+200B 등 Cf) 통과, 거절 사유 `code` 원문 표시 | `net/proto.rs:196` · `net/node.rs:201-211` | 피어 목록 · 알림에서 다른 PC 이름으로 위장, 키 미고정 주소 추가에서 임의 서버가 오해 유발 문구 표시 | `clean_name` 이 Cf 범위도 제거, `refused()` 에 `clean_code` |
| S10 | Low | 원격 본문이 프롬프트 **맨 앞**(메모 없는 사람 요청은 원문 그대로) · 틀 문구 `[요청 — 이 PC 사용자]` 위장 가능 | `agent/chat.rs:1641-1668` | 프롬프트 주입의 사회공학 효과 증가(내장 슬래시 명령 해석 여부는 확인 필요) | 늘 머리말 + 구분자로 감싸고 가드에 "틀 문구 불신" 추가 |
| S11 | Low | `VCRUNTIME140.dll` 동적 import · DLL 검색 경로 기본값 | 릴리스 exe import 표 | 다운로드 폴더에서 실행 시 같은 폴더에 심은 DLL 이 먼저 로드(DLL 심기) | `+crt-static` · `/DEPENDENTLOADFLAG:0x800` |
| S12 | Low | 받는 쪽 자동 왕복 멈춤을 연 쪽 `Relay{on:true}` 가 해제 | `agent/chat.rs:1024-1050` | 받는 쪽이 멈춰도 상대가 재개 — 확실한 차단은 집중 · 연결 삭제뿐 | 받는 쪽이 건 멈춤은 받는 쪽만 해제, 또는 문구로 고지 |
| S13 | Low | DB `Mutex` 오염 시 재시작 전까지 피어 · 대화 기능 정지 | `agent/chat.rs:194` · `net/node.rs:283` | 가드를 쥔 채 panic 1회(원격 유발 경로는 못 찾음 — 추정) | `PoisonError::into_inner` |
| S14 | Low | CLI 평문 출력이 원격 답의 제어 문자(ANSI · OSC)를 거르지 않는다 | `app/cli.rs:263` | 파이프 소비자 · 디버그 콘솔 화면 조작 | C0 · C1 제어 문자 제거(`\n` `\t` 제외) |
| S15 | Low | 연결 슬롯 고갈 — IP 8개 × 4 연결로 전역 32 슬롯 점유 | `net/node.rs:29-32,311` | LAN 지속 DoS. 기존 원장 `#01 R2-F2`(조건 대기 — Phase 2) | 인증 전 제한 시간 단축 · 원장 유지 |

### Info

| ID | 요지 | 위치 |
|---|---|---|
| I1 | 피어 포트 `0.0.0.0` 바인드, 방화벽 규칙은 매뉴얼 안내뿐(공용 Wi-Fi · VPN 에서도 수신) | `net/node.rs:139` |
| I2 | 진입 URL `?t=` 를 실행 내내 재사용 · macOS/Linux 는 `open` argv 에 토큰 | `app/server.rs:93-109` · `browser.rs:30` |
| I3 | `claude -p` 가 앱 환경 변수를 그대로 상속(`env_clear` 없음) — 셸에서 띄운 앱이면 그 셸 비밀도 | `agent/runner.rs` `run_process` |
| I4 | 원격 문의 세션 기록이 `~/.claude/projects/<폴더>/` 에 남아, 사용자가 그 폴더에서 `--resume` 하면 가드 없이 이어받을 수 있다(확인 필요) | Claude Code 동작 |
| I5 | 마이그레이션 `.vN.bak` 영구 보존 — 지운 대화가 사본에 남는다 | `core/db.rs:444-451` |
| I6 | 인증 전 실패 연결마다 info 로그 — 회전으로 옛 로그가 밀린다 | `net/node.rs:320` |
| I7 | `cargo audit` 미실행(도구 없음) — 291 크레이트 crates.io 출처 · lock 커밋됨 | `Cargo.lock` |
| I8 | 허용 도구 이름 `LS` · `NotebookRead` 가 현행 CLI 에 있는지 확인 필요 | `core/agents.rs` |
| I9 | 실패한 `Approve` 응답(`unknown`/`stale`)으로 키 등록 여부가 드러남 — 키 소유자만 알 수 있어 영향 미미 | `net/node.rs:439` |

## 이미 잘 막힌 것

- **로컬 웹 API**: `127.0.0.1` 동적 포트만 · 토큰 32바이트 난수 · 상수 시간 비교 · Host 정확 일치(`localhost` 도 거부 — DNS rebinding 차단) ·
  Origin 정확 일치(`null` 거부) · 쿠키 요청의 비-GET 은 Origin 필수 · GET 은 전부 부작용 없음 · SSE · 정적 자산도 인증 뒤.
- **원격 텍스트 → 화면(XSS)**: `textContent` · `setAttribute` 만, 마크다운은 정해진 태그 · 링크 글자 그대로, href 는 숫자 id 경로만,
  HTML 본문은 `sandbox=""` iframe + CSP `default-src 'none'`.
- **피어 TLS**: 클라이언트 인증서 필수 · 실제 서명 검증 · TLS 1.3 · ED25519 만 · 페어링 뒤 모든 송신 키 고정 · 미등록 키는 Hello 외 프레임 불가 ·
  Approve/Remove 는 인증된 자기 기록에만 · 프레임 길이 검사가 할당보다 먼저 · 대화는 `(uuid, peer)` 로 찾아 남의 대화에 못 끼어든다.
- **에이전트 실행**: 원격 텍스트는 stdin 으로만(argv 주입 없음) · `claude.exe` 고정이라 `.cmd` 경유(BatBadBut) 없음 · 도구는 등록 · 실행 양쪽에서
  읽기 전용으로 거름 · `--restricted`(위 실측) · `--strict-mcp-config` · 매 실행 가드 프롬프트(스냅샷 끔) · 원격은 세션 · 모델 · 옵션을 못 정한다 ·
  30분 제한 + Job Object 로 자손 종료 · 집중은 두 진입점 모두 검사.
- **저장**: SQL 전부 바인드 · LIKE 이스케이프 · 로그에 본문 · 토큰 · 초대 코드 · 키 없음 · 키 생성 원자적 · 단일 인스턴스는 OS 파일 잠금.
