# plan — 00.skeleton

> 근거: [intent](intent.md)(승인 2026-10-11). **구현자는 이 문서만 보고 일한다** — intent 의 성공 기준·결정·제외를
> 아래에 옮겼다. 프로젝트 결정 번호(D·S)는 [masterPlan](../../masterPlan.md).

## 목표

단일 실행 파일 `agentNetwork` 가 트레이에 상주하며(S2) 로컬 웹UI(D9)를 띄우고 사용자 경로(D15)에 SQLite·로그를
여는 실행 골격. 01.pairing · 02.agent-roundtrip 이 이 위에 올라선다.

### 성공 기준(intent 사본)

- A1: Windows x64 에서 릴리스 빌드가 실행 파일 1개를 만들고, 그 밖의 실행 파일을 함께 배포하지 않는다.
- A2: 보유한 Mac 에서 macOS universal(arm64 + x64) 실행 파일 1개가 빌드되고 실행된다.
- A3: 실행하면 트레이(메뉴 막대) 아이콘이 뜨고(macOS 는 Dock 아이콘 없이), 트레이 메뉴 "열기" 가 기본 브라우저에 상태
  페이지(버전 · 데이터 경로 · DB 상태)를 열며 "종료" 가 프로세스를 끝낸다 — Windows · macOS 양쪽.
- A4: 웹UI 서버는 127.0.0.1 에만 바인딩하고, 토큰이 없거나 틀린 요청과 Host/Origin 이 맞지 않는 요청을 거부한다.
- A5: 첫 실행 시 D15 경로에 폴더 · SQLite 파일 · 로그 파일이 생기고, 실행 파일이 있는 폴더에는 아무것도 쓰지 않는다.
- A6: 이미 실행 중일 때 다시 실행하면 새 서버를 띄우지 않고, 기존 인스턴스가 웹UI 를 연 뒤 새 프로세스는 종료한다.
- A7: 루트 `PROJECT.md` 검증 명령의 테스트 · 경고 0 행이 통과한다.

### intent 결정 · 제외(사본)

- T1 트레이 메뉴는 "열기" · "종료". T2 첫 화면은 상태 페이지(버전 · 데이터 경로 · DB 상태). T3 D15 경로 아래 파일
  로그(크기 회전). T4 macOS 는 Dock 아이콘 숨김.
- 제외: 신원 · 내 정보 보기 · 피어 · 페어링 · TLS(01) / 에이전트 · `claude -p` · 메시지 · 이력(02) / `chat [--peer]` 인자 ·
  자동 실행 · 자동 업데이트(Phase 3) / 슬랙형 대화 UI(Phase 1) / 코드 서명(D12).

## 전제

- 의존 첫 다운로드는 이 PC 에서 `CARGO_HTTP_CHECK_REVOKE=false` 가 필요하다 —
  [환경 노트](../../tech/windows-crates-io-tls-revocation.md). 이후 `Cargo.lock`(커밋) + 로컬 캐시로 빌드된다.
- gate CHECK 는 저장소 루트에서 Windows cmd.exe 로 돌고 각각 600초 제한이다. E1 · E2 · E3 은 빌드 프로필이 서로
  달라 한 번 데운 뒤 판정한다.
- 검증 환경: Windows 11 · rustc 1.95.0 `x86_64-pc-windows-msvc` · Python 3.12. macOS 는 사용자가 보유한 Mac.

## 설계

### 크레이트 — Cargo workspace(루트 `Cargo.toml`, 멤버 `crates/*`)

| 크레이트 | 종류 | 담는 것 | 내부 의존 |
|---|---|---|---|
| `an-core`(`crates/core`) | lib | 데이터 경로 · SQLite 열기·스키마 버전 · 파일 로그(회전) · 단일 인스턴스 잠금·핸드오프 정보 | 없음 |
| `agentnetwork-app`(`crates/app`) | bin `agentNetwork` | 트레이 · 이벤트 루프 · 로컬 웹UI 서버 · 브라우저 열기 · 핸드오프 | `an-core` |

- 방향은 app → core 단방향 — core 는 UI · HTTP · 트레이를 모른다. Layer 어휘: core = base, app = app.
- `tools/check_crates.py` — `cargo metadata --format-version 1 --no-deps` 로 워크스페이스 크레이트와 내부 간선을 실측해
  Layer 역방향 · 미분류 크레이트를 잡는다. 출력 한 줄 `crates=N edges=M violations=V`(대상 수와 위반 수를 함께 단정).
  루트 `PROJECT.md` 검증 명령 "의존 경계" 행의 실제 명령이 된다(종결 시 반영).
- bin 이름 `agentNetwork` 는 `cargo clippy -- -D warnings` 를 통과한다(lint 없음 — 실측).

