# stage 3 — 실제 화면 · 문서 · README 사진

## 목표

plan stage 3 — 디버그 빌드 실제 화면 두 테마 확인, README 사진 셋 다시, 매뉴얼 · PRODUCT · README.

## 산출 파일

- `docs/images/workspace.png` · `pairing.png` · `agents.png` — 레일 색 타일 · 그룹 세 사람 · 말풍선 상대 아바타 색(다크, @2x, 시험 데이터 가짜 이름).
- [an_core](../../../manual/dev/an_core.md)(스키마 29 · `SCHEMA_VERSION` 29) · [web-ui](../../../manual/dev/web-ui.md)(레일 · 타일 편집 폼 · 토큰 표) ·
  [사용자 가이드](../../../manual/user/user-guide.md)(레일 · 타일 편집 · 이미지 지우기) · PRODUCT(#1z-15 줄만 인덱스에) · README 1.4.0 줄(스키마 29).
- 측정 스크립트 · 결과는 scratchpad(`ego/1z15/z15.mjs` · `out/z15.json` · 화면 12 장) — 저장소에 넣지 않음.

## 측정(디버그 빌드 · ego-browser 1440 × 900 · 시험 데이터 스키마 28 → 29)

- 스키마 29 로 올리며 있던 상대 셋 · 그룹 둘이 여섯 중 색을 받음(peer-3 · peer-2 · peer-4 / peer-4 · peer-5).
- 라이트 · 다크: 레일 상대 · 그룹 타일 바탕 = 그 색 토큰, 글자 흰색, '나' = 흑연(`--rail-2`), 그룹 그림 원 3(세 사람) · 상대 원 1.
  타일 편집 색 단추 — 상대 · 그룹 6 개(지금 색 눌림), 미리보기 = 지금 색. 연결 목록 머리글자 = 그 색, 말풍선 상대 아바타 = 그 상대 색,
  주고받은 메시지 상대 아바타 = 그 상대 색 · '나' = 무채(색 안 고름).
- 저장 흐름: 상대 1 을 peer-1 로 → 미리보기 · 저장 · 새로 읽기 뒤 레일 peer-1 · 단추 peer-1 눌림. '나' 단추 7 개(흑연 눌림) → peer-2 저장 = 레일 · 프로필 얼굴 peer-2 · API `me_color` peer-2 →
  흑연 저장 = 레일 흑연 · 프로필 무채 · API `null`. 끝에 상대 1 색 되돌림.
- 콘솔 · 페이지 오류 0(측정 · README 사진).

## 설계 결정

- 시험 데이터 DB 는 이 확인으로 스키마 29 가 됐다(scratchpad 사본 — 이전 실행 파일로 열지 않는다).

## 테스트 결과

- 위 측정. README 사진 `readme pop=true errs=[]`.

## 다음 단계

평가(E1 ~ E8 · codex 2 라운드) → 종결 · main → 1.4.0 릴리스 빌드 다시.
