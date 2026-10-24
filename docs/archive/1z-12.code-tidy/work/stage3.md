# stage 3 — 죽은 코드 · 불필요 로직

## 목표

plan "설계 · stage 3" — 어디서도 안 쓰는 코드 · CSS 규칙을 지운다. 동작이 바뀌는 지움(기능 제거)은 하지 않고 후보 표로 보고한다(결정 4).

## 찾는 방법

scratchpad 스크립트(이름 출현 수 — 쓰는 것을 죽었다고 하지 않는 쪽으로 넉넉히 센다):

- Rust(`deadrs.py`): `crates/*/src` 의 `pub` 함수 · 타입 · 상수 · trait 이름이 정의 줄 밖(원본 + 시험, 주석 제외)에 한 번도 안 나오는 것 → 0.
  비공개 항목은 컴파일러(`dead_code` — E2 clippy 깨끗). `#![allow(dead_code)]` 는 시험 공용 모듈 셋(`tests/common` — 시험 바이너리마다 쓰는 도우미가 다름)뿐, 제품 원본엔 없음.
- 웹 JS(`deadjs.py`): 최상위 선언 이름이 자기 선언 밖에서 코드 낱말로 한 번도 안 나오는 것(문자열 · 주석 · import 줄 · `.속성` 제외). 지운 것을 빼고 다시 세어 묶음으로 죽은 것까지.
- CSS(`deadjs.py`): 선택자의 클래스 · id 이름이 JS 모듈 · `index.html` 어디에도 낱말로 없는 것. JS 가 만드는 이름 접두(`act-` · `coworkagent-` · `ico-` · `peer-` · `pm-` · `pm-av-` · `req-` · `side-` · `st-`)는 뺌.

## 지운 것

| 곳 | 항목 | 근거 |
| --- | --- | --- |
| `js/conv.js` | `TRASH_SVG` | 쓰는 곳 0 |
| `js/conv.js` | `redrawMessages` | 부르는 곳 0 |
| `js/core/widgets.js` | `svgButton` | 부르는 곳 0(`innerHTML` 을 쓰는 곳도 하나 줄어듦) |
| `app.css` | `.muted` · `.unread-dot` · `.tag.recv` · `.ro` · `.dot` · `.dot.ok` · `.checks` · `.checks label` · `.state-ok` · `.state-wait` · `.room-preview` · `.link-btn` 규칙, `.mono, .fp` 의 `.fp` | 이름이 JS · HTML 어디에도 없음. `ro` · `room-preview` 는 `ux_literals` 없음 행(JS 쪽 쓰임을 지운 것)이 남긴 규칙 |
| `js/` 18 모듈 | 쓰지 않는 import 23개(줄 5개 통째) | stage 2 의 넉넉한 import 중 코드에서 안 쓰는 것 — 원문 줄로 하나씩 확인: 문자열(`"button"` · `"preview"` · `"refresh"` · 클래스 글 `act-…`), 주석(`DRAG_SLOP` · `fillPrefs` · `showSettings`), `.속성`(`.preview` · `.inlines` · `.uid` · `.pick` 류) |

지운 뒤 남는 CSS 토큰(`--recv` · `--faint` · `--ok` · `--mono`)은 다른 규칙이 계속 쓴다.

## 남긴 것 — 후보(사용자 판단)

| 인덱스 | 후보 | 곳 | 지금 | 남긴 이유 |
| --- | --- | --- | --- | --- |
| stage3-1 | `#p` 주고받은 메시지 화면 | `js/peers.js`(152줄) · `js/logic/inbox.js`(84줄) · `index.html` `#peer` | 메뉴 없음(이전 지시 — 내 활동과 겹침), 주소 `#p` 를 직접 쳐야만 열림(`#p/{id}` 는 그 상대 개요로 넘김) | 지우면 화면 하나가 없어진다 — 동작 변경(결정 4) |
| stage3-2 | 시험만 쓰는 logic 넷 | `roomPreview`(`logic/conv.js`, 시험 8곳 — 대화 줄 미리보기를 지운 뒤 화면에서 안 씀) · `unlinkedReplies`(`logic/message.js`, 2곳) · `searchItems`(`logic/search.js`, 6곳) · `ENGINES`(`logic/agent.js`, 2곳) | 제품 코드에서 안 부름 | plan — 시험 약화로 읽히지 않게 지우지 않고 보고 |

Rust 시험 이음새(`set_watch` · `set_retry` · `set_step_every`)는 plan 대로 남김.

## 불필요 로직

stage 3 에서 읽다가 찾은 것은 없다. stage 4(주석) 서브 에이전트가 돌려주는 "이상한 로직" 목록은 stage 4 work 문서에 두고 판단한다.

## 보완(stage 4 뒤)

stage 4 서브 에이전트가 읽다가 찾은 것 중 동작이 같은 네 곳을 지웠다(나머지는 [stage4](stage4.md) "이상한 로직" 표 — 서명 · 문구 · 시점이 바뀌는 것이 섞여 보고만):

| 곳 | 지운 것 | 근거 |
| --- | --- | --- |
| `js/agents.js` | import `lead` | 쓰는 곳은 `row` 매개변수 `lead`(같은 이름이 가림) — stage 3 분석이 코드 낱말만 봐서 놓침 |
| `js/tiles.js` `renderTilePreview` | `const name = …` | 읽기만 하고 쓰지 않음(부수 효과 없음) |
| `js/sidebar.js` `renderConvList` | `groupEmpty` | `empty` 와 같은 요소(글 · 클래스 같음) — 두 갈래가 같다 |
| `app.css` | 토큰 `--fs-3xs` · `--fs-2xs` | CSS · JS · HTML · 시험 어디서도 안 씀 |

확인: `node --test` 172 통과 · `cargo test -p coworkagent-app` ok 32줄 · 실제 화면 57 단계 다시(수집 오류 0, 서버가 준 `/js/tiles.js` 에 지운 줄 없음 — 새 빌드).

## 확인

- `node --test` → `# pass 172` · `# fail 0`. `cargo test -p coworkagent-app --test web_assets --test ux_literals --test css_tokens` → `test result: ok` 3줄.
- 실제 화면 다시(import 정리로 모듈 평가 차례가 바뀔 수 있어서): stage 2 와 같은 57 단계 — 수집 오류 0, 토큰 없이 `/` 도 같음. 서버가 준 `/js/home.js` 에 `settings/prefs.js` import 없음(새 빌드 확인).
