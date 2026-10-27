# report — 1z-14.ui-touchup

종결: 2026-10-26 · 브랜치 `task/1z-14-ui-touchup`(원 작업 폴더 — worktree 없음) → main(로컬 반영, push 는 사용자 확인 뒤).

## 결과

#1z-13(Graphite, 1.3.7 — 배포 전) 뒤 UI 손보기 — 버전 1.4.0(1.3.7 과 함께 내보냄), 기능 · 문구 · DB 스키마 그대로(28).

- stage 1 — 레일 타일 = 무채 네모 + 사람(나 · 상대) · 사람 + 그림자(그룹) 그림(`railFace` — `d86a728` 그림 그대로, 22px · 작게 16px), 타일 편집 미리보기도 같음.
  개요 미리보기 카드 테두리 = 대화 유형 색(에이전트 대화 `--card-agent` 청록 · 내 대화 `--card-mine` 남색 #46607E / 다크 #8AA4C4 · 관전 `--card-watch` 호박), 안 읽음 = 같은 색 2px + "새 글 n".
  사이드바 대화 목록의 내 대화(`out`) 줄 유형 아이콘 없음(`convItem` 셋째 인자 `sideList` — `renderConvList` 만). 시험 · 매뉴얼 · README · 버전 1.4.0.
- stage 2 — 실제 화면 라이트 · 다크 측정(오류 0), README 사진 셋(workspace · pairing · agents) 새 UI.
- 평가 — check_gates 8/8, codex 라운드 1/2 approve 지적 0(라운드 2 는 고칠 것이 없어 돌리지 않음). 미조치 없음.

| 성공 기준 | 판정 | 근거 |
|---|---|---|
| A1 레일 사람 · 그룹 그림, 두 테마 | 충족 | E4(`railFace` 행 · 크기 행), E7 — 나 · 상대 원 1 · 그룹 원 2, 22px · 작게 16px, 두 테마 |
| A2 카드 테두리 유형 세 색 · 안 읽음 2px · 대비 | 충족 | E4 `token_contrast` 82 경우(`--card-mine` × `--bg` · `--surface` × 두 테마 3:1), E7 카드 색 불일치 0 · 안 읽음 폭 2 배 |
| A3 사이드바 내 대화 줄 아이콘 0 | 충족 | E4 조건 행, E7 out 줄 아이콘 0 · shared 줄 눈 그대로 |
| A4 동작 · 문구 그대로 | 충족 | E1 · E2 · E3 통과, E5 `labels n=280 missing=0 added=0`, 바뀐 시험 행은 plan '바꾸는 시험' 대로 |
| A5 1.4.0 · README 사진 · 검증 · 매뉴얼 | 충족 | E6 `crates=4 v140=4`, E7 README 사진 셋, E8 문서 체커 · 매뉴얼 수 |

- 편차 · 한계:
  - E1 은 둘째 실행에서 한 번 rc=101(문서만 바뀐 커밋), 단독 재실행 exit 0 · 셋째 실행 8/8. 원인 찾기 반복 4 번 중 1 번 `cli_tx.rs` `unreachable_peer_stalls_with_code_7_then_wait_resumes` 가
    `os error 10055`(WSAENOBUFS — 소켓 버퍼 · 포트 고갈)로 실패 — 연속 실행 환경 문제로 보이며(추론) 이 Task 변경과 무관, 고치지 않음(report-3). 측정 기록은 [eval](eval.md).
  - 측정 화면의 테두리 계산값은 0.8 · 1.6px(이 PC 화면 배율 125%) — 1px · 2px 가 기기 화소로 맞춰진 값, 비율 2 배로 판정.
  - README 사진은 제품 기본 타일 크기(작게)로 찍음. 멘션 후보는 시험 데이터 상대가 꺼져 있어 화면 상태에 이름 셋을 넣어 띄움(사진용).
  - 미실행: macOS · 두 PC 실연동 · 화면 낭독기 · 서명 빌드(리뷰어 next steps 와 같음).
- 구현자 해석(사용자 미확인):
  - 결정 7 — 1.3.7 미배포라 README 의 1.3.7 줄을 1.4.0 줄로 합침.
  - plan 승인 질문 없이 구현 — 사용자 지시 + 질문 답 U1 ~ U4 가 설계를 정함.
  - 사이드바 '대화' 목록 = `renderConvList`(나 · 상대 · 그룹 사이드바의 대화 절) — 즐겨찾기 · 에이전트 응답 화면은 유형이 섞여 아이콘 그대로.

## 사용자 결정 필요

> **2026-10-27 사용자 결정 — 종결**: 이 절의 남은 항목 · 이 Task 의 구현자 해석은 모두 종결 · 폐기한다 — 사용자 "잔여 이슈는 종결/폐기 처리. 왜냐하면 이전 task에서 결정사항이여도 현재 UI/UX 및 기능을 현재 디자인에 맞춰 개발 완료함." 원장 행이 있으면 원장에서 `수용`.

| 인덱스 | 내용 | 곳 |
|---|---|---|
| report-1 | 1.4.0 릴리스 — origin push(squash) · 릴리스 빌드 서명 · gh release(1.3.7 은 따로 내지 않음) | — |
| report-2 | 내 대화 테두리 남색(#46607E / 다크 #8AA4C4) 실제 화면 확인 — 측정 사진 scratchpad, 바꿀 색이면 토큰 한 줄 | [work/stage2](work/stage2.md) |
| report-3 | `cli_tx` 시험 간헐 실패(`os error 10055`, 반복 실행 4 번 중 1) — 별도 Task 로 살필지 | [eval](eval.md) 측정 기록 |

## 산출

- web: `crates/app/web/js/core/widgets.js`(`PERSON_SVG` · `GROUP_SVG` · `railFace`) · `js/rail.js` · `js/tiles.js` · `js/sidebar.js` · `app.css`(토큰 셋 · 카드 테두리 · 레일 그림 크기).
- 시험: `crates/app/tests/ux_literals.rs`(바뀜 4 · 새 PRESENT 7 · 지운 ABSENT 4 · 새 ABSENT 2 · `token_contrast` 82).
- 문서: [web-ui](../../manual/dev/web-ui.md) · [사용자 가이드](../../manual/user/user-guide.md) · PRODUCT(#1z-14 줄) · README 1.4.0 · README 사진 셋 · masterPlan S4 개정.
- 버전 1.4.0 — `crates/{core,net,agent,app}/Cargo.toml` · `Cargo.lock`.
