# report — 1a.security-hardening

종결: 2026-10-13 · 브랜치 `task/1a-security-hardening` → main

## 결과

점검([audit](audit.md) — Critical · High 0, 전제 개정 뒤 Medium 1 · Low 13)에서 찾은 빈틈을 막았다. **로컬 경계**: 데이터 루트를 시작마다 소유자
전용 권한으로(Windows 보호 DACL 사용자 · SYSTEM / Unix 0700 · 0600 · `umask 077`) · 웹UI 인증을 쿠키에서 탭 메모리 + `Authorization: Bearer` 로(쿠키는
포트를 가리지 않는다 — 같은 PC 의 다른 `127.0.0.1` 포트로 토큰이 새던 것) · 모든 응답에 CSP · `X-Frame-Options` · `nosniff` · `no-referrer` · 데이터
루트를 품거나 네트워크 경로인 에이전트 폴더를 등록 · 수정 · **실행 직전** 거부 · 정적 C 런타임 + DependentLoadFlags + `SetDefaultDllDirectories`(DLL
심기) · 오염된 잠금 이어 쓰기 · CLI 출력 제어 문자 제거. **원격 경계**: 실패 답은 상대에게 정해진 문구만(stderr · 경로 · DB 오류 · claude 오류 결과는
내 쪽에만) · 빈 답은 실패로 · 초대 코드는 한 창에서 IP 당 10회 틀리면 그 IP 거절 · 원격 대화 id 는 UUID 꼴만(`invalid`) · 원격 이름의 서식 문자(Cf)
제거 · 상대 거절 사유는 정해진 꼴만 · 실행 입력은 늘 구분자 틀(상대 이름은 `상대(…)`, 원격 글 안의 구분자는 지움, 메모 20개 · 32 KiB). 지금 원격
실행을 지키는 `--restricted`(작업 폴더 밖 Read · Grep · Glob · junction 거부, 프로젝트 훅 · 스킬 미적용)는 실측했고 canary 로 고정했다. 로컬 API 의
gRPC 전환은 검토 후 기각(보안 이득 없음 · 공격면 증가 · 응답 1.4 ~ 6.6 ms 실측 · D10 충돌).

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 데이터 루트 소유자 전용(icacls 2 항목 · macOS 0700/0600) | 충족(Windows) · Unix 는 코드만 | E8 `root_dacl_is_protected_owner_and_system_only` · `existing_and_new_children_follow_root` · E10 이 PC 실데이터 `aces=[2, 2, 2] others=0`(변경 전 `[4, 4, 4] others=6`) · Unix 컴파일 확인 못 함(미조치) |
| A2 쿠키로 인증 안 됨 · 헤더 토큰으로 기능 그대로 | 충족 | E4 `cookie_only_401` · `api_needs_bearer` · `events_accepts_query_token` 등 11 · E3 `entry_token` · E12 실화면 |
| A3 보안 헤더 3종 · HTML 본문 보기 그대로 | 충족 | E4 `security_headers_on_every_response` · `csp_and_no_store_on_page` · E12(CSP 위반 0 · data: 그림 · 인라인 스타일) |
| A4 상대에게 stderr · 경로 · 오류 원문 없음 · 내 쪽에 원인 | 충족 | E6 시간 초과 · 버전 · 그 밖 · 기록 오류 · `is_error` 결과(개정 1) · `roundtrip` 빈 답 · 폴더 막힘 |
| A5 IP 별 초대 코드 실패 상한 · 다른 IP 정상 | 충족 | E7 `invite_guesses_capped_per_ip` · `nine_wrong_then_right_pairs` · `capped_ip_does_not_block_others` · `new_window_resets_count` |
| A7 데이터 루트를 품는 폴더 등록 · 수정 거부 | 충족 | E5 · 실행 직전 재검사(`blocked_folder_refused_at_run`) · 네트워크 경로 거부 |
| A8 원격 id UUID 꼴 · 메모 상한 | 충족 | E7 `malformed_ids_rejected_from_paired_peer` · E6 `notes_capped_by_count` · `notes_capped_by_bytes` |
| A9 이름 서식 문자 제거 · 거절 사유 정해진 꼴 | 충족 | E7 `names_drop_format_chars` · `refused_reason_is_cleaned` |
| A10 원격 본문은 늘 머리말 뒤 구분자 안 | 충족 | E6 `body_framed_without_notes` · `notes_framed_before_request` · `frame_markers_stripped_from_remote_text` |
| A11 `--restricted` canary | 충족 | E11 시도별 거부(Read · Grep · Glob · junction) · 대조 실행이 직접 · 링크로 읽고 훅이 돎(`control_read=1` · `control_hook=1`) |
| A12 기존 테스트 · 웹UI 테스트 · 경고 0 | 충족 | E1 · E2 · E3(35) |
| A13 릴리스 exe VCRUNTIME 동적 import 없음 | 충족 | E9 `imports=15 vcruntime=0 dependent_load_flag=0x800`(변경 전 `23 · 1 · 0x0`) |
| A14 CLI 제어 문자 · 오염된 잠금 | 충족 | E5 `printable_strips_control_chars` · E8 `poisoned_lock_keeps_working` · 원시 잠금 0(도우미 17) |

