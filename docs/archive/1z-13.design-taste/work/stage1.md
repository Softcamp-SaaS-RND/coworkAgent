# stage 1 — 토큰 · 공통 부품 · 단추 자리

## 목표

plan "설계 · stage 1" — Graphite 토큰(라이트 · 다크 · 대입 줄)과 공통 부품(단추 · 탭 · 입력창 · 말풍선 · 세부정보 · 설정 타일 · 프로필 단추 · 개요 미리보기)을
흑연 무채색 + 청록 강조 하나로, 단추 자리(새로 만들기 = 테두리 단추 · 입력창 바 순서 · 상세 단추 줄 = 오른쪽 한 묶음 — P3).

## 산출 파일

- `crates/app/web/app.css`
  - 첫 `:root` — 토큰 표 값(라이트 · 다크), 새 토큰 `--border-strong` · `--primary` · `--primary-hover` · `--on-primary` · `--avatar-bg` · `--avatar-text`(+ `--dark-*` · 대입 줄),
    `--agent` · `--agent-soft` = 강조색 따라감(`--dark-agent*` 지움), 에이전트 아이콘 색 `--agent-tile` · `--peer-2` · `--peer-5` 채도 45 % 아래, `--radius-bubble` 8px.
  - 지움 — `--pv-*` 색 28 · `--dark-pv-*` 28 · 대입 28줄, `--bubble-mine-alpha` · `--bubble-theirs-alpha`, 말풍선 색 층(`.msg[data-tone]` 5 · `::before` 2 · 내 글 링크 `--sent`),
    상대 사람 글 주황 면, 에이전트 글 1.5px 테두리 · `::before` 끔, 내 활동 종류 색(탭 색 3 · 고른 탭 굵기 · 고른 탭 바탕 4 · 칩 3 — 실패만 둠), `.pv-head .st-*` 덮어쓰기 3.
  - 바뀜 — 단추(테두리 `--border-strong` · 올림 = 테두리 `--faint` · 주 단추 = `--primary`), 메인 바탕 `--surface`, 글자 탭(`.tabs` · `.seg` — 바탕 · 테두리 없이 고른 칸 2px 밑줄),
    도구 단추(테두리 없음 · 켬 = 강조 글자), 입력창(`--border-strong` · 그림자 없음 · 바 간격 8px · 키 안내 말줄임 · `.ask-kind label > span` · 고른 칸 = 1px 고리),
    말풍선(사람 글 = `--subtle` 면 · 투명 테두리, 에이전트 글 = `--surface` + `--border`, '에이전트' = 바탕 없는 강조 글자), 세부정보(절 아이콘 `--faint` · 대화 정보 = 아래 선 · 북마크 = 아래 선),
    설정(타일 = 아래 선 · 한 줄 높이 맞춤, 고른 목록 줄 = 옅은 청록 바탕 + 청록 글자, 제목 아이콘 `--muted`), 프로필 단추(알약 · 고리 없음 — 24px 네모 얼굴 + 점 + 이름),
    개요 미리보기(색 = 공용 토큰 — 목록 바탕 흰 면, 카드 흰 면 + 선, 사람 글 회색 면, 에이전트 글 흰 면 + 선, "+n개" 실선).
- `crates/app/web/index.html` — `#new_btn` 의 `primary` 뺌, 입력창 바 = 서식 | 키 안내 · 질의 / 요청 · 보내기, 상세 단추 줄 둘(초대 수락 · 에이전트 2단계) = 빈칸 앞.
- `crates/app/web/js/settings/index.js` — `mdFoot` = 빈칸 앞 · 작업들 · 닫기(오른쪽 끝 한 묶음, 순서 그대로).
- `crates/app/tests/ux_literals.rs` — 바뀐 행 10(미리보기 9 — 종류 색은 임시 `--conv-in` · 설정 제목 아이콘 1), 지운 행 9(말풍선 층 7 · 미리보기 바탕 토큰 2), 새 PRESENT 13 · 새 ABSENT 10,
  `bubble_contrast`(36 경우) → `token_contrast`(두 테마 × 32 짝 + 에이전트 아이콘 6 = 70 경우).

## 설계 결정

- 대화 줄(카드 · 종류 테두리 · 점선)과 대화 머리 종류 선 · 종류 칩은 stage 3 으로 — 줄 카드의 테두리를 먼저 지우면 공유 줄의 `border-style: dashed` 가 기본 굵기(3px) 테두리로 살아난다
  (plan 개정 ②).
- 단추 올림 = 테두리 `--faint`(바탕 바꿈 아님) — 색 바탕 단추(에이전트 아이콘 색 견본 `.swatch` · 레일 타일)가 올릴 때 제 색을 잃지 않게(plan 개정 ④).
- 개요 미리보기 목록 바탕 = `--surface`(흰 면, plan 표의 `--bg` 대신) — 메인 바탕이 흰 면이 됐고, 사용자 지시 "개요의 배경색 조금더 연하게"(#1z-11 뒤)를 그대로 지킨다(plan 개정 ③).
- 질의 / 요청 줄바꿈의 뿌리 — `.ask-kind span` 이 안쪽 아이콘 `span` 에도 걸려 아이콘이 `0 12px` 여백을 받았다. 선택자를 `label > span` 으로 좁히고 칸 여백 8px · 아이콘 여백 0(간격 4px).
  세부정보를 연 1440px 에서 키 안내까지 한 줄에 든다(실측).
- 다크 — 같은 언어(흑연 바탕 · 밝은 회색 주 단추 · 밝은 청록 강조), 대비는 `token_contrast` 가 두 테마 모두 잰다.
- 종류 색 토큰(`--conv-*`)은 stage 3 까지 그대로 — 그 사이 응답 · 문의가 같은 청록으로 보인다.

## 테스트 결과

- RED — 시험을 먼저 고친 뒤 `cargo test -p coworkagent-app --test ux_literals --test css_tokens` → `token_contrast` · `literals_present` · `literals_absent` FAILED(새 행 · 바뀐 행 · `토큰 on-primary`).
- GREEN — 같은 명령 → `ok. 3 passed` · `ok. 5 passed`.
- `cargo test --workspace` → 84 묶음 · 522 passed · 0 failed. `cargo clippy --workspace --all-targets -- -D warnings` → `Finished`(경고 0).
- `node --test --test-reporter=tap "crates/app/web/*.test.mjs"` → `# pass 172` · `# fail 0`.
- `python {TASK}/design/taste.py` → `taste files=50 checks=9 violations=7`(남은 7 = 사람 아이콘 아바타 — stage 2). `labels.py c3568f5` → `labels n=280 missing=0 added=0`.
- 실제 화면(디버그 빌드 + scratchpad `ui1z12\data`, ego-browser 1440 × 900 — 대화 · 개요 미리보기 · 내 활동 · 그룹설정 · 화면 × 라이트 · 다크): 주 단추 바탕 rgb(39, 39, 42) / 다크 rgb(228, 228, 231),
  새로 만들기 바탕 흰 면 / 다크 rgb(24, 24, 27), 입력창 바 한 줄(키 안내 · 질의 · 요청 · 보내기), 상세 단추 줄 [이름 바꾸기][삭제][닫기] 오른쪽 끝.

## 다음 단계

stage 2 — 머리글자 타일 · 아바타(사람 아이콘 아바타 7 → 0).