### 외부 의존 — 2026-10-11 crates.io 최신 안정판, Windows 에서 함께 빌드·실행 실측

| crate | 버전 | 용도 | 실측 |
|---|---|---|---|
| tray-icon | 0.26.1 | 트레이 아이콘·메뉴(muda 재노출) | `Icon::from_rgba` 컴파일 |
| tao | 0.37.1 | 메인 스레드 이벤트 루프 · macOS `set_dock_visibility` · `set_activation_policy`(T4) | 컴파일 · docs.rs 확인 |
| axum | 0.8.9 | 로컬 웹UI 서버 | 컴파일 |
| tokio | 1.53.2 | 런타임(`rt-multi-thread` · `net` · `macros`) | 컴파일 |
| rusqlite | 0.40.2 `bundled` | SQLite 를 실행 파일에 포함 — 시스템 라이브러리 의존 없음(D2) | SQLite 3.53.2 빌드·실행 |
| dirs | 7.0.0 | D15 경로 `data_local_dir()` | `%LOCALAPPDATA%` 반환 |
| getrandom | 0.4.3 | 토큰 난수 `fill` | 실행 |
| log | 0.4.34 | 로그 파사드(파일 로거는 자체 구현) | 컴파일 |

표준 라이브러리로 하는 것: 파일 잠금 `File::try_lock`(Rust 1.89+, 같은 파일의 두 번째 핸들은 실패 — Windows 실측) ·
브라우저 열기(Windows `rundll32 url.dll,FileProtocolHandler {url}` · macOS `open {url}`) · 핸드오프 HTTP 요청(`TcpStream`) ·
정적 자산(`include_str!`) · 로그 파일 회전.

### 데이터 경로(D15 · A5)

- 루트 = `dirs::data_local_dir()` / `agentNetwork` → Windows `%LOCALAPPDATA%\agentNetwork` · macOS
  `~/Library/Application Support/agentNetwork`.
- 루트 아래: `agentNetwork.db` · `logs/agentNetwork.log`(+ `.1`) · `instance.lock` · `instance.json`.
- `an-core` 의 경로 API 는 루트를 인자로 받는다 — 운영은 위 루트, 테스트는 임시 폴더. 환경변수 · 플래그 스위치를 두지
  않는다(실경로 우회 금지).
- 쓰기는 전부 이 루트 아래 — 실행 파일 폴더 · 현재 폴더에 쓰지 않는다.

### SQLite

첫 실행에 파일을 만들고 `PRAGMA user_version = 1` 로 스키마 버전을 남긴다. 재실행은 그대로 연다(버전 1 유지).

### 로그(T3)

- `log` 파사드 + 자체 파일 로거. 이번 기록 뒤 크기가 `LOG_ROTATE_BYTES = 1_048_576` 을 **넘게 되면** 먼저
  `agentNetwork.log` → `agentNetwork.log.1`(1개 보관)로 바꾸고 새 파일에 쓴다. 기록 뒤 크기가 정확히 1_048_576 이면
  유지한다.

### 단일 인스턴스(D14 · A6)

- 시작 시 `instance.lock` 을 `try_lock`. 성공 = 주 인스턴스 — 웹서버 포트 · 토큰을 `instance.json` 에 쓴다.
  실패 = 이미 실행 중 — `instance.json` 을 읽어 `POST /api/open`(헤더 `Authorization: Bearer {token}`)을 보내고 종료한다.
- 잠금은 프로세스가 죽으면 OS 가 푼다. 남은 `instance.json` 은 다음 주 인스턴스가 덮어쓴다.
- 신뢰 경계: 같은 OS 사용자로 도는 프로세스는 `instance.json` 의 토큰을 읽을 수 있다(사용자 데이터 폴더 권한에 기댄다).
  다른 OS 사용자 · 원격은 이 파일을 읽지 못한다(쿠키 경로의 예외는 로컬 웹UI 절).
- 두 번째 실행의 요청은 연결 · 쓰기 · 상태 줄 읽기 전체에 5초 상한을 둔다(개정 기록 2).

### 로컬 웹UI(D9 · D10 · A4 · T2)

- 백그라운드 스레드의 tokio 런타임 + axum, `127.0.0.1:0` 바인딩(동적 포트 — 재실행 · 여러 OS 사용자 충돌 회피).
- 토큰: 실행마다 `getrandom` 32바이트 → hex 64자.
- 진입: `http://127.0.0.1:{port}/?t={token}` → 쿠키 검사보다 먼저 처리해, 토큰이 맞으면 `HttpOnly; SameSite=Strict` 쿠키를 설정하고 `/` 로
  리다이렉트한다(주소창에서 토큰 제거).
