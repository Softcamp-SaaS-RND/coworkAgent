# stage 5 — 문서

## 목표

plan "설계 · stage 5" — 매뉴얼의 파일 경로 · 모듈 이름을 새 구조로, PRODUCT · PROJECT 반영.

## 바꾼 것

| 문서 | 바꾼 것 |
| --- | --- |
| [web-ui](../../../manual/dev/web-ui.md) | 머리 안내의 원본 목록(`app.js` · `view.js` → `js/` 모듈), 새 절 "모듈" — 48 모듈 표(파일 첫 줄 주석 그대로) · 진입 순서(`core/token.js` 먼저) · 층 · 공유 상태 규칙, `view.X` 참조 23곳 → `js/logic/<모듈>.js` `X`, `view.js` `X` 5곳 |
| [an_app](../../../manual/dev/an_app.md) | `chat_api/` 하위 모듈 표, 토큰 없이 주는 경로(`/js/{모듈}` — `SCRIPTS` 정확 일치 · 표 밖 401 · 404), `view.X` 24곳 · `view.js` · `app.js` 참조 → `js/logic/` · UI 모듈 이름 |
| [an_core](../../../manual/dev/an_core.md) | `chat/` 하위 모듈 표(공개 경로 `an_core::chat::X` 그대로) |
| [an_agent](../../../manual/dev/an_agent.md) | `chat/` 하위 모듈 표(모듈마다 `impl Chat`) |
| [developer-guide](../../../manual/dev/developer-guide.md) | 저장소 구조 표의 `crates/app/web` 줄, 정적 자산 공개 경로 |
| `PRODUCT.md` | 웹UI 테스트 대상(`js/logic/`) · 변경 신호 로더(`js/logic/sync.js` `loadersFor`) — 다른 세션이 고친 2줄(security-posture 링크)은 건드리지 않고 이 Task 줄만 커밋 |
| `PROJECT.md` | 검증 명령 "코드 정리"(`python tools/check_code_tidy.py`) — stage 4 커밋 |

## 남긴 것(이번 Task 밖)

- [an_app](../../../manual/dev/an_app.md) 에 이미 없어진 함수 이름 5개가 남아 있다 — `keepPicked`(1.0.0) · `groupSection`(#1z) · `settingsGroupSections` · `USER_LIST` · `convCopy`(#1z-8).
  경로를 바꿀 대상이 없어(지금 코드에 없음) 글 그대로 두었다 — 그 절의 기능 설명부터 다시 써야 해서 원장 후보.
- [web-ui](../../../manual/dev/web-ui.md) "문구 규칙" 의 `chat.rs` 는 1p 가 바꾸지 않은 파일 목록(그때 이름)이라 그대로.

## 테스트 결과

- E7 셈: `web=2 agent=1 core=1 app=1`(plan 개정 ④ 의 `chat_api/message.rs`).
- `python tools/check_ddd_docs.py` → PASS · `python tools/check_crates.py` → `crates=4 edges=6 violations=0` · `python tools/check_readme.py` → `readme ok`.
