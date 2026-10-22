# STATE.md — 작업 상태

> **세션 시작 시 가장 먼저 읽는 문서.** 지금 어디서 재개하는지의 단일 좌표 — **매번 덮어쓴다.**
> Task 목록은 폴더가 SSOT(`python tools/check_ddd_docs.py --tasks`), 미조치는 [overview/ledger.md](docs/overview/ledger.md).

- **갱신**: 2026-10-22 / main — 개요 미리보기 말풍선 글자 · 이름 축소 · "미리보기 크기" 가 카드 가로 · 세로를 키움(글자 그대로, 사용자 지시 · Task 밖), 버전 1.3.2(#05-3 포함) · main push

## 현재 작업

없음 — 열린 Task 없음. #05-3 에이전트 "셸 명령 실행 허용" 종결([report](docs/archive/05-3.shell-tool/report.md) — 기본 끔, 켜면 `Bash` · `PowerShell` · workflow 수행용 셸 가드 · `env_clear` ·
켤 때 위험 확인, 읽기 전용 도구 3개 `Read` · `Grep` · `Glob`, 스키마 25). origin/main `7c9dbe2..b16d136` squash push(트리 같음 확인) · 브랜치 `task/05-3-shell-tool` · 작업 폴더 정리 · graphify 코드 갱신.

- **중단 지점**: 없음. 원 작업 폴더의 다른 세션 미커밋(PRODUCT.md 링크 2줄 · docs/tech/security-posture.md)은 그대로 —
  security-posture 2.2 절 "도구 = 읽기 전용 5개 · 셸 없음" 은 #05-3 뒤 사실과 다르다(그 문서 커밋 때 갱신).
- **작업 메모**: `cargo fmt` 쓰지 않음. `cargo test` 는 PowerShell 로(VS 자동 업데이트 중에는 MSVC `link.exe` 가 없어 링크 실패 — 끝난 뒤 다시).

## 다음 행동

1. 1.3.2(#05-3 · 미리보기 축소 포함) 서명 릴리스 여부 결정 — 버전만 올림, 릴리스 미결정. 선생님 확인: 라벨 "셸 명령 실행 허용"(intent 결정 8) · 세션 비움(7) · 압축 요약 셸 없이(9).
2. 19 acceptance TC(사용자) — 19 문서는 `task/19-prototype-acceptance` 브랜치에(main 병합 — 이후 main 은 일반 merge). 19 원장 재측정 · `tools/check_acceptance.py` 번호 형식(1a ~ 1z) 수정 필요.
3. v1.0.0 · v1.0.1 · v1.1.0 · v1.3.0 macOS zip 첨부(보유한 Mac).

- **블로커 / 미해소**: 19 acceptance TC(사용자). 그룹 타일 묶음 머리 "에이전트 대화" vs 메뉴 "에이전트 응답"(#1z-2 report — 사용자 확인 필요). 구현자 해석(사용자 미확인 — 각 report): 1q 결정 7 · 15,
  1r plan 해석 1 · 3 · 결정 3 · 11, 1s 결정 1 · 2, 1z-2 결정 2. D34 개정 문구 "편집 · 삭제는 CLI(· API)" 와 달리 CLI 에 편집 · 삭제 명령 없음(확인 필요).
  #30 미조치 2행 원장 수용(사용자 "수용" — `#30 R2-1` · `#30 (#1y 이어받음)`). #1z-4 미조치 1행 원장 조건 대기(`#1z-4 R1-1` — #1z-5 가 이어받음).
