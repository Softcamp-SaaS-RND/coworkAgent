# stage1 — 스키마 4 · 저장

- **적대적 평가 기준선(BASE)**: `d5fadb4` — plan 커밋(자동 진행 위임 + 확인 답, 2026-10-11). 브랜치는 03(종결 전) 위에 쌓았다.

## 목표

[plan](../plan.md) Stage 1 — `agents` · `conversations` · `messages` 표와 `an_core::{agents, chat}`. 대상 gate: E3.

## 산출 파일

- `crates/core/src/{db,agents,chat,lib}.rs` · `crates/core/tests/{store,paths}.rs`

## 설계 결정

- 대화 찾기는 (uuid, 상대) 쌍 — 다른 상대가 같은 uuid 를 대면 UNIQUE 위반으로 거부(남의 대화에 끼어들기 차단).
- 상태 · 작성자는 문자열 열 + Rust enum — 모르는 값은 읽기 오류(조용히 기본값으로 바꾸지 않는다).

## 테스트 결과

- store 3 · core 전체 통과. mutation: 대화 찾기에서 상대 조건 제거 → conversation_by_uuid_per_peer FAIL.

## 다음 단계

Stage 2 — `an-agent` 실행기(인자 · 출력 파싱 · 시간 제한).
