# stage3 — 상대 단위 그룹(스키마 18) · 타일 메뉴 그룹 · 연결 목록 그룹 절 · 설정 정리 · 문서 · 이 PC 실측

> 근거: [plan](../plan.md) Stage 3.

## 목표

해석 10 ~ 12 — 그룹에 상대를 넣으면 그 상대의 대화(뒤에 생긴 것도)가 그룹에 보인다 · 옛 자유 입력 그룹(`peers.grp`)을 같은 이름 레일 그룹으로 옮김(Q1) ·
타일 메뉴 그룹 체크 · 연결 목록 = 레일 그룹별 절(끌기 · "그룹 ▾" · "+ 그룹 추가" · 이름 바꾸기 · 삭제) · 설정 "페어링" · 초대 두 단추 같은 모양 · "주소 · 코드로
직접 추가" · "읽기 전용" · 연결 · 에이전트 그룹 칸 없앰. 문서 · 캡처 · 이 PC 실측. 대상 gate: E1 ~ E10.

## 산출 파일

- `crates/core/src/db.rs` — 스키마 18: `group_peers(group_id, peer_id, PRIMARY KEY)` · `group_peers_peer` 색인 · `peers.grp` → 같은 이름 그룹(없으면 만듦, 상대
  자리 순).
- `crates/core/src/groups.rs` — `Group.peers` · `fit_name`(제어 문자 뺌 · 앞뒤 공백 · 앞 40자, 빈 이름 `None`) · `unknown_peers` · `change_peers` · `remove` 가
  `group_peers` 도.
- `crates/core/src/chat.rs` — `Scope::Group` SQL 에 `OR c.peer_id IN (SELECT peer_id FROM group_peers WHERE group_id = ?)`.
- `crates/app/src/groups_api.rs` — `PUT /api/groups/{id}/peers {add, remove}`(모르는 상대 400 · 없는 그룹 404) · 그룹 JSON `peers`.
- `crates/app/web/view.js` — `inGroup`(+ 상대) · `searchItems` · `peerSections` · `NO_PEER_GROUP` · `peerMove` · `peerGroupItems` · `josa` · `peerGroupText`.
  `groupFilter` · `groupNames` · `NO_GROUP` 없앰.
- `crates/app/web/app.js` — `peerGroupMenu`(타일 메뉴 · "그룹 ▾" 공용) · `changePeerGroup` · `groupDeleteForm`(타일 메뉴 · 절 머리 공용) · `openGroupRename` ·
  `openGroupNew(anchor, after)` · 연결 목록 절 · 끌기(`peerDrag`) · 내 에이전트 평평한 목록 · 그룹 대화 수 = `inGroup`(레일 · 사이드바 머리 · "그룹 대화 · n").
  `groupBox` · `fillGroupSelect` · `add_form` 처리 없앰. 초대 결과는 `invite_msg`.
- `crates/app/web/index.html` — "페어링" 카드 · `window_btn` `primary` · 수락 단추 "초대 수락" · 연결 목록 별도 카드(`peer_group_add`) · 읽기 전용 10곳 ·
  `add_form` · `peer_group` · `agent_group` 없앰. `app.css` — 연결 목록 절 · 끌기 손잡이.
- 시험 — `store.rs` `schema18_peer_groups_from_old_text` · `schema18_group_includes_peer_conversations` · `paths.rs` 18 · `groups_api.rs` `peer_membership` ·
  `group_scope_includes_peer_conversations` · `delete_group_clears_peers`(13 → 16) · `view.test.mjs` 87 → 95(`group_filter_search` 없앰, `search_items` ·
  `peer_sections` · `peer_move` · `peer_move_round_trip` · `in_group_peers` · `group_summary_peer_member` · `peer_group_items` · `josa` · `peer_group_text`) ·
  `ux_literals.rs` `peer_group_settings_literals`(새) · `invite_labels_without_messenger` 고침.
- 문서 — README · 사용자 가이드 · 개발자 가이드 · 모듈 매뉴얼 · PRODUCT · masterPlan(D35 · D37 · S4 · 개정 기록) · 캡처 `docs/images/{workspace,home,group,pairing,agents}.png`
  (mock 가상 데이터).

## 설계 결정 · plan 편차

- **수락 단추 문구** — "추가 · 승인" → "초대 수락"(시안 · 사용자 요청 "초대 수락 · 초대 생성 단추 같은 모양" — 두 단추가 같은 이름 · 같은 `primary`). 사용자 노출 문구 변경.
- **"페어링" 카드 · 연결 목록 카드 분리** — 시안 그대로 두 카드. 시안 안내 "메신저로 보내고" 는 문구 시험(`invite_labels_without_messenger` — 화면에 "메신저" 없음)과 맞지
  않아 "보내고" 로 옮겼다(편차). 초대 결과 문구는 `invite_msg`(페어링 카드), 연결 · 그룹 결과는 `peer_msg`(연결 목록 카드).
- **옮김 안내** — 끌어 옮기면 넣기 먼저(실패하면 옛 그룹에 남는다) · 빼기, 마지막 안내는 넣은 쪽 문구.
- **조사** — 시안 문구 "이준호를 … 이준호와의" 는 이름 받침에 따라 `josa`(한글이 아니면 "을(를)").
- **대화 ⋯ 메뉴 · "대화 추가"** — 대화를 하나씩 넣는 기존 체크는 그대로 `convs` 기준(상대 단위로 보이는 대화도 체크 안 됨 — 하나씩 넣은 것이 아님).
- **상대를 지워도 `group_peers` 는 남긴다** — 상대 번호를 다시 쓰지 않으므로(스키마 17) 새 상대에 붙지 않고, 남은 대화는 그룹에 그대로 보인다(#1i "지운 상대의 대화도 그룹에
  남는다" 와 같은 규칙).

## 검증

- `cargo test --workspace` 실패 0 · `cargo clippy --workspace --all-targets -- -D warnings` Finished · node `# pass 95` · `# fail 0` · `ux_literals` 17 · `groups_api`
  16 · `store schema1` 5 · `check_crates` violations=0.
- mock(가상 데이터, `j1/s3.mjs`) — 타일 메뉴 체크 → `결제 모듈:p[3]` · 안내 "최유나를 그룹 "결제 모듈" 에 넣었습니다 — 최유나와의 대화 3개가 함께 보입니다" · 메뉴 열린
  채 체크 바뀜 · "+ 새 그룹에 넣기…" → 새 그룹에 상대 · 레일 그룹 대화 수 · 그룹 타일 사이드바에 그 상대 대화 · 끌기(그룹 밖 → 릴리스 → 결제, 옮김) · "그룹 ▾" 로 빼기 ·
  이름 바꾸기 → 레일 이름 · 삭제 · 읽기 전용 · 직접 추가 없음 · JS 오류 0.
- 이 PC 실측(E10) — [eval](../eval.md) E10.
