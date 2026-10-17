//! 화면 문구 · favicon 링크 literal(plan 해석 1) — 웹UI 원본(빌드 없는 정적 파일)을 그대로 읽는다.
//! 문구는 표 셋(있음 · 없음 · 개수)에 모으고, 계산이 있는 것(초대 글 · 말풍선 대비)만 따로 시험한다. 행의 셋째 칸은 출처(Task · 무엇).
use an_net::invite::Invite;

const INDEX: &str = include_str!("../web/index.html");
const APP: &str = include_str!("../web/app.js");
const VIEW: &str = include_str!("../web/view.js");
const TRAY: &str = include_str!("../src/tray.rs");
const CSS: &str = include_str!("../web/app.css");

/// 파일 키 → 원본. `web` 은 화면 원본 셋 중 어디든(있음 표에서만 쓴다), `usage` 는 CLI 사용법.
fn sources(key: &str) -> Vec<(&'static str, &'static str)> {
    let one = |k: &str| match k {
        "index.html" => ("index.html", INDEX),
        "app.js" => ("app.js", APP),
        "view.js" => ("view.js", VIEW),
        "app.css" => ("app.css", CSS),
        "tray.rs" => ("tray.rs", TRAY),
        "usage" => ("usage", an_app::cli::USAGE),
        other => panic!("모르는 파일 키 {other}"),
    };
    match key {
        "web" => vec![one("index.html"), one("app.js"), one("view.js")],
        _ => key.split('+').map(one).collect(),
    }
}