- 적대적 평가 2 라운드(상한, codex-review 세션) — 라운드 1: high 1(`is_error` 결과 원문이 상대로 — 사실 · 고침, plan 개정 1) · medium 3(빈 성공 답 ·
  A1 fail-open · Unix 0600 · canary 가 거부 1건만 요구 — 빈 답 · Unix · canary 고침, fail-open 은 설계 유지), 라운드 2: medium 3(라운드 1 canary 수정이
  junction 생성을 잘라 헛통과 — 고침 · Unix 새 파일 0600 — `umask 077` · 여러 줄 `.lock()` `.expect(` 가 E8 을 빠져나감 — 고침 · CHECK 보정). 미조치
  15건은 [eval.md](eval.md) → 원장.
- stage 리뷰 2회(서브에이전트, 읽기 전용) — Stage 1(M 2 · L 4): 실행 직전 폴더 재검사 · `SetDefaultDllDirectories` · 권한 맞추기는 주 인스턴스만 ·
  네트워크 경로 거부 — 반영. Stage 2(M 2 · L 5): 상대 이름 틀 · 초대 셈 원자성 · 예상 밖 응답 내용 · 가드 문장 · 메모 줄 들여쓰기 — 반영.
- plan 개정 1건(위임): E6 9 → 10 · 해석 5(`is_error` 도 정해진 문구) · 해석 16(canary 시도별 + 대조 읽기) · Unix 파일 0600.
- 위협 모델 전제(사용자 결정): 한 PC = 한 사용자 · OS 전용 암호화 배제 → masterPlan D33.

## 산출

- `an-core`: `owner_only::restrict` · `sync::lock` · `Paths::restrict` · `paths::protected_roots` · `agent_folder_problem` · `folder_blocked` ·
  `InstanceInfo::write`(Unix 0600).
- `an-net`: `MAX_CODE_MISSES` · 창별 IP 실패 셈 · `proto::is_format` · `is_uuid` · `Request::ids_ok` · 오류 코드 `invalid` · `refused` 정리.
- `an-agent`: `Failure` · 공개 실패 문구 7종 · 빈 답 실패 · 빈 에이전트 답 프레임 거절 · `with_notes` 틀 · `MAX_NOTES` · `MAX_NOTES_BYTES` · `unframe` ·
  실행 직전 폴더 검사 · `GUARD_PROMPT` 한 문장.
- `coworkagent-app`: guard(정적 공개 · Bearer 만 · `/api/events?t=` · 보안 헤더 · `CSP`) · 쿠키 삭제 · `cli::printable` · `main` DLL 검색 · Unix `umask` ·
  웹UI 탭 메모리 토큰 · 끊김 안내.
- 도구 · 설정: `.cargo/config.toml`(crt-static · DependentLoadFlags) · `tools/pe_imports.py` · `crates/agent/tests/restricted_canary.rs`(`#[ignore]`).
- 문서: 매뉴얼(user · developer · 모듈 4) · README 통신 표 · `docs/images/comm-overview.png` · PRODUCT 도메인 규칙.

## 남은 것

- 미조치 지적 — [eval.md](eval.md) `## 미조치 지적` → 원장(fail-open 설계 · Unix 미확인 · 위장 문자 범위 · 점검 Info).
- 19 인수 TC — 새 탭 · 다시 켠 앱의 "연결이 끊겼습니다" 안내 · CSP · 실행 실패 문구(상대 쪽) · 데이터 폴더 권한(icacls).
- 다음 Task — [1b.inbound-capacity](../1b.inbound-capacity/intent.md)(대기열 · 처리 중 안내 · 포트 관리), 그 뒤 처리경과 콜백 Task.
