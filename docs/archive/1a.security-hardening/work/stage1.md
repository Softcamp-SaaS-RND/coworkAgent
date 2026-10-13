# stage1 — 로컬 경계

> 근거: [plan](../plan.md) Stage 1. 사용자 검증은 D28 — 19.prototype-acceptance 에서 일괄.

## 목표

같은 PC 의 샌드박스 · 로컬 서버가 신원 키 · 로컬 API 를 가져가지 못하게 한다 — 데이터 루트 소유자 전용 권한(A1) · 쿠키 → 탭 메모리 토큰 +
`Authorization: Bearer`(A2) · 보안 헤더(A3) · 데이터 루트를 품는 에이전트 폴더 거부(A7) · 정적 C 런타임 + DependentLoadFlags(A13) · 오염된 잠금
이어 쓰기 · CLI 제어 문자 제거(A14). 대상 gate: E1 ~ E5 · E8(이 단계 몫), E9 의 빌드 설정.

## 산출 파일

- `crates/core/src/owner_only.rs` — `restrict(dir)`: Windows 보호 DACL(토큰 사용자 SID · SYSTEM, `FILE_ALL_ACCESS`, 하위 상속) · Unix 0700.
  `crates/core/Cargo.toml` — `windows-sys` 0.61.2(lock 에 있던 판, 기능 `Win32_Security_Authorization` 등).
- `crates/core/src/paths.rs` — `Paths::restrict()`. `crates/app/src/main.rs` — 로그를 연 직후 부르고 실패는 warn.
- `crates/core/src/sync.rs` — `lock(&Mutex<T>)`(오염이면 안의 값 + warn). 원시 `.lock().expect(` 13곳(agent 2 · app notify 6 · net node 5) → `sync::lock`.
- `crates/app/src/server.rs` — `PUBLIC` 정적 경로(GET, Host 검사만) · `/?t=` 틀리면 401 · Bearer 만(+ `/api/events?t=`) · `harden()` 이 모든 응답에
  `X-Frame-Options` · `X-Content-Type-Options` · `Referrer-Policy` · `CSP` · `GET /` `Cache-Control: no-store` · 쿠키 코드 · `COOKIE` 상수 삭제.
- `crates/app/web/view.js` `entryToken` · `DISCONNECTED` / `app.js` 토큰 가져오기 · 주소창 지우기 · `Authorization` · `EventSource` 쿼리 · 401 안내 /
  `app.css` `.disconnected`.
- `crates/app/src/chat_api.rs` — `checked_folder` 가 데이터 루트(지금 · 옛) 검사 · `pub fn folder_blocked`(정규화 · 대소문자 무시).
- `crates/app/src/cli.rs` — `pub fn printable` · 결과 줄 · 실패 사유 둘 다.
- `.cargo/config.toml` — `x86_64-pc-windows-msvc` `+crt-static` · `/DEPENDENTLOADFLAG:0x800`. `tools/pe_imports.py` — import DLL · `DependentLoadFlags`.
- 테스트 — 새 `core/tests/hardening.rs`(3) · `app/tests/web_auth.rs`(11) · `app/tests/hardening.rs`(5) · `view.test.mjs` `entry_token`.
  정리: 삭제 5(`entry.rs` 쿠키 진입 2 · `guard_extra.rs` 쿠키 2 · 정적 자산 인증 1) + `events_api.rs` `cookie_post_without_origin_403`(→ `web_auth`
  `cookie_only_401`) · 나머지 `Cookie` 도우미 → `Authorization: Bearer` · `tests/common` `send` 는 바이트로 읽는다(favicon).

## 설계 결정

- **권한 맞추기는 `ensure` 가 아니라 로그를 연 뒤**(`Paths::restrict`) — plan 해석 11 은 "`Paths::ensure` 가 매 시작 부른다" 였으나 `ensure` 는 로거
  설치 전이라 실패 경고가 사라진다("실패는 드러낸다"). 매 시작 · 실패 시 계속은 그대로.
- **`FILE_ALL_ACCESS` 를 글자로**(`0x001F01FF`) — `GENERIC_ALL` 은 icacls 에 `(GA)` 로 보이고, 상수는 `Win32_Storage_FileSystem` 기능을 하나 더
  끈다.
