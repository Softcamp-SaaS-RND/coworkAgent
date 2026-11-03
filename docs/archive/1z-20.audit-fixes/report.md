# report — 1z-20.audit-fixes

종결: 2026-11-03 · 브랜치 `task/1z-20-audit-fixes`(원 작업 폴더 — worktree 없음) → main squash(1.4.5 — 릴리스 · 서명은 별도 요청).

## 결과

2026-11-03 저장소 점검(4 영역) 지적 20건을 고치거나 결정대로 수용하고, 개요 미리보기 절 머리의 접속 글("…통신")을 지웠다. 내부 회의 결정(결정 6)으로 원격 실행의
도구 · 탐색 · MCP 는 최대 허용 — 경로 금지 · MCP 금지 · Codex 환경 변수 목록은 철회하고 위험 3건을 원장에 수용했다. 버전 1.4.4 → 1.4.5, DB 스키마 30(색인).

- stage 1(보안) — 안내문 사실화("그 위는 읽을 수 없다" 삭제) · 받기 DB 일시 오류 = `retry`(`chat::open_refused`) · 상대별 실행 상한 `PEER_RUN_CAP` 10.
- stage 2(버그 14) — 자동 왕복 재개 · 마크다운 번호 · `*` · 두 번째 실행 기다림(150초) · DB 사본 임시 이름 · 옛 폴더 이어 옮김 · 스키마 30 색인 · 공유 `exists` · 알림 상태 ·
  검증 실패 태그 건너뛰기(`Outcome::Rejected`) · CLI 키 16칸 · `history` 들여 씀 · 초대 되돌림 · 프로토콜 종류 나열.
- stage 3 — 개요 절 머리 점만 · 지울 코드(status JSON · 같은 문자열 시험 · 안 쓰는 인자 · 버전 확인 캐시) · 문서 · 캡처(home · group) · 1.4.5.
- 평가 — check_gates 17/17(마지막 측정 R2 수정 뒤), codex 라운드 1/2 needs-attention 지적 4 → 고침(`034be2b`), 라운드 2/2 needs-attention 지적 2 → 고침(`574d8de`, 상한 — 재리뷰 없음).
  미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 도구 · 탐색 · MCP 최대 허용 그대로 · 안내문 사실 · 위험 원장 수용 | 충족 | E1(claim=0) · E2(runner 19) · E3(원장 3행) |
| A2 실행 대기 중 멈춤 → 재개 → 실행 | 충족 | E4(`stop_while_waiting_then_resume` — 고치기 전 FAILED) · 리뷰 R1-3 · R2-1 · R2-2 수정 |
| A3 지적 3 ~ 20 고침 또는 수용 | 충족 | E5 ~ E9 · E13(지적 4 · 5 = 결정 6 수용 — 원장) |
| A4 개요 절 머리 "…통신" 없음 | 충족 | E10 · E11 · E12(화면 — `.pv-head` 글) |
| A5 회귀 없음 · 1.4.5 | 충족 | E14 ~ E17 |

- 편차 · 한계:
  - 결정 1 · 3(경로 · MCP 금지)은 라운드 2 에서 사용자 결정 6 으로 철회 — 원격 질문은 이 PC 의 신원 키 · 대화 DB · `~/.claude` · `~/.ssh` 를 읽어 받을 수 있다(원장 R0-1 — 실측 `leak=True`).
  - 지적 9 기다림은 plan 의 30초 대신 150초(감시의 업데이트 받기 120초를 덮음).
  - 지적 16 은 원래 `ponytail:` 지름길 주석(알고 받아들인 결정)이었으나 승인된 plan 대로 키 칸으로 바꿈.
  - 리뷰 R1-2 · R1-3 · R2-1 · R2-2 의 경쟁 · 실패 순서는 고정 순서 시험 없이 같은 잠금 · 성공 때만 해제로 보장.
  - 시험 중 간헐 rc 101 — 원인 하나는 링커 `LNK1201`(PDB 쓰기 실패, 디스크 · 권한 아님): 해당 `.pdb` 지우고 다시 빌드하면 통과. `target/t15` 231 GB.
  - macOS 실기 확인 없음(코드는 두 OS 공통).

## 사용자 결정 필요

> **2026-11-03 사용자 결정 — 종결**: 사용자 "종결 진행"(report-1). report-2 · 3 은 답 없음 — 릴리스 · `target` 정리는 하지 않음.

- report-1 — 종결(archive · main squash push) 승인.
- report-2 — 1.4.5 릴리스(서명 · gh release) 여부.
- report-3 — `target/` 231 GB 정리(`cargo clean` — 다음 빌드가 전부 다시 컴파일) 여부.

## 산출

- 코드: `crates/agent/src/{runner.rs, chat/{inbound,context,execute,relay,sharing,tx,mod}.rs}` · `crates/core/src/{db.rs, paths.rs, chat/{conversation,message}.rs}` ·
  `crates/net/src/{node,proto}.rs` · `crates/app/src/{main,watchdog,update,notify,cli,server,tray}.rs` · `crates/app/src/chat_api/{conversation,tx}.rs` ·
  `crates/app/web/js/{home,conv}.js` · `js/logic/markdown.js` · `crates/*/Cargo.toml` · `Cargo.lock`.
- 시험: an-agent `runner` · `capacity`(상한 · Context) · `relay`(재개) · `share`(`exists`) · an-core `store`(`open_refused`) · `paths`(사본 · 옛 폴더 · 색인) · an-net `auto_pair`(초대) ·
  app `autostart`(알림 상태) · `update`(`Rejected`) · `cli`(`history`) · `ux_literals` · `view.test.mjs`(마크다운).
- 문서: [사용자 가이드](../../manual/user/user-guide.md) · [an_agent](../../manual/dev/an_agent.md) · [an_core](../../manual/dev/an_core.md) · [an_app](../../manual/dev/an_app.md) ·
  [web-ui](../../manual/dev/web-ui.md) · [개발자 가이드](../../manual/dev/developer-guide.md) · PRODUCT · README · masterPlan(D31 · S4 · 개정 기록) · [원장](../../overview/ledger.md) · 캡처 home · group.
- 문서 체인: [intent](intent.md) · [plan](plan.md) · [stage1](work/stage1.md) · [stage2](work/stage2.md) · [stage3](work/stage3.md) · [eval](eval.md).