- 모든 요청에 차례로: Host 가 `127.0.0.1:{port}` 가 아니면 403 · Origin 헤더가 있는데 `http://127.0.0.1:{port}` 가 아니면
  403 · 쿠키나 Bearer 토큰이 없거나 틀리면 401.
- 경로: `/`(상태 페이지) · `/app.js` · `/app.css` · `/api/status`(JSON: `version` · `data_dir` · `db`) · `/api/open`(POST —
  핸드오프, 브라우저 열기 1회).
- 상태 페이지는 `/api/status` 결과를 `textContent` 로만 그린다(HTML 주입 없음).
- 브라우저 열기는 함수 인자로 주입한다 — 운영은 OS 열기, 테스트는 호출 횟수를 세는 대역.
- 신뢰 경계: 브라우저 쿠키는 포트를 구분하지 않는다 — 같은 PC 의 다른 OS 사용자가 127.0.0.1 의 다른 포트에 띄운
  페이지로 `an_token` 을 받아 갈 수 있다. 이 위협은 막지 않는다(사용자 결정, 개정 기록 1).

### 트레이(S2 · T1 · T4)

- 메인 스레드 tao 이벤트 루프 + tray-icon. 메뉴 "열기"(→ 토큰 URL 로 브라우저) · "종료"(→ 서버 정지 · 프로세스 종료).
  아이콘은 코드로 만든 RGBA(파일 자산 없음).
- macOS: 이벤트 루프 실행 전 `set_dock_visibility(false)` + activation policy Accessory.
- Windows 릴리스 빌드는 `windows_subsystem = "windows"`(콘솔 창 없음), 디버그 빌드는 콘솔 유지.
- tray-icon 이벤트는 `EventLoopProxy` 로 이벤트 루프에 넘긴다(tray-icon 문서 권장). 실동작은 미실측 — E10 · E11 에서 본다.

### macOS universal(A2) — 보유한 Mac 에서

```bash
rustup target add aarch64-apple-darwin x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
mkdir -p target/universal
lipo -create -output target/universal/agentNetwork \
  target/aarch64-apple-darwin/release/agentNetwork target/x86_64-apple-darwin/release/agentNetwork
lipo -archs target/universal/agentNetwork   # 기대: x86_64 arm64
```

## Stage

| stage | 목표 | 산출 | gate |
|---|---|---|---|
| 1 | workspace + `an-core` + `tools/check_crates.py` | `crates/core` · 테스트(paths · log_rotate · instance_lock) · `Cargo.lock` | E5 · E6 · E7 · E12 |
| 2 | 로컬 웹UI 서버 + 상태 페이지 | `crates/app` 서버 · `web/` · 테스트(local_ui · handoff) | E4 · E8 |
| 3 | 트레이 · 핸드오프 배선 · Windows 릴리스 | `crates/app` main · 트레이 | E1 · E2 · E3 · E10 |
| 4 | macOS universal · 실동작 | 코드 변경 없음이 목표 | E9 · E11 |

- 각 stage 는 TDD(RED 를 먼저 눈으로 본다) · `work/stage{M}.md`(목표 / 산출 파일 / 설계 결정 / 테스트 결과 / 다음
  단계) · stage 리뷰 · 사용자 검증.
- 개수를 단정하는 gate(E4~E8)의 테스트 이름은 아래 제목에 적은 그대로 둔다. 그 밖의 테스트는 다른 파일에 둔다.

## 평가 항목

- [ ] E1: A7 — 워크스페이스 테스트 전부 통과
  CHECK: cargo test --workspace
  EXPECT: /test result: ok\. [1-9][0-9]* passed; 0 failed/
  EVIDENCE: pending
- [ ] E2: A7 — 우리 코드 경고 0(clippy -D warnings)
  CHECK: cargo clippy --workspace --all-targets -- -D warnings
  EXPECT: Finished
  EVIDENCE: pending
- [ ] E3: A1 — Windows 릴리스 빌드의 실행 파일이 agentNetwork.exe 하나
  CHECK: cargo build --release --workspace && python -c "import glob,os;fs=sorted(os.path.basename(f) for f in glob.glob('target/release/*.exe') if os.path.isfile(f));n=','.join(fs);print(f'exes={len(fs)} names={n}')"
  EXPECT: exes=1 names=agentNetwork.exe
  EVIDENCE: pending
- [ ] E4: A4 — 로컬 웹UI 경계 6건(loopback_only · no_token_401 · wrong_token_401 · wrong_host_403 · cross_origin_403 · valid_200)
  CHECK: cargo test -p agentnetwork-app --test local_ui
  EXPECT: test result: ok. 6 passed; 0 failed
  EVIDENCE: pending
