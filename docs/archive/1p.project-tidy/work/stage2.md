# stage2 — 죽은 코드 · 중복 도우미 · 미사용 의존 · 문구 시험 표

> 근거: [plan](../plan.md) Stage 2(해석 8 · 9) · 성공 기준 A3.

## 목표

동작을 바꾸지 않고 코드를 줄인다 — `notice_for` 삭제 · `db_err` 한 곳 · `an-agent` 의 `serde` 의존 빼기 · `ux_literals.rs` 를 표로(시험 6개 이하,
잠근 단정 수 전후 같음). 대상 gate: E3 · E10 · E11.

## 산출 파일

- `crates/core/src/db.rs` — `pub fn db_err(e: Error) -> std::io::Error`(`io::Error::other`) 한 곳.
- `crates/agent/src/chat.rs` · `crates/app/src/notify.rs` · `crates/net/src/node.rs` — 로컬 `fn db_err` 를 지우고 `use an_core::db::{Connection, db_err};`.
  호출부(`map_err(db_err)`)는 이름 그대로.
- `crates/app/src/notify.rs` — `pub fn notice_for` 삭제(참조 0 — 호출부는 모두 `notice_with`).
- `crates/agent/Cargo.toml` — `serde`(derive) 줄 삭제, `Cargo.lock` 의 an-agent 의존 목록에서 `serde` 한 줄 빠짐.
- `crates/app/tests/ux_literals.rs` — 806줄 · `#[test]` 24개 → 528줄 · 5개. 표 셋 `PRESENT`(258행) · `ABSENT`(77행 — 파일별로 펴면 132) ·
  `COUNTS`(3행)와 시험 `literals_present` · `literals_absent` · `literal_counts`(개수 표 + '받은 글' 규칙 + 편집 단추가 입력창 위) · `invite_text` ·
  `bubble_contrast`. 행의 셋째 칸은 출처(Task · 무엇) — 실패 메시지에 찍힌다.
- [design/gates.py](../design/gates.py) `ux <sha>` — 옛 시험과 새 표에서 (있음 · 없음 · 개수, 파일, 문구) 짝을 뽑아 비교한다(해석 9 의 "전후 같음" 증거).

## 설계 결정

- **`db_err` 는 `an_core::db`**: `Error` 가 `rusqlite::Error` 재수출이라 `From<Error> for io::Error` 를 구현할 수 없다(고아 규칙). 함수 하나를
  `pub` 으로 두고 세 크레이트는 `use` 만 바꿨다 — 호출 268곳 diff 0.
- **표의 파일 키**: `web` = index.html · app.js · view.js 중 어디든(옛 `[INDEX, APP, VIEW].concat()` 과 같은 뜻, 있음 표에서만). `a+b` = 둘 다에 없어야
  한다(없음 표). `usage` = `an_app::cli::USAGE` — 옛 `control_usage_literals` 시험을 표 행 넷으로 합쳤다.
- **계산 시험은 따로**: 초대 글(`Invite::to_text` 결과) · 말풍선 대비(2 테마 × 6 바탕 × 글자 3 = 36 경우) — 표로 옮기면 뜻이 흐려진다.
- **개수로만 말할 수 있는 규칙 둘은 `literal_counts` 안에**: '받은 글' 수 = '주고받은 글' 수(#17), `id="edit_btn"` 이 `<form id="reply_form"` 보다 앞(#1f).
- **단정 수 세기는 스크립트로**: 눈으로 옮긴 표는 빠뜨리기 쉽다. `gates.py ux` 가 옛 파일(`git show <sha>:…`)의 직접 단정 · `for` 배열(`(file, text)` ·
  `(file, text, label)` · `(file, text, lits)` · `[..].concat()`)을 풀어 짝을 만들고 새 표와 집합으로 비교한다 — 남은 쪽 0 · 새 쪽 0.

## 테스트 결과

```text
python docs/tasks/1p.project-tidy/design/gates.py code
notice_for=0 db_err_defs=1 agent_serde=False ux_tests=5 end
python docs/tasks/1p.project-tidy/design/gates.py ux 303f65a
present=258/258 absent=132/132 counts=3/3 only_old=0 only_new=0
cargo test -p coworkagent-app --test ux_literals
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
cargo test --workspace                                → test result 74줄 모두 ok, FAILED 0
cargo clippy --workspace --all-targets -- -D warnings → Finished(경고 0)
node --test --test-reporter=tap "crates/app/web/*.test.mjs"
# pass 112
# fail 0
```

| 항목 | 전 | 후 |
|------|------|------|
| `notice_for` 참조 | 1 | 0 |
| `fn db_err` 정의(src) | 3 | 1 |
| `an-agent` `serde` 의존 | 있음 | 없음 |
| `ux_literals.rs` 줄 · `#[test]` | 806 · 24 | 528 · 5 |
| 잠근 단정 짝(있음 / 없음 / 개수) | 258 / 132 / 3 | 258 / 132 / 3 |

## 다음 단계

- ▶ 다음: Stage 3 앞 — Claude Design 캔버스에 지금 화면 + 시안 셋을 내고 사용자 선택 · plan 개정 1(A11 · E13) 승인을 받는다(해석 18). 선택 뒤
  tasks/1p.project-tidy/work/stage3.md — `design/tokens.json` · 첫 `:root` · `css_tokens.rs` · mono 허용 목록.
