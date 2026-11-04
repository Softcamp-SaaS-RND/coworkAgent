# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-11-04 / main — v1.4.5 릴리스(서명 · SHA-256 65a2ce98 · 받은 파일 대조 · real_ 시험) · `target/` 정리

## 현재 작업

없음 — #1z-20.audit-fixes 종결([report](docs/archive/1z-20.audit-fixes/report.md)). 저장소 점검 지적 20 개선 · 원격 실행 도구 · 탐색 · MCP 최대 허용(원장 수용 3) ·
상대별 실행 상한 10 · 개요 미리보기 절 머리 접속 점만, 버전 1.4.5(DB 스키마 30).

- **직전에 한 일**: #1z-19 종결 · v1.4.4 릴리스 → 저장소 점검 → #1z-20(stage 1 ~ 3 · 리뷰 R1 지적 4 · R2 지적 2 수정) → 종결.
- **중단 지점**: 없음 — v1.4.5 게시(`0476238`), `target/` 는 `release` · `rel-1.4.0` 만 남기고 비움(약 243 GB). 다음 `cargo test` 는 대부분 다시 빌드.
- **작업 메모**: 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 끝 2줄 security-posture 링크 · 미추적 docs/tech/security-posture.md)은 커밋에서 뺀다 —
  그 문서 §2.2(`--restricted --strict-mcp-config`)는 지금 코드와 다르다(그 세션에 알릴 것). `cargo fmt` 쓰지 않음, `cargo test` 는 PowerShell 로(`CARGO_TARGET_DIR=target/t15`).
  링커 `LNK1201`(PDB 쓰기 실패)이 나면 그 `.pdb` 를 지우고 다시 빌드 — 간헐 rc 101 의 한 원인(추정).

## 다음 행동

1. 후보: 리뷰 경쟁 지점(상한 · 재개 · 되돌리기) 고정 순서 시험 · 기본 `~/.claude` 토큰 만료가 실제 앱 실행에 주는 영향 확인 · macOS 1.4.4 / 1.4.5 zip ·
   실기기 트레이 "확인할 일" · `os error 10055` 간헐 실패 · 19 acceptance TC(사용자).

- **블로커 / 미해소**: 없음.