- [ ] E5: A5 — 데이터 경로 2건(layout_created_under_root · db_user_version_1_kept_on_reopen)
  CHECK: cargo test -p an-core --test paths
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E6: A5 — 로그 회전 경계 4건(limit_minus_1_kept · limit_exact_kept · limit_plus_1_rotated · limit_constant_is_1048576)
  CHECK: cargo test -p an-core --test log_rotate
  EXPECT: test result: ok. 4 passed; 0 failed
  EVIDENCE: pending
- [ ] E7: A6 — 인스턴스 잠금 3건(first_is_primary · second_while_held_is_secondary · reacquire_after_release)
  CHECK: cargo test -p an-core --test instance_lock
  EXPECT: test result: ok. 3 passed; 0 failed
  EVIDENCE: pending
- [ ] E8: A6 — 핸드오프 2건(valid_token_opens_once · wrong_token_401_opens_none)
  CHECK: cargo test -p agentnetwork-app --test handoff
  EXPECT: test result: ok. 2 passed; 0 failed
  EVIDENCE: pending
- [ ] E9: A2 — 보유한 Mac 에서 universal 빌드 · 실행, 같은 Mac 에서 cargo test --workspace 통과(수동 — 누가·언제 / lipo -archs 출력 / 테스트 결과 줄 / macOS 버전·칩)
  EVIDENCE: pending
- [ ] E10: A3·A5·A6 — Windows 실동작(수동): 빈 폴더에 릴리스 exe 하나만 두고 실행 → 트레이 아이콘 · "열기" 로 상태 페이지(버전 · 데이터 경로 %LOCALAPPDATA%\agentNetwork · DB) · 두 번째 실행은 브라우저만 열고 agentNetwork 프로세스 1개 · exe 폴더 파일 1개 유지 · 데이터 폴더에 db · logs · "종료" 뒤 프로세스 0개
  EVIDENCE: pending
- [ ] E11: A3·A5·A6 — macOS 실동작(수동): E10 과 같은 순서 + Dock 아이콘 없음 + 데이터 경로 ~/Library/Application Support/agentNetwork
  EVIDENCE: pending
- [ ] E12: 의존 경계 — 워크스페이스 크레이트 2개 · 내부 간선 1개(app → core) · 위반 0
  CHECK: python tools/check_crates.py
  EXPECT: crates=2 edges=1 violations=0
  EVIDENCE: pending
- [ ] E13: A6 — 핸드오프 열기 실패 1건(browser_failure_is_500_not_204)
  CHECK: cargo test -p agentnetwork-app --test handoff_failure
  EXPECT: test result: ok. 1 passed; 0 failed
  EVIDENCE: pending

## 종결 시 할 일

- 루트 `PROJECT.md` 검증 명령: 테스트 · 경고 0 행을 "도입 예정" 에서 활성으로, 의존 경계 행 명령을 `python tools/check_crates.py` 로.
- 루트 `PRODUCT.md`: 핵심 의존 · 빌드 · 모듈 Layer 표 · 개발된 모듈 표(실측).
- `docs/manual/dev/` 에 `an_core` · `agentnetwork-app` 매뉴얼 등록(+ 라우팅 표).
- masterPlan `## 개정 기록` 에 `#00` 한 줄.
- `eval.md`: gate 원장 → 적대적 평가(BASE = plan 승인 커밋, 최대 2 라운드, 분리된 codex 리뷰 세션) → `## 미조치 지적` →
  미조치 원장.

## 개정 기록

1. 2026-10-11 · 설계 "로컬 웹UI" 신뢰 경계 — 쿠키의 포트 비격리로 같은 PC 의 다른 OS 사용자가 토큰을 받을 수 있음을
   명시하고 막지 않는다. 사유: stage 2 리뷰 지적, 사용자 선택 "쿠키 유지"(승인 2026-10-11). gate 항목 · EXPECT 변경 없음.
2. 2026-10-11 · 설계 "로컬 웹UI" · "단일 인스턴스" — 진입 `/?t=` 를 쿠키 검사보다 먼저(쿠키가 있어도 주소창에서 토큰
   제거), 핸드오프 요청 전체 5초 상한. 사유: stage 2 리뷰 지적(plan 원래 의도의 구현 보정). gate 항목 · EXPECT 변경 없음.
3. 2026-10-11 · 평가 항목 E13 추가 · 설계 "로컬 웹UI" — 브라우저 열기 실패는 `POST /api/open` 500, 두 번째 실행은 5xx 를
   재시도 없이 실패 종료. 브라우저 열기는 Windows `ShellExecuteW` 반환값 · macOS `open` 종료 상태로 판정. 시작 실패는 Windows
   릴리스에서 메시지 상자로 보인다. 사유: 적대적 평가 라운드 1·2 지적. 승인: 자동 진행 위임(masterPlan).