- **`printable` 은 JSON 출력에도 걸린다** — serde 가 제어 문자를 `\u00XX` 로 이스케이프해 원본에 남지 않으므로 결과가 같다(한 출구).
- **`DependentLoadFlags` 위치** — `IMAGE_LOAD_CONFIG_DIRECTORY64` 오프셋 0x4E(처음 0x9A 로 잘못 읽어 0 이 나왔다 — notepad · 디버그 빌드로 대조).

## 테스트 결과

```text
cargo test -p an-core --test hardening              → test result: ok. 3 passed; 0 failed
cargo test -p coworkagent-app --test hardening      → test result: ok. 5 passed; 0 failed
cargo test -p coworkagent-app --test web_auth       → test result: ok. 11 passed; 0 failed
node --test --test-reporter=tap "crates/app/web/*.test.mjs" → # pass 35 / # fail 0
cargo test --workspace                              → rc=0 (FAILED · panicked 0)
cargo clippy --workspace --all-targets -- -D warnings → Finished
python tools/pe_imports.py target/debug/coworkAgent.exe → imports=15 vcruntime=0 dependent_load_flag=0x800
  (변경 전 릴리스: imports=23 vcruntime=1 dependent_load_flag=0x0)
```

- RED 확인: `core` hardening 은 `owner_only` · `sync` 가 없어 컴파일 실패 → 구현 뒤 3 passed. `app` hardening 은 `folder_blocked` · `printable` 없음.

## 단계 리뷰 (2026-10-13, 분리된 리뷰 에이전트 · diff `a5dc55c..0ec258a`)

High 0 · Medium 2 · Low 4. guard 우회 탐침(HEAD · OPTIONS · `//` · 끝 `/` · 퍼센트 인코딩 · 쿼리 중복) · 헤더(200 ~ 500 · 404 · 405) · `owner_only`
unsafe · 13곳 잠금 치환은 결함 없음. 반영 커밋 `5a3c118`.

| 지적 | 판단 | 처리 |
|---|---|---|
| M · 등록 · 수정 때만 폴더를 거른다 — 1a 이전에 홈으로 등록된 에이전트 · 등록 뒤 바뀐 링크는 그대로 실행 | 사실(명세 밖 결함 — plan 해석 12 는 "등록 · 수정 때") | 규칙을 `an_core::paths::agent_folder_problem` 하나로 옮기고 **실행 직전** 다시 본다 — 막히면 실행 안 함 · 상대에게 정해진 문구. 테스트 `roundtrip` `blocked_folder_refused_at_run` |
| M · `DependentLoadFlags` 는 정적 import 만 — tao · muda 가 `LoadLibraryA("uxtheme.dll")` | 사실(실제 악용 여부는 추정) | `main` 첫 줄 `SetDefaultDllDirectories(LOAD_LIBRARY_SEARCH_SYSTEM32)` |
| L · 권한 맞추기 호출 위치가 plan 과 다름 · 두 번째 인스턴스도 매번 전파 | 사실 | 편차는 위 설계 결정에 기록 · 호출을 주 인스턴스 잠금 뒤로 |
| L · `\\localhost\C$\…` UNC 는 정규화가 로컬과 달라 통과 | 사실(비관리자 접근 여부는 추정) | 네트워크 경로(UNC · `\\?\UNC\`)는 에이전트 폴더로 거부 · `app` hardening 단정 추가 |
| L · 주소창 토큰 지우기가 서버 303 → 클라이언트 `replaceState` 로 옮겨감(스크립트가 안 돌면 남음) | 사실 | 미조치 — 종결 시 원장(`no-referrer` · `no-store` 로 노출 제한) |
| L · `RUSTFLAGS` 환경 변수가 있으면 `.cargo/config.toml` rustflags 가 통째로 무시된다 | 사실 | 미조치 — E9 가 릴리스 산출물을 직접 재므로 드러난다 · 개발자 매뉴얼에 적는다 |

## 다음 단계

▶ stage2 — 원격 경계(실패 공개 문구 · 초대 실패 상한 · id 형식 · 메모 상한 · 서식 문자 · 거절 사유 · 프롬프트 틀).