/// 있어야 하는 문구 — (파일 키, 문구, 출처).
#[rustfmt::skip]
const PRESENT: &[(&str, &str, &str)] = &[
    // 13 — 알림 절 자동 왕복 상한 라벨(사용자 원문)
    ("index.html", r#"<label for="relay_cap_default">자동 왕복 기본 상한</label>"#, "#13 상한 라벨"),
    // 15 — 초대 생성 · 초대 수락
    ("index.html", r#"id="window_btn" class="primary big">"#, "#15 초대"),
    ("index.html", r#"<label for="invite_text">초대 수락</label>"#, "#15 초대"),
    ("index.html", "받은 초대를 통째로 붙여 넣으세요. 지문이 맞는 상대에게만 연결합니다.", "#15 초대"),
    ("index.html", r#"<link rel="icon" href="/favicon.ico">"#, "#15 favicon"),
    ("index.html", r#"<input id="conv_search" type="search" placeholder="제목 검색" aria-label="제목 검색">"#, "#15 제목 검색"),
    ("tray.rs", r#"MenuItem::new("열기""#, "#17 트레이"),
    ("tray.rs", r#"MenuItem::new("종료""#, "#17 트레이"),
    // 17 — 시안 v4 영역별 라벨
    ("web", r#"id="search_q" type="search" placeholder="검색""#, "#17 라벨"),
    ("web", "+ 새로 만들기", "#17 라벨"),
    ("web", "새 대화", "#17 라벨"),
    ("web", "초대 생성", "#17 라벨"),
    ("web", "초대 수락", "#17 라벨"),
    ("web", "에이전트 등록", "#17 라벨"),
    ("web", "프로필 편집", "#17 라벨"),
    ("web", "알림 일시 중지", "#17 라벨"),
    ("web", r##"{ key: "settings", label: "설정", href: "#settings" }"##, "#17 라벨"),
    ("web", "내 활동", "#17 라벨"),
    ("web", "에이전트", "#17 라벨"),
    ("web", "주고받은 메시지", "#17 라벨"),
    ("web", "제목 검색", "#17 라벨"),
    ("web", "즐겨찾기", "#17 라벨"),
    ("web", "제목 편집", "#17 라벨"),
    ("web", "대화 보기", "#17 라벨"),
    ("web", "원문 보기", "#17 라벨"),
    ("web", "세부정보", "#17 라벨"),
    ("web", "북마크", "#17 라벨"),
    ("web", "복사", "#17 라벨"),
    ("web", "굵게", "#17 라벨"),
    ("web", "목록", "#17 라벨"),
    ("web", "코드", "#17 라벨"),
    ("index.html", r#"<span class="hint kbd-hint"><kbd>Enter</kbd> 보내기 · <kbd>Shift+Enter</kbd> 줄바꿈</span>"#, "#17 라벨 · #1p 키만 고정폭"),
    ("web", "보내기", "#17 라벨"),
    ("web", " 에게 보내기`", "#17 라벨"),
    ("web", "닫기", "#17 라벨"),
    ("web", "진행", "#17 라벨"),
    ("web", "요청 보냄", "#17 라벨"),
    ("web", "대기열", "#17 라벨"),
    ("web", "실행 완료", "#17 라벨"),
    ("web", "응답 받음", "#17 라벨"),
    ("web", "대화 정보", "#17 라벨"),
    ("web", "상대 에이전트", "#17 라벨"),
    ("web", "실행 기본값", "#17 라벨"),
    ("web", "이 PC 에만 저장", "#17 라벨"),
    ("web", "자동 왕복", "#17 라벨"),
    ("web", "멈춤", "#17 라벨"),
    // 18 — 집중 · 멘션 · 글 종류
    ("web", "받기 가능", "#18 문구"),
    ("web", "집중", "#18 문구"),
    ("web", "방금 통신", "#18 문구"),
    ("web", "집중 중", "#18 문구"),
    ("web", "통신 기록 없음", "#18 문구"),
    ("web", "상대가 집중 중", "#18 문구"),
    ("web", "글 종류", "#18 문구"),
    ("web", "질의", "#18 문구"),
    ("web", "요청", "#18 문구"),
    ("web", "에이전트 멘션", "#18 문구"),
    ("web", "@ 로 상대 에이전트 부르기", "#18 문구"),
    // 1d — 요청 목록 · 상태 · 도구 · 모델
    ("web", "받은 요청", "#1d 요청 목록"),
    ("web", "보낸 요청", "#1d 요청 목록"),
    ("web", r#"wait: "대기", accepted: "수용", rejected: "반려", done: "완료""#, "#1d 요청 목록"),
    ("web", "실행하지 않고 상대의 요청 목록에 올립니다", "#1d 요청 목록"),
    ("web", "탐색 도구(읽기 전용) 항상 사용", "#1d 도구"),
    ("web", r#"<select id="claude_model"></select>"#, "#1d 모델"),
    ("web", r#"export const MODELS = ["opus", "sonnet", "haiku"];"#, "#1d 모델"),
    // 1f — 이력 정리
    ("index.html", r#"aria-pressed="false" title="편집 모드">편집</button>"#, "#1f 편집"),
    ("index.html", r#"<span id="edit_count">0개 선택</span>"#, "#1f 편집"),
    ("index.html", r#"<button id="edit_delete" type="button" class="danger" disabled>삭제</button>"#, "#1f 편집"),
    ("index.html", r#"<button id="edit_done" type="button" class="primary">완료</button>"#, "#1f 편집"),
    ("index.html", r#"<button value="mine">내 창에서만</button>"#, "#1f 반영 범위"),
    ("index.html", r#"<button value="both">상대 창에도</button>"#, "#1f 반영 범위"),
    ("index.html", r#"<button value="cancel">취소</button>"#, "#1f 반영 범위"),
    ("app.js", r#"iconButton(COPY_SVG, "복사""#, "#1f 복사"),
    ("app.js", r#""복사했습니다""#, "#1f 복사"),
    ("app.js", r#"iconButton(PENCIL_SVG, "본문 편집""#, "#1f 편집"),
    ("view.js", r#""(편집됨)""#, "#1f 편집됨"),
    // 1g — 레일 범위('나' 메뉴 · 상대 타일)
    ("view.js", r##"{ label: "요청", href: "#requests", count: waiting, icon: "inbox" },"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "북마크", href: "#saved", icon: "bookmark" }"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "주고받은 메시지", href: "#p", icon: "swap" }"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "즐겨찾기", href: "#starred", count: starred, icon: "star" }"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "에이전트", href: "#agents", icon: "bot" }"##, "#1g 메뉴"),
    ("view.js", r##"[{ label: "개요", href: "#", icon: "home" }]"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }"##, "#1g 메뉴"),
    ("index.html", r#"<ol id="home_summary" class="rows"></ol>"#, "#1g 개요"),
    ("app.js", "{ ui_folded: keys }", "#1g 접힘"),
    ("app.js", "foldSaver.push([...state.folded])", "#1g 접힘"),
    ("app.js", "`대기 요청 ${g.waiting}`", "#1g 개요"),
    // 1i — 레일 그룹 · 타일 메뉴 · 끌어 놓기
    ("index.html", r#"id="rail_group_add""#, "#1i 그룹"),
    ("index.html", ">+ 그룹</button>", "#1i 그룹"),
    ("index.html", r#"id="conv_more_btn""#, "#1i 대화 메뉴"),
    ("index.html", r#"<div id="float" class="popover float" hidden></div>"#, "#1i 떠 있는 상자"),
    ("app.js", r#"text: "대화 추가""#, "#1i 그룹"),
    ("app.js", r#"text: "그룹에서 빼기""#, "#1i 그룹"),
    ("app.js", r#"role: "menuitemcheckbox""#, "#1i 그룹"),
    ("view.js", "같은 이름의 그룹이 있습니다", "#1i 그룹"),
    ("index.html", r#"id="tile_more_btn""#, "#1i 타일 메뉴"),
    ("index.html", r#"<label for="tile_alias" id="tile_alias_label">표시 이름</label>"#, "#1i 타일 편집"),
    ("app.js", r#"item("pencil", "타일 편집""#, "#1i 타일 메뉴"),
    ("app.js", r#"item("image-off", "이미지 지우기""#, "#1i 타일 메뉴"),
    ("app.js", r#""그룹 삭제","#, "#1i 타일 메뉴"),
    ("app.js", "그룹만 지웁니다. 대화는 남습니다.", "#1i 그룹 삭제"),
    ("app.js", r#""contextmenu""#, "#1i 타일 메뉴"),
    ("index.html", r#"<p id="order_live" class="sr-only" aria-live="polite"></p>"#, "#1i 자리 옮기기"),
    ("app.js", r#"draggable: "true""#, "#1i 자리 옮기기"),
    ("app.js", r#"text: dir < 0 ? "Alt + ↑" : "Alt + ↓""#, "#1i 자리 옮기기"),
    ("view.js", "번째로 옮김", "#1i 자리 옮기기"),
    // 1h — 안정화(북마크 · 개요 · 구분선 · 프로필 알약)
    ("index.html", r#"<strong><span class="ico ico-bookmark" aria-hidden="true"></span>북마크</strong>"#, "#1h 북마크"),
    ("index.html", r#"<h3>북마크 <span id="detail_saved_n""#, "#1h 북마크"),
    ("index.html", r#"<strong><span class="ico ico-home" aria-hidden="true"></span><span id="home_title">개요</span></strong>"#, "#1h 개요"),
    ("app.js", r#"act(on ? "북마크" : "북마크 해제""#, "#1h 북마크"),
    ("app.js", r#""북마크했습니다""#, "#1h 북마크"),
    ("app.js", r#"el("li", { class: "nav-sep", role: "separator" })"#, "#1h 구분선"),
    ("app.js", r#"el("span", { class: "avatar-name", text: name })"#, "#1h 프로필"),
    ("app.js", "`${name} · ${st.text}`", "#1h 프로필"),
    ("app.js", r#": "개요";"#, "#1h 개요"),
    ("app.js", r#"askScope("고친 본문 저장")"#, "#1h 북마크가 아닌 저장"),
    ("app.js", r#"peerAct("연결 설정 저장""#, "#1h 북마크가 아닌 저장"),
    ("app.js", r#"iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크""#, "#1h 북마크"),
    ("app.css", ".side-nav .nav-sep {", "#1h 구분선"),
    ("app.css", ".avatar-name { max-width: 12em;", "#1h 프로필"),
    // 1j — 메신저형 대화 · 메뉴 · 상태 색 · 레일 A안 · 상대 그룹 · 설정
    ("index.html", r#"<button id="view_btn" type="button" aria-pressed="true">대화 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<button id="raw_btn" type="button" aria-pressed="false">원문 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<span id="saved_n" class="count"></span>"#, "#1j 머리"),
    ("index.html", r#"id="star_btn" class="star-btn" aria-label="즐겨찾기""#, "#1j 별"),
    ("app.js", "side-${side}", "#1j 말풍선"),
    ("app.js", r#"class: "day-break""#, "#1j 말풍선"),
    ("app.js", r#"class: "room-preview""#, "#1j 대화방 줄"),
    ("app.js", "function starButton(", "#1j 별"),
    ("app.js", "function bookmarkButton(", "#1j 북마크"),
    ("app.css", ".msg.side-mine", "#1j 말풍선"),
    ("app.css", ".day-break", "#1j 말풍선"),
    ("app.css", ".seg", "#1j 머리"),
    ("app.css", ".star-btn", "#1j 별"),
    ("index.html", r#"<section id="starred" class="view" hidden>"#, "#1j 즐겨찾기"),
    ("index.html", r#"<strong><span class="ico ico-star" aria-hidden="true"></span>즐겨찾기</strong>"#, "#1j 즐겨찾기"),
    ("index.html", r#"<div id="starred_list" class="starred-list"></div>"#, "#1j 즐겨찾기"),
    ("app.js", r#"el("div", { class: "menu-head", text: "상태" })"#, "#1j 상태"),
    ("app.js", r#""menuitemradio""#, "#1j 상태"),
    ("app.js", "const OFFLINE_AFTER = 2;", "#1j 상태"),
    ("app.js", "function keepFocus(", "#1j 레일"),
    ("app.js", "function stepMove(", "#1j 레일"),
    ("app.js", r#"["위로 옮기기", -1],"#, "#1j 타일 메뉴"),
    ("app.js", r#"["아래로 옮기기", 1],"#, "#1j 타일 메뉴"),
    ("app.js", r#"class: "tile-name""#, "#1j 레일"),
    ("app.js", "tileFace(g.icon, true)", "#1j 레일"),
    ("app.js", r#""이미지를 지웠습니다""#, "#1j 타일 메뉴"),
    ("app.js", r##"go("#");"##, "#1j 레일"),
    ("view.js", r#"{ key: "avail", label: "받기 가능", hint: "새 요청을 받습니다""#, "#1j 상태"),
    ("view.js", r#"{ key: "focus", label: "집중", hint: "새 요청을 거절합니다""#, "#1j 상태"),
    ("view.js", r#"{ key: "offline", label: "앱과 연결 끊김", hint: "자동 표시""#, "#1j 상태"),
    ("view.js", r#""starred", "saved""#, "#1j 즐겨찾기"),
    ("app.css", "grid-template-columns: 72px ", "#1j 레일"),
    ("app.css", ".st-online { --st: var(--ok); }", "#1j 상태"),
    ("app.css", ".st-focus { --st: var(--run); }", "#1j 상태"),
    ("app.css", ".st-offline { --st: var(--danger); }", "#1j 상태"),
    ("app.css", ".rail-btn .tile-name {", "#1j 레일"),
    ("app.css", ".msg { gap: var(--sp-3); max-width: 90%; }", "#1j 말풍선 · #1p 간격 토큰"),
    ("index.html", "<h2>페어링<span class=\"help-wrap\">", "#1j 설정"),
    ("index.html", r#"<button type="submit" class="primary">수락</button>"#, "#1j 설정"),
    ("index.html", r#"<button type="button" id="peer_group_add">+ 그룹 추가</button>"#, "#1j 설정"),
    ("index.html", r#"<section id="peers" class="card">"#, "#1j 설정"),
    ("index.html", r#"<p id="invite_msg" role="status"></p>"#, "#1j 설정"),
    ("app.js", r#"text: "그룹 ▾","#, "#1j 상대 그룹"),
    ("app.js", r#"el("span", { text: "그룹" })"#, "#1j 상대 그룹"),
    ("app.js", r#"ico("plus"), el("span", { text: "새 그룹에 넣기…" })"#, "#1j 상대 그룹"),
    ("app.js", "/api/groups/${id}/peers", "#1j 상대 그룹"),
    ("app.js", r#""data-peer-section""#, "#1j 상대 그룹"),
    ("app.js", "function openGroupRename(", "#1j 상대 그룹"),
    ("app.js", "function groupDeleteForm(", "#1j 상대 그룹"),
    ("app.js", "peerMove(d.peer, d.from, to, state.groups)", "#1j 상대 그룹"),
    ("view.js", r#"export const NO_PEER_GROUP = "그룹에 없는 연결";"#, "#1j 상대 그룹"),
    ("view.js", "그룹 \"${group}\" 에 넣었습니다`", "#1j 상대 그룹"),
    // 1k — 다듬기
    ("index.html", r#"<button type="button" id="accept_btn" class="primary big" aria-expanded="false" aria-controls="invite_accept">"#, "#1k 설정"),
    ("index.html", r#"<section id="invite_accept" class="sub" hidden>"#, "#1k 설정"),
    ("index.html", r#"<button type="button" id="agent_new_btn" class="primary" aria-expanded="false" aria-controls="agent_new">에이전트 등록</button>"#, "#1k 설정"),
    ("index.html", "초대를 만들어 보내면 상대가 붙여 넣어 연결합니다. 5분 안에 들어온 첫 요청 1건은 승인 없이 등록되니 모르는 연결은 연결 목록에서 지우세요.", "#1k 설정"),
    ("app.js", "function manageItem(", "#1k 관리 줄"),
    ("app.js", r#"text: "관리","#, "#1k 관리 줄"),
    ("app.js", "`${a.name} 편집`", "#1k 에이전트"),
    ("app.js", "function editAgent(", "#1k 에이전트"),
    ("app.js", r#""data-peer-sub": String(p.id)"#, "#1k 관리 줄"),
    ("app.js", "function peerSub(", "#1k 관리 줄"),
    ("app.js", "`peer${p.id}_${s.group ?? \"none\"}`", "#1k 관리 줄"),
    ("app.js", r#"rowEls(rs, "주고받은 메시지 없음", { tile: true })"#, "#1k 주고받은 글"),
    ("app.js", r#"class: "chip agent-chip""#, "#1k 주고받은 글"),
    ("app.js", r#"goSettings("conn", "invite_text")"#, "#1k 레일 +"),
    ("app.js", r#"rowTile("agent-tile", null)"#, "#1k 줄 타일"),
    ("view.js", "agent: conv.dir === \"in\" ? `내 ${conv.agent}` : conv.agent", "#1k 받은 대화만 '내 '"),
    ("app.css", ".pair-btns {", "#1k 설정"),
    ("app.css", ".manage-head {", "#1k 설정"),
    ("app.css", ".rows a.row-with-tile {", "#1k 줄 타일"),
    // 1l — 아이콘 · 말풍선 색 · 설정 정리
    ("index.html", r#"<dialog id="me_dialog" class="me-dialog" aria-labelledby="me_title">"#, "#1l 사용자 정보"),
    ("index.html", r#"<span class="ico ico-user" aria-hidden="true"></span>사용자 정보</h2>"#, "#1l 사용자 정보"),
    ("index.html", r#"<span id="me_fingerprint" class="mono">"#, "#1l 사용자 정보"),
    ("index.html", r#"<div id="agent_new" class="new-agent" hidden>"#, "#1l 에이전트 등록"),
    ("index.html", r#"<label for="agent_prompt">추가 지시</label><span class="help-wrap">"#, "#1l 도움말"),
    ("index.html", "추가 지시 사용 예", "#1l 도움말"),
    ("index.html", r#"<select id="ui_rail_order">"#, "#1l 타일 순서"),
    ("index.html", r#"<option value="peers_first">사용자 → 그룹</option>"#, "#1l 타일 순서"),
    ("index.html", r#"<option value="groups_first">그룹 → 사용자</option>"#, "#1l 타일 순서"),
    ("index.html", "<h2>화면</h2>", "#1l 설정"),
    ("index.html", r#"<button type="button" class="help" aria-label="도움말" aria-describedby="tip_conn">?</button>"#, "#1l 도움말"),
    ("index.html", "지문이 상대 화면(프로필 → 사용자 정보)과 같은지 전화나 대면으로 확인하세요.", "#1l 지문 확인"),
    ("index.html", r#"<strong><span class="ico ico-bot" aria-hidden="true"></span><span id="agents_title">에이전트</span></strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-swap" aria-hidden="true"></span><span id="peer_title"></span></strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-bell" aria-hidden="true"></span>내 활동</strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-inbox" aria-hidden="true"></span>요청</strong>"#, "#1l 아이콘"),
    ("app.js", "function ico(name)", "#1l 아이콘"),
    ("app.js", r#"el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label }))"#, "#1l 아이콘"),
    ("app.js", r#""data-tone": colorOf(conv.peer_id)"#, "#1l 말풍선 색"),
    ("app.js", r#"$("me_dialog").showModal();"#, "#1l 사용자 정보"),
    ("app.js", "function showAgentNew(on)", "#1l 에이전트 등록"),
    ("app.js", r#"if (i.focus === "agent_name") showAgentNew(true);"#, "#1l 에이전트 등록"),
    ("app.js", "function applyRailOrder(order)", "#1l 타일 순서"),
    ("app.js", r#""ui_rail_order","#, "#1l 타일 순서"),
    ("app.js", r#"item(dir < 0 ? "arrow-up" : "arrow-down", text"#, "#1l 아이콘"),
    ("view.js", r#"{ key: "me", label: "사용자 정보" }"#, "#1l 사용자 정보"),
    ("app.css", ".ico { display: inline-block; flex: none; width: 16px; height: 16px;", "#1l 아이콘"),
    ("app.css", r#".msg[data-tone="peer-1"] { --tone: var(--peer-1); }"#, "#1l 말풍선 색"),
    ("app.css", ".msg.side-theirs .bubble::before { content: \"\"; position: absolute; inset: 0; z-index: -1; border-radius: inherit; background: var(--tone, var(--wait)); opacity: var(--bubble-theirs-alpha);", "#1l 말풍선 색 · #1p 층 토큰"),
    ("app.css", ".msg.side-mine .bubble { isolation: isolate; background: var(--surface); color: var(--text);", "#1l 말풍선 색"),
    ("app.css", ".help-wrap:hover .tip, .help:focus + .tip { display: block; }", "#1l 도움말"),
    // 1m — 컨텍스트
    ("index.html", r#"<button id="ctx_btn" type="button" class="tool-btn ctx-btn" aria-haspopup="menu""#, "#1m 컨텍스트"),
    ("index.html", r#"<span class="ico ico-gauge" aria-hidden="true"></span><span id="ctx_text">컨텍스트 알 수 없음</span>"#, "#1m 컨텍스트"),
    ("index.html", r#"<label for="auto_compact">자동 압축</label>"#, "#1m 자동 압축"),
    ("index.html", r#"<input type="checkbox" id="auto_compact" title="컨텍스트가 80% 를 넘으면 대화를 요약해 새 세션으로 이어 갑니다(이 PC 의 Claude 사용량을 씁니다)">"#, "#1m 자동 압축"),
    ("view.js", r#"{ key: "compact", label: "압축", icon: "compress""#, "#1m 컨텍스트"),
    ("view.js", r#"{ key: "clear", label: "비우기", icon: "reset""#, "#1m 컨텍스트"),
    ("view.js", "자동 압축(80%)", "#1m 컨텍스트"),
    ("view.js", "상대가 0.5.0 이상이어야 합니다", "#1m 컨텍스트"),
    ("view.js", "export const CONTEXT_LIMIT = 1_000_000;", "#1m 컨텍스트"),
    ("app.js", "/api/conversations/${conv.id}/context", "#1m 컨텍스트"),
    ("app.js", "\"auto_compact\",", "#1m 자동 압축"),
    ("app.js", "m.author === \"context\" ? contextEl(m) : messageEl(conv, m)", "#1m 구분선"),
    ("app.css", ".ctx-break {", "#1m 구분선"),
    ("app.css", ".ctx-btn[data-level=\"warn\"]", "#1m 컨텍스트"),
    ("app.css", ".ctx-btn[data-level=\"high\"]", "#1m 컨텍스트"),
    // 1o — 목록 나눔 · 말풍선 음영 · 레일 우클릭 · 설명 지우기
    ("view.js", r#"{ dir: "out", title: "내 대화" }"#, "#1o 목록 나눔"),
    ("view.js", r#"{ dir: "in", title: "에이전트 대화" }"#, "#1o 목록 나눔"),
    ("view.js", "export function splitByDir(", "#1o 목록 나눔"),
    ("view.js", "export function splitKey(", "#1o 목록 나눔"),
    ("app.js", "splitByDir(s.rows)", "#1o 목록 나눔"),
    ("app.js", "foldEl(`convs:${s.peer}`", "#1o 목록 나눔"),
    ("app.js", "starredSections(", "#1o 즐겨찾기 그대로"),
    ("app.js", r#"$("rail").addEventListener("contextmenu", (ev) => ev.preventDefault());"#, "#1o 레일 우클릭"),
    ("app.css", ".msg.side-mine .bubble { isolation: isolate; background: var(--surface); color: var(--text); }", "#1o 음영 · #1p 옆 띠 없음"),
    ("app.css", ".msg.side-mine .bubble::before { content: \"\"; position: absolute; inset: 0; z-index: -1; border-radius: inherit; background: var(--accent); opacity: var(--bubble-mine-alpha);", "#1o 음영 · #1p 층 토큰"),
    ("app.css", ".msg.side-theirs .bubble { isolation: isolate; background: var(--surface); }", "#1o 음영 · #1p 옆 띠 없음"),
    ("app.css", ".msg.side-mine .bubble a { color: var(--sent); }", "#1o 음영"),
    ("app.css", ".dir-head {", "#1o 목록 나눔"),
    ("index.html", r#"<p class="hint">이 PC 에서만 표시</p>"#, "#1o 설명 지우기"),
    ("app.js", r#"class: "small agent-new""#, "#1o 에이전트 새 대화"),
    ("app.js", r#"text: "새 대화""#, "#1o 에이전트 새 대화"),
    // 1n — 부모 도구 메뉴 사용법
    ("usage", "coworkAgent start [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent open [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent stop\n", "#1n 사용법"),
    ("usage", "종료 코드 8 = 시작 실패", "#1n 사용법"),
];

/// 없어야 하는 문구 — (파일 키, 문구, 출처). `a+b` 는 두 파일 모두에 없어야 한다.
#[rustfmt::skip]
const ABSENT: &[(&str, &str, &str)] = &[
    ("index.html", "새 자동 왕복 대화의 상한 기본값", "#13 상한 라벨"),
    ("index.html+app.js", "페어링 받기", "#15 초대"),
    ("index.html+app.js", "초대 붙여넣기", "#15 초대"),
    ("index.html+app.js", "메신저", "#15 초대"),
    ("index.html", "대화 검색", "#15 제목 검색"),
    ("tray.rs", "내 정보 보기", "#17 트레이"),
    ("index.html+app.js+view.js", "tab_timeline", "#17 타임라인 탭 없음"),
    ("index.html+app.js+view.js", "panel_timeline", "#17 타임라인 탭 없음"),
    ("index.html+app.js+view.js", "#t/", "#17 타임라인 탭 없음"),
    ("index.html+app.js+view.js", "타임라인", "#17 타임라인 탭 없음"),
    ("index.html+app.js+view.js", "완료 표시", "#1d 없어진 문구"),
    ("index.html+app.js+view.js", "완료 취소", "#1d 없어진 문구"),
    ("index.html+app.js+view.js", "허용 도구", "#1d 없어진 문구"),
    ("index.html+app.js+view.js", "model_names", "#1d 없어진 문구"),
    ("view.js", r#"key: "copy""#, "#1f 작업 막대에 복사 없음"),
    ("index.html", r#"id="side_agents""#, "#1g 사이드바 에이전트 절 없음"),
    ("index.html", r#"id="act_peer""#, "#1g 상대 거르기 칸 없음"),
    ("app.js", "renderSideAgents", "#1g 사이드바 에이전트 절 없음"),
    ("index.html+app.js+view.js", "저장된 항목", "#1h 북마크 항목으로"),
    ("index.html", "tab_sent", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "panel_sent", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "sent_list", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "conv_tag", "#1j 보냄 · 받음 태그 없음"),
    ("index.html", "보낸 메시지", "#1j 보낸 메시지 탭 없음"),
    ("app.js+view.js", "kindText", "#1j 방향 라벨 없음"),
    ("app.js+view.js", "사람 → 에이전트", "#1j 방향 라벨 없음"),
    ("app.js+view.js", "tag: conv.dir", "#1j 보냄 · 받음 태그 없음"),
    ("app.js+view.js", "★ 즐겨찾기", "#1j 별 절 없음"),
    ("app.js+view.js", "보낸 메시지", "#1j 보낸 메시지 탭 없음"),
    ("view.js", r#"key: "save""#, "#1j 북마크는 말풍선 아이콘 — 막대에 없음"),
    ("index.html+app.js+app.css", "focus_chip", "#1j 상단 집중 칩 없음"),
    ("index.html+app.js+app.css", "status-chip", "#1j 상단 집중 칩 없음"),
    ("index.html+app.js+app.css", "tile-text", "#1j 글자 타일 없음"),
    ("index.html+app.js+app.css", "tileText", "#1j 글자 타일 없음"),
    ("index.html+app.js+app.css", "renderFocusChip", "#1j 상단 집중 칩 없음"),
    ("view.js", "export function tileText", "#1j 글자 타일 없음"),
    ("app.css", ".rail-btn.group { box-shadow", "#1j 겹친 카드 없음"),
    ("view.js", "집중 — 요청 받지 않음", "#1j 프로필 메뉴는 받기 가능 · 집중 고르기"),
    ("index.html", "주소 · 코드로 직접 추가", "#1j 직접 추가 칸 없음"),
    ("index.html", r#"id="add_form""#, "#1j 직접 추가 칸 없음"),
    ("index.html", "읽기 전용</span>", "#1j 읽기 전용 표시 없음"),
    ("index.html", r#"id="peer_group""#, "#1j 그룹 칸 없음"),
    ("index.html", r#"id="agent_group""#, "#1j 그룹 칸 없음"),
    ("app.js+view.js", r#"placeholder: "그룹 없음""#, "#1j 그룹 칸 없음"),
    ("app.js+view.js", "add_form", "#1j 직접 추가 칸 없음"),
    ("app.js+view.js", "groupFilter", "#1j 그룹 칸 없음"),
    ("app.js+view.js", "fillGroupSelect", "#1j 그룹 칸 없음"),
    // 연결 목록의 "읽기 전용" 표시는 없다 — #1o 공유 대화 머리(`view.js` `shareHead`) · 공유한 대화 상태 줄(#1p C11)은 별개.
    ("app.js", r#"class: "ro", text: "읽기 전용""#, "#1j 읽기 전용 표시 없음"),
    ("index.html", "상대 에이전트를 누르면 새 대화, 내 에이전트를 누르면 설정으로", "#1k 안내문 없음"),
    ("index.html", "주고받은 글 — 모든 대화, 최신순", "#1k 안내문 없음"),
    ("index.html+app.js+view.js", r#"label: "다시 보내기""#, "#1k 다시 보내기 없음"),
    ("index.html+app.js+view.js", r#"key === "resend""#, "#1k 다시 보내기 없음"),
    ("index.html+app.js+view.js", "다시 보내기로", "#1k 다시 보내기 없음"),
    ("index.html", r#"<section id="me" class="card">"#, "#1l 내 정보 카드 없음"),
    ("index.html", r#"id="agent_q""#, "#1l 에이전트 검색 없음"),
    ("index.html", "<summary>새 에이전트 등록</summary>", "#1l 새 에이전트 등록 없음"),
    ("index.html", r#"<details class="guide">"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", "settings-cols", "#1l 두 열 없음"),
    ("index.html", r#"<p class="hint">한쪽이 초대를 만들어 보내고"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", r#"<p class="hint">페어링된 상대가 부를 수 있는"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", "\"내 정보 → 지문\"", "#1l 지문 확인"),
    ("index.html", r#"width="15" height="15""#, "#1l 아이콘 16px 한 벌"),
    ("app.js", "agent_q", "#1l 에이전트 검색 없음"),
    ("view.js", "자동 왕복 대화", "#1l 새로 만들기 메뉴에 자동 왕복 대화 없음"),
    ("view.js", r#"{ id: "me", label: "내 정보" }"#, "#1l 내 정보 → 사용자 정보"),
    ("app.css", "color-mix(", "#1l macOS 12 Safari 15 에 없는 color-mix"),
    ("app.css", "border-right: 3px solid var(--accent)", "#1p 말풍선 옆 띠 없음"),
    ("app.css", "border-left: 3px solid var(--tone", "#1p 말풍선 옆 띠 없음"),
    ("index.html", r#"<span class="hint mono kbd-hint">"#, "#1p 안내 한글은 본문 글꼴"),
    ("app.css", ".msg.side-mine .bubble { background: var(--accent); color: var(--on-accent);", "#1o 옛 말풍선"),
    ("app.css", "opacity: 0.12;", "#1o 옛 말풍선"),
    ("index.html+app.js", "상대별 대화 요약 — 대화 수 · 안 읽음 · 대기 요청 · 최근 활동, 누르면 그 대화로", "#1o 설명 지우기"),
    ("index.html+app.js", "실행 없이 모인 요청 — 받은 요청은 수용 · 반려 · 완료로 답하고, 보낸 요청은 상대가 바꾼 상태를 봅니다", "#1o 설명 지우기"),
    ("index.html+app.js", "☆ 로 넣은 대화방 — 이 PC 에만, 대화방 줄의 ☆ 로 넣고 뺍니다", "#1o 설명 지우기"),
    ("index.html+app.js", "메시지 작업 '북마크' 로 모은 글 — 이 PC 에만, 북마크한 순서 최신부터", "#1o 설명 지우기"),
    ("index.html+app.js", "— 메시지 작업 '북마크' 로 모읍니다", "#1o 설명 지우기"),
    ("index.html+app.js", "이 PC 에서만 보입니다. 그림은 가운데를 정사각형으로 잘라 작게 줄여 저장합니다.", "#1o 설명 지우기"),
    ("index.html+app.js", "의 에이전트 — 누르면 새 대화", "#1o 설명 지우기"),
    ("index.html+app.js", "text: `최근 ${formatTime(g.last)}`", "#1o 설명 지우기"),
    ("index.html+app.js", "text: `대화 ${g.convs}`", "#1o 설명 지우기"),
    // 1p — 문구 표(plan-copy)로 바뀐 잠금 문구의 옛 꼴.
    ("index.html", "대화 턴에 대한 상한 기본값", "#1p 문구"),
    ("index.html", "에서 만든 초대 메시지를 통째로 붙여 넣으세요", "#1p 문구"),
    ("index.html", r#"<label for="conv_search">"#, "#1p 문구 — 라벨과 같은 placeholder 하나"),
    ("index.html", "검색 — 메시지 · 대화 · 사람 · 에이전트", "#1p 문구"),
    ("index.html", "북마크 항목</strong>", "#1p 문구 T2"),
    ("index.html", "<h3>북마크 항목", "#1p 문구 T2"),
    ("view.js", r#"label: "북마크 항목""#, "#1p 문구 T2"),
    ("view.js", r#"label: "주고받은 글""#, "#1p 문구 T7"),
    ("app.js", r#""주고받은 글 없음""#, "#1p 문구 T7"),
    ("view.js", r#"label: "표시 이름 · 아이콘""#, "#1p 문구"),
    ("index.html+app.js+view.js", r#""환경설정""#, "#1p 문구 T5"),
    ("index.html+app.js+view.js", r#""실행 상세""#, "#1p 문구 T1"),
    ("index.html", ">실행 상세<", "#1p 문구 T1"),
    ("view.js", r#"title: "agent 대화""#, "#1p 문구 T3"),
    ("app.js", "에게 보내기 — markdown 가능", "#1p 문구"),
    ("web", "실행 없이 상대 요청 목록에 등록", "#1p 문구"),
    ("index.html", "편집 모드 — ", "#1p 문구"),
    ("app.js", r#""복사했다""#, "#1p 문구"),
    ("index.html", ">그룹 +<", "#1p 문구"),
    ("app.js", "대화는 지우지 않습니다", "#1p 문구"),
    ("app.js", "로 자리를 바꿉니다", "#1p 문구 — 자리 옮기기는 타일 메뉴 · 사용자 가이드"),
    ("app.js", "북마크 항목에 모았다", "#1p 문구"),
    ("app.js", "${name} — ${st.text}", "#1p 문구"),
    ("app.js", "상태 — 아이콘 테두리 색", "#1p 문구"),
    ("app.js", "사람 모양으로 보입니다", "#1p 문구"),
    ("view.js", r#"hint: "새 요청을 받는다""#, "#1p 문구"),
    ("view.js", r#"hint: "새 요청을 거절""#, "#1p 문구"),
    ("index.html", ">추가 · 승인<", "#1p 문구"),
    ("app.js", "체크하면 넣고, 풀면 뺀다", "#1p 문구"),
    ("view.js", "개가 함께 보입니다", "#1p 문구"),
    ("index.html", "받은 쪽이 붙여 넣으면 연결됩니다", "#1p 문구"),
    ("web", "화면 — 테마 · 타일 순서", "#1p 문구"),
    ("index.html", "프로필 → 사용자 정보 → 지문", "#1p 문구"),
    ("index.html", "자동 압축 — 컨텍스트 80% 넘으면", "#1p 문구"),
    ("view.js", "압축 — 요약 후 새 세션", "#1p 문구 Y"),
    ("view.js", "비우기 — 세션만", "#1p 문구 Y"),
    ("view.js", "${name} — ", "#1p 문구 Y — 컨텍스트 구분선은 ' · '"),
    ("view.js", ") — 다시 누르면 다시 요청", "#1p 문구 Y"),
    ("view.js", "요약 실행 실패 — ", "#1p 문구 Y"),
    ("view.js", "항상 사용 — ", "#1p 문구 Y"),
    ("view.js", "연결이 끊겼습니다 — ", "#1p 문구 Y"),
];

/// 정확한 개수 — (파일 키, 문구, 개수, 출처).
#[rustfmt::skip]
const COUNTS: &[(&str, &str, usize, &str)] = &[
    ("tray.rs", "MenuItem::new(", 4, "#17 정상 모드 열기 · 종료 + 오류 모드 오류 보기 · 종료"),
    ("index.html", r#"id="raw_btn""#, 1, "#17 원문 보기는 머리글 하나"),
    ("app.css", "\n.ico { ", 1, "#1l 아이콘 크기 정의는 한 곳(16px)"),
];

#[test]
fn literals_present() {
    let missing: Vec<String> = PRESENT
        .iter()
        .filter(|(key, lit, _)| !sources(key).iter().any(|(_, text)| text.contains(lit)))
        .map(|(key, lit, why)| format!("{why} — {key} 에 없음: {lit}"))
        .collect();
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}

#[test]
fn literals_absent() {
    let found: Vec<String> = ABSENT
        .iter()
        .flat_map(|(key, lit, why)| {
            sources(key)
                .into_iter()
                .filter(move |(_, text)| text.contains(lit))
                .map(move |(file, _)| format!("{why} — {file} 에 있음: {lit}"))
        })
        .collect();
    assert!(found.is_empty(), "{}", found.join("\n"));
}

/// 개수 표 + 개수로만 말할 수 있는 규칙 둘 — '받은 메시지' 라벨 없음('주고받은 메시지' 안의 것만, #17 · #1p T7) · 편집 단추는 입력창 위(#1f).
#[test]
fn literal_counts() {
    for (key, lit, n, why) in COUNTS {
        for (file, text) in sources(key) {
            assert_eq!(text.matches(lit).count(), *n, "{why} — {file}: {lit}");
        }
    }
    for (file, text) in sources("index.html+app.js+view.js") {
        assert_eq!(
            text.matches("받은 메시지").count(),
            text.matches("주고받은 메시지").count(),
            "#17 '받은 메시지' 라벨 없음 — {file}"
        );
    }
    let edit = INDEX.find(r#"id="edit_btn""#).expect("편집 단추");
    let reply = INDEX.find(r#"<form id="reply_form""#).expect("입력창");
    assert!(edit < reply, "#1f 편집 단추는 입력창 위");
}

/// 초대 글(#15) — "초대 수락" 기준 안내, "메신저" · "초대 붙여넣기" 없음.
#[test]
fn invite_text() {
    let invite = Invite::parse(
        "coworkagent-invite:v1/192.168.0.10:47100/ABCD-EFGH/00112233445566778899AABBCCDDEEFF",
    )
    .unwrap()
    .to_text("user-a", "PC-A");
    assert!(invite.contains("\"초대 수락\""), "{invite}");
    assert!(
        !invite.contains("메신저") && !invite.contains("초대 붙여넣기"),
        "{invite}"
    );
}

/// CSS 토큰 `--{name}: #RRGGBB;` → (r, g, b).
fn token(name: &str) -> [f64; 3] {
    let key = format!("--{name}: #");
    let at = CSS.find(&key).unwrap_or_else(|| panic!("토큰 {name}")) + key.len();
    let hex = &CSS[at..at + 6];
    [0, 2, 4].map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f64)
}

/// 말풍선 층 불투명도 토큰 `--bubble-{side}-alpha` 값(#1p).
fn alpha(side: &str) -> f64 {
    let key = format!("--bubble-{side}-alpha: ");
    let at = CSS.find(&key).unwrap_or_else(|| panic!("토큰 {key}")) + key.len();
    let v = &CSS[at..];
    v[..v.find(';').unwrap()].parse().unwrap()
}

fn luminance(c: [f64; 3]) -> f64 {
    let lin = |v: f64| {
        let s = v / 255.0;
        if s <= 0.03928 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2])
}

fn contrast(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// 말풍선 글자 대비(#1o A3 · #1p) — 2 테마 × (내 글 1 + 상대 글 타일 5색) × 글자 3(본문 · muted · 링크) = 36 경우 모두 4.5:1 이상. 바탕 = 색 층 × 불투명도(토큰) + surface.
#[test]
fn bubble_contrast() {
    let (mine, theirs) = (alpha("mine"), alpha("theirs"));
    assert_eq!((mine, theirs), (0.1, 0.0), "#1p 시안 A 먹선 층 불투명도");
    let mut cases = 0;
    for dark in [false, true] {
        let t = |name: &str| {
            token(&if dark {
                format!("dark-{name}")
            } else {
                name.to_string()
            })
        };
        let surface = t("surface");
        // (층 색, 불투명도, 링크 색) — 내 글 링크 = --sent(plan 해석 6), 상대 글 링크 = 전역 a(--accent).
        let mut layers = vec![(t("accent"), mine, t("sent"), "내 글".to_string())];
        layers.extend((1..=5).map(|i| {
            (
                token(&format!("peer-{i}")),
                theirs,
                t("accent"),
                format!("상대 글 peer-{i}"),
            )
        }));
        for (tone, alpha, link, who) in layers {
            let bg = [0, 1, 2].map(|i| tone[i] * alpha + surface[i] * (1.0 - alpha));
            for (what, fg) in [("본문", t("text")), ("muted", t("muted")), ("링크", link)] {
                let r = contrast(fg, bg);
                assert!(
                    r >= 4.5,
                    "{} {who} {what} {r:.2}",
                    if dark { "다크" } else { "라이트" }
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 36);
}
