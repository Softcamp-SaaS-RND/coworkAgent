# stage 2 — 실제 화면 · 문서

## 목표

[plan](../plan.md) stage 2 — 디버그 빌드 실제 화면(두 테마) 확인, 매뉴얼 · PRODUCT · README 반영.

## 산출 파일

- [web-ui](../../../manual/dev/web-ui.md)(대화 — 내가 쓴 글 `ownHuman` · `ownTint` · `--own-*`, 입력창 — 받은 대화 `asksKind`, 대비 경우 102) ·
  [사용자 가이드](../../../manual/user/user-guide.md)(7.2 내 메시지 모양 · 7.3 에이전트 대화 입력창 단추 없음 · 9장 의견 칸 여백) · PRODUCT(#1z-16 줄) · README 1.4.1 줄.
- 측정 스크립트 · 결과는 scratchpad(`ego/1z16-20261027/r01.mjs` · `r08.mjs` · `out/r01.json` · 화면 19 장) — 저장소에 넣지 않음.

## 측정(디버그 빌드 1.4.1 `target\t16` · ego-browser 1440 × 900 · 시험 데이터 = #1z-15 `ui1z15b` 사본 `ui1z16` + 보낸 대화 2 에 CLI 글 1 건)

- **입력창**(라이트 · 다크): 보낸 대화 2 = `.ask-kind` 보임, '요청' 고른 채 받은 대화 11 로 → `.ask-kind` 숨김 · 값 `query`.
- **말풍선**(라이트 · 다크): 대화 2 내 글 5 · 7 = `msg human side-mine own own-rail-2` · 아바타 `person/me-tile`, 면 라이트 `rgb(229,229,229)` · 다크 `rgb(51,51,56)`,
  선 라이트 `rgb(212,212,216)` · 다크 `rgb(63,63,70)`. CLI 글 37 = `own` 없음 · 아바타 없음 · 면 `--subtle`. 상대 글 = `side-theirs` · `person/tinted peer-3` · 면 `--subtle`,
  에이전트 글 = 로봇 아바타 · `--surface`. 받은 대화 11 내 글 36 = `own own-rail-2` · `person/me-tile`.
- **'나' 색 바꾸기**: `PUT /api/settings {me_color: "peer-2"}` 200 → 새로 읽기 뒤 대화 11 글 36 = `own own-peer-2` · 아바타 `person/tinted peer-2` · 면 `rgb(230,237,236)`(= `--own-peer-2`).
  끝에 `me_color: ""` 200 으로 되돌림.
- **고른 글**: 시험 데이터에 실행 기록이 없어 세부정보 '진행' 이 "요청 없음" — 실제 고르기로는 `.picked` 가 붙지 않는다. 글 7 에 `.picked` 를 붙여 판독:
  article 바탕 라이트 `rgb(231,240,239)`(`--accent-soft`) · 말풍선 `rgb(229,229,229)` + 선 `rgb(212,212,216)`, 다크 `rgb(26,43,41)` · `rgb(51,51,56)` + `rgb(63,63,70)` — 갈림.
- **요청 의견 칸**: 대화 11 '수용' → `.req-note textarea` padding `12px 16px`(라이트 · 다크), 요청 화면 '완료' → `12px 16px`. 둘 다 취소(상태 안 바꿈).
- 콘솔 · 페이지 오류 0(모든 판독의 `errs: []`).

## 설계 결정

- 시험 데이터는 scratchpad 사본만 썼다(`COWORKAGENT_DATA_DIR` — 디버그 빌드). 이 PC 의 실제 데이터는 건드리지 않음.

## 테스트 결과

- 위 측정. `python tools/check_readme.py` → `readme ok`.

## 다음 단계

평가(E1 ~ E7 기계 판정 → `eval.md` · codex 적대적 리뷰 최대 2 라운드) → 종결.
