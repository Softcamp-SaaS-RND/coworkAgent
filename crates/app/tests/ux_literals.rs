//! 화면 문구 · favicon 링크 literal(plan 해석 1) — 웹UI 원본(빌드 없는 정적 파일)을 그대로 읽는다.
//! 문구는 표 셋(있음 · 없음 · 개수)에 모으고, 계산이 있는 것(초대 글 · 말풍선 대비)만 따로 시험한다. 행의 셋째 칸은 출처(Task · 무엇).
use an_net::invite::Invite;
use std::sync::LazyLock;

const INDEX: &str = include_str!("../web/index.html");
const TRAY: &str = include_str!("../src/tray.rs");
const CSS: &str = include_str!("../web/app.css");
/// 화면 모듈을 이어 붙인 원본 — `app` = UI 모듈(`logic/` 밖 전부), `view` = DOM 없는 `logic/`.
static APP: LazyLock<String> = LazyLock::new(|| scripts(false));
static VIEW: LazyLock<String> = LazyLock::new(|| scripts(true));

fn scripts(logic: bool) -> String {
    let parts: Vec<&str> = an_app::server::SCRIPTS
        .iter()
        .filter(|(name, _)| name.starts_with("logic/") == logic)
        .map(|(_, body)| *body)
        .collect();
    parts.join("\n")
}

/// 파일 키 → 원본. `web` 은 화면 원본 셋 중 어디든(있음 표에서만 쓴다), `usage` 는 CLI 사용법.
fn sources(key: &str) -> Vec<(&'static str, &'static str)> {
    let one = |k: &str| match k {
        "index.html" => ("index.html", INDEX),
        "app" => ("app", APP.as_str()),
        "view" => ("view", VIEW.as_str()),
        "app.css" => ("app.css", CSS),
        "tray.rs" => ("tray.rs", TRAY),
        "usage" => ("usage", an_app::cli::USAGE),
        other => panic!("모르는 파일 키 {other}"),
    };
    match key {
        "web" => vec![one("index.html"), one("app"), one("view")],
        _ => key.split('+').map(one).collect(),
    }
}

/// 있어야 하는 문구 — (파일 키, 문구, 출처).
#[rustfmt::skip]
const PRESENT: &[(&str, &str, &str)] = &[
    // 13 — 알림 절 자동 왕복 상한 라벨(사용자 원문)
    ("index.html", r#"<label for="relay_cap_default"><span class="ico ico-reset lead" aria-hidden="true"></span>자동 왕복 기본 상한</label>"#, "#13 상한 라벨"),
    // 15 — 초대 생성 · 초대 수락
    ("index.html", r#"<button type="button" id="window_btn" class="primary" aria-controls="invite_make">"#, "#15 초대 · #1z-8 연결 머리 단추"),
    ("index.html", r#"<label for="invite_text">초대 수락</label>"#, "#15 초대"),
    ("index.html", "받은 초대를 통째로 붙여 넣으세요. 지문이 맞는 상대에게만 연결합니다.", "#15 초대"),
    ("index.html", r#"<link rel="icon" href="/favicon.ico">"#, "#15 favicon"),
    ("index.html", r#"<input id="conv_search" type="search" placeholder="제목 검색" aria-label="제목 검색">"#, "#15 제목 검색"),
    ("tray.rs", r#"MenuItem::new("열기""#, "#17 트레이"),
    ("tray.rs", r#"MenuItem::new("종료""#, "#17 트레이"),
    // 17 — 시안 v4 영역별 라벨
    ("web", r#"id="search_q" type="search" placeholder="검색""#, "#17 라벨"),
    ("index.html", r#"aria-controls="new_menu"><span class="ico ico-plus lead" aria-hidden="true"></span>새로 만들기</button>"#, "#17 라벨 · #1z-10 + 글자 → 아이콘"),
    ("web", "세션 생성", "#17 라벨 · #1x 새 대화 → 세션 생성"),
    ("web", "초대 생성", "#17 라벨"),
    ("web", "초대 수락", "#17 라벨"),
    ("web", "에이전트 등록", "#17 라벨"),
    ("web", "프로필 편집", "#17 라벨"),
    ("web", "알림 일시 중지", "#17 라벨"),
    ("web", r##"{ key: "settings", label: "설정", icon: "gear", href: "#settings" }"##, "#17 라벨 · #1r 아이콘"),
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
    ("web", "닫기", "#17 라벨"),
    ("web", "진행", "#17 라벨"),
    ("web", "요청 보냄", "#17 라벨"),
    ("web", "대기열", "#17 라벨"),
    ("web", "실행 완료", "#17 라벨"),
    ("web", "응답 받음", "#17 라벨"),
    ("web", "대화 정보", "#17 라벨"),
    ("web", "상대 에이전트", "#17 라벨"),
    ("index.html", r#"<dl id="info_rows" class="info"></dl>"#, "#17 라벨 · #1z-10 대화 정보 줄(JS)"),
    ("web", "자동 왕복", "#17 라벨"),
    ("web", "멈춤", "#17 라벨"),
    // 18 — 집중 · 멘션 · 글 종류
    ("web", "받기 가능", "#18 문구"),
    ("web", "집중", "#18 문구"),
    ("web", "방금 통신", "#18 문구"),
    ("web", "집중 중", "#18 문구"),
    ("web", "통신 기록 없음", "#18 문구"),
    ("web", "상대가 집중 중", "#18 문구"),
    ("index.html", r#"role="radiogroup" aria-label="메시지 종류""#, "#18 문구 — 질의 · 요청 고르기(#1p 이름 '메시지 종류')"),
    ("web", "질의", "#18 문구"),
    ("web", "요청", "#18 문구"),
    ("web", "에이전트 멘션", "#18 문구"),
    // 1d — 요청 목록 · 상태 · 도구 · 모델
    ("web", "받은 요청", "#1d 요청 목록"),
    ("web", "보낸 요청", "#1d 요청 목록"),
    ("web", r#"wait: "대기", accepted: "수용", rejected: "반려", done: "완료""#, "#1d 요청 목록"),
    ("web", "읽기 도구 항상 허용", "#1d 도구 · #1z-9 결정 9(편집 도구 막음)"),
    ("web", r#"<select id="claude_model"></select>"#, "#1d 모델"),
    ("web", r#"export const MODELS = ["opus", "sonnet", "haiku"];"#, "#1d 모델"),
    // 1f — 이력 정리(웹UI 편집 모드는 #1v 에서 삭제 — 없음 표)
    ("app", r#"iconButton(COPY_SVG, "복사""#, "#1f 복사"),
    ("app", r#""복사했습니다""#, "#1f 복사"),
    ("view", r#""(편집됨)""#, "#1f 편집됨"),
    // 1g — 레일 범위('나' 메뉴 · 상대 타일)
    ("view", r##"{ label: "요청", href: "#requests", count: waiting, icon: "inbox" },"##, "#1g 메뉴"),
    ("view", r##"{ label: "북마크", href: "#saved", icon: "bookmark" }"##, "#1g 메뉴"),
    ("view", r##"{ label: "즐겨찾기", href: "#starred", count: starred, icon: "star" }"##, "#1g 메뉴"),
    ("view", r##"{ label: "에이전트", icon: "bot", head: true }"##, "#1v 메뉴 머리"),
    ("view", r##"{ label: "등록된 에이전트", href: "#agents", icon: "bot", sub: true }"##, "#1v 하위 메뉴"),
    ("view", r##"...(me ? [{ label: "에이전트 응답", href: "#agents/convs", icon: "chat", sub: true, count: agentUnread }] : [])"##, "#1v 하위 메뉴 · #1z-2 에이전트 응답 '나' 에만"),
    ("index.html", r#"<button id="req_view_list" type="button" aria-pressed="true"><span class="ico ico-list lead" aria-hidden="true"></span>목록</button>"#, "#1v 요청 보기"),
    ("index.html", r#"<button id="req_view_board" type="button" aria-pressed="false"><span class="ico ico-grid lead" aria-hidden="true"></span>보드</button>"#, "#1v 요청 보기"),
    ("app", r#"`요청 의견 · ${formatTime(f.at)}`"#, "#1v 의견 말풍선(#1z 용어)"),
    ("app", r#"placeholder: "의견(선택) · 요청자에게 보입니다""#, "#1z 의견 칸"),
    ("app", r#"text: "의견" })"#, "#1z 요청 카드 의견"),
    ("app", r#"class: "tag agent-tag", text: "에이전트""#, "#1z 에이전트 칩"),
    ("app", r#"el("div", { class: "avatar agent-av", "aria-hidden": "true" }, ico("bot"))"#, "#1z 에이전트 아이콘"),
    ("app.css", "--chat-gutter: max(48px, 8%);", "#1z 말풍선 영역 · 입력창 좌우 여백"),
    ("app.css", "padding: var(--sp-5) var(--chat-gutter) 0;", "#1z 말풍선 영역 여백 · 스크롤 밖 간격"),
    ("app.css", ".composer { margin: var(--sp-5) var(--chat-gutter) var(--sp-4);", "#1z 입력창 여백"),
    ("index.html", r#"<div id="rail_resize" role="separator""#, "#1z 레일 손잡이"),
    ("index.html", r#"<option value="normal">크게</option>"#, "#1z 타일 크기 표시"),
    ("view", r#"{ id: "groups", label: "그룹설정", icon: "users" },"#, "#1z 그룹설정 · #1z-8 메뉴 8개"),
    ("index.html", r#"<h2 id="groups_h">그룹설정<span"#, "#1z 그룹설정 · #1z-8 화면 머리"),
    ("app", "onpointerdown: tileGrab,", "#1z 레일 타일 포인터 끌기"),
    ("app", "function renderConnPage(force) {", "#1z-8 연결 = 사용자 목록 · 상세(시안 B)"),
    ("app", "function renderGroupPage(force) {", "#1z-8 그룹설정 = 그룹 목록 · 상세(시안 B)"),
    ("app", r#"el("img", { src: icon, alt: "", draggable: "false" })"#, "#1z R2 타일 그림 기본 끌기 없음"),
    ("app", r#"$("rail").addEventListener("dragstart", (ev) => ev.preventDefault());"#, "#1z R2 레일 HTML 끌기 막음"),
    ("app", r#"if (p) n.replaceWith(gsDot(p));"#, "#1z R2 그룹설정 접속 점 갱신"),
    ("view", r##"[{ label: "개요", href: "#", icon: "home" }]"##, "#1g 메뉴"),
    ("view", r##"{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }"##, "#1g 메뉴"),
    ("index.html", r#"<ol id="home_summary" class="rows"></ol>"#, "#1g 개요"),
    ("app", "{ ui_folded: keys }", "#1g 접힘"),
    ("app", "foldSaver.push([...state.folded])", "#1g 접힘"),
    ("app", "`대기 요청 ${g.waiting}`", "#1g 개요"),
    // 1i — 레일 그룹 · 타일 메뉴 · 끌어 놓기
    ("index.html", r#"id="rail_group_add""#, "#1i 그룹"),
    ("index.html", r#"<span class="ico ico-folder-plus lead" aria-hidden="true"></span>그룹</button>"#, "#1i 그룹 · #1z-10 + 글자 → 아이콘"),
    ("index.html", r#"id="conv_more_btn""#, "#1i 대화 메뉴"),
    ("index.html", r#"<div id="float" class="popover float" hidden></div>"#, "#1i 떠 있는 상자"),
    ("app", r#"text: "대화 추가""#, "#1i 그룹"),
    ("app", r#"text: "그룹에서 빼기""#, "#1i 그룹"),
    ("app", r#"role: "menuitemcheckbox""#, "#1i 그룹"),
    ("view", "같은 이름의 그룹이 있습니다", "#1i 그룹"),
    ("index.html", r#"id="tile_more_btn""#, "#1i 타일 메뉴"),
    ("index.html", r#"<label for="tile_alias" id="tile_alias_label">표시 이름</label>"#, "#1i 타일 편집"),
    ("app", r#"item("pencil", "타일 편집""#, "#1i 타일 메뉴"),
    ("app", r#"item("image-off", "이미지 지우기""#, "#1i 타일 메뉴"),
    ("app", r#""그룹 삭제","#, "#1i 타일 메뉴"),
    ("app", "그룹만 지웁니다. 대화는 남습니다.", "#1i 그룹 삭제"),
    ("app", r#""contextmenu""#, "#1i 타일 메뉴"),
    ("index.html", r#"<p id="order_live" class="sr-only" aria-live="polite"></p>"#, "#1i 자리 옮기기"),
    ("app", r#"text: dir < 0 ? "Alt + ↑" : "Alt + ↓""#, "#1i 자리 옮기기"),
    ("view", "번째로 옮김", "#1i 자리 옮기기"),
    // 1h — 안정화(북마크 · 개요 · 구분선 · 프로필 알약)
    ("index.html", r#"<strong><span class="ico ico-bookmark" aria-hidden="true"></span>북마크</strong>"#, "#1h 북마크"),
    ("index.html", r#"<h3><span class="ico ico-bookmark lead" aria-hidden="true"></span>북마크 <span id="detail_saved_n""#, "#1h 북마크"),
    ("index.html", r#"<strong><span class="ico ico-home" aria-hidden="true"></span><span id="home_title">개요</span></strong>"#, "#1h 개요"),
    ("app", r#"act(on ? "북마크" : "북마크 해제""#, "#1h 북마크"),
    ("app", r#""북마크했습니다""#, "#1h 북마크"),
    ("app", r#"el("li", { class: "nav-sep", role: "separator" })"#, "#1h 구분선"),
    ("app", r#"el("span", { class: "avatar-name", text: name })"#, "#1h 프로필"),
    ("app", "`${name} · ${st.text}`", "#1h 프로필"),
    ("app", r#": "개요";"#, "#1h 개요"),
    ("app", r#"peerAct("연결 설정 저장""#, "#1h 북마크가 아닌 저장"),
    ("app", r#"iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크""#, "#1h 북마크"),
    ("app.css", ".side-nav .nav-sep {", "#1h 구분선"),
    ("app.css", ".avatar-name { max-width: 12em;", "#1h 프로필"),
    // 1j — 메신저형 대화 · 메뉴 · 상태 색 · 레일 A안 · 상대 그룹 · 설정
    ("index.html", r#"<button id="view_btn" type="button" aria-pressed="true"><span class="ico ico-chat lead" aria-hidden="true"></span>대화 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<button id="raw_btn" type="button" aria-pressed="false"><span class="ico ico-code lead" aria-hidden="true"></span>원문 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<span id="saved_n" class="count"></span>"#, "#1j 머리"),
    ("index.html", r#"id="star_btn" class="star-btn" aria-label="즐겨찾기""#, "#1j 별"),
    ("app", "side-${side}", "#1j 말풍선"),
    ("app", r#"class: "day-break""#, "#1j 말풍선"),
    ("app", r#"sideList && c.dir === "out" ? "" : kindChip(c, "room-kind"),"#, "#1z-2 대화 줄 아이콘 · #1z-7 유형별 → #1z-13 U8 (가) 무채 선 아이콘 → #1z-14 결정 4 사이드바 대화 목록 내 대화 줄은 아이콘 없음"),
    ("app", "...g.rows.map((r) => convItem(r, splitKey(s.key, g.dir), true))", "#1z-14 결정 4 사이드바 대화 목록만 sideList"),
    ("app", "function starButton(", "#1j 별"),
    ("app", "function bookmarkButton(", "#1j 북마크"),
    ("app.css", ".msg.side-mine", "#1j 말풍선"),
    ("app.css", ".day-break", "#1j 말풍선"),
    ("app.css", ".seg", "#1j 머리"),
    ("app.css", ".star-btn", "#1j 별"),
    ("index.html", r#"<section id="starred" class="view" hidden>"#, "#1j 즐겨찾기"),
    ("index.html", r#"<strong><span class="ico ico-star" aria-hidden="true"></span>즐겨찾기</strong>"#, "#1j 즐겨찾기"),
    ("index.html", r#"<div id="starred_list" class="starred-list"></div>"#, "#1j 즐겨찾기"),
    ("app", r#"el("div", { class: "menu-head", text: "상태" })"#, "#1j 상태"),
    ("app", r#""menuitemradio""#, "#1j 상태"),
    ("app", "const OFFLINE_AFTER = 2;", "#1j 상태"),
    ("app", "function keepFocus(", "#1j 레일"),
    ("app", "function stepMove(", "#1j 레일"),
    ("app", r#"["위로 옮기기", -1],"#, "#1j 타일 메뉴"),
    ("app", r#"["아래로 옮기기", 1],"#, "#1j 타일 메뉴"),
    ("app", r#"class: "tile-name""#, "#1j 레일"),
    ("app", "railFace(g.icon, true, g.color)", "#1j 레일 → #1z-13 결정 12 머리글자 → #1z-14 결정 1 그룹 그림 → #1z-15 결정 5 타일 색"),
    ("app", r#""이미지를 지웠습니다""#, "#1j 타일 메뉴"),
    ("app", r##"go("#");"##, "#1j 레일"),
    ("view", r#"{ key: "avail", label: "받기 가능", hint: "새 요청을 받습니다""#, "#1j 상태"),
    ("view", r#"{ key: "focus", label: "집중", hint: "새 요청을 거절합니다""#, "#1j 상태"),
    ("view", r#"{ key: "offline", label: "앱과 연결 끊김", hint: "자동 표시""#, "#1j 상태"),
    ("view", r#""starred", "saved""#, "#1j 즐겨찾기"),
    ("app.css", "--rail-w-normal: 72px;", "#1j 레일(#1z 끌기 — 폭 토큰)"),
    ("app.css", ".st-online { --st: var(--ok); }", "#1j 상태"),
    ("app.css", ".st-focus { --st: var(--run); }", "#1j 상태"),
    ("app.css", ".st-offline { --st: var(--danger); }", "#1j 상태"),
    ("app.css", ".rail-btn .tile-name {", "#1j 레일"),
    ("app.css", ".msg { gap: var(--sp-3); max-width: 90%; }", "#1j 말풍선 · #1p 간격 토큰"),
    ("index.html", r#"<h2 id="conn_h">연결<span class="help-wrap">"#, "#1j 설정 · #1z-8 연결 화면"),
    ("index.html", r#"<button type="submit" form="invite_form" class="primary"><span class="ico ico-check lead" aria-hidden="true"></span>수락</button>"#, "#1j 설정 · #1z-8 상세 아래 단추"),
    ("index.html", r#"<button type="button" id="peer_group_add"><span class="ico ico-plus" aria-hidden="true"></span>그룹 추가</button>"#, "#1j 설정 · #1z-8 그룹설정 머리"),
    ("index.html", r#"<section id="conn" class="set-page" aria-labelledby="conn_h" hidden>"#, "#1j 설정 · #1z-8 화면 = section"),
    ("index.html", r#"<p id="invite_msg" role="status"></p>"#, "#1j 설정"),
    ("app", r#"el("span", { text: "그룹" })"#, "#1j 상대 그룹"),
    ("app", r#"convAct(
                        "넣기","#, "#1z-8 그룹 넣기 = 단추(설정 끌기 없음)"),
    ("app", r#"convAct(
                    "빼기","#, "#1z-8 그룹 빼기 = 단추"),
    ("app", "function openGroupRename(", "#1j 상대 그룹"),
    ("app", "function groupDeleteForm(", "#1j 상대 그룹"),
    // 1k — 다듬기
    ("index.html", r#"<button type="button" id="accept_btn" aria-expanded="false" aria-controls="invite_accept">"#, "#1k 설정 · #1z-8 연결 머리 단추"),
    ("index.html", r#"<section id="invite_accept" class="md-pane" aria-labelledby="invite_accept_h" hidden>"#, "#1k 설정 · #1z-8 초대 수락 = 상세 칸"),
    ("index.html", r#"<button type="button" id="agent_new_btn" class="primary" aria-expanded="false" aria-controls="agent_new"><span class="ico ico-plus" aria-hidden="true"></span>에이전트 등록</button>"#, "#1k 설정"),
    ("index.html", "초대를 만들어 보내면 상대가 붙여 넣어 연결합니다. 5분 안에 들어온 첫 요청 1건은 승인 없이 등록되니 모르는 연결은 연결에서 지우세요.", "#1k 설정(#1z-8 사용자 표 = 연결)"),
    ("app", "function mdRow(page, id, face, name, sub, ...end) {", "#1k 관리 줄 → #1z-8 목록 줄 고르기"),
    ("app", r#"class: "md-row", "aria-pressed": String(pick[page] === id), onclick: () => setPick(page, id) },"#, "#1z-8 줄 고르기(연필 없음 — #1r 연필 개정)"),
    ("app", "`${a.name} 편집`", "#1k 에이전트"),
    ("app", "function editAgent(", "#1k 에이전트"),
    ("app", r#""data-peer-sub": String(p.id)"#, "#1k 관리 줄"),
    ("app", "function peerSub(", "#1k 관리 줄"),
    ("app", r#"$("conn_user").replaceChildren(...(p ? userDetail(p, convs) : []));"#, "#1z-2 사용자 펼침 → #1z-8 오른쪽 상세"),
    // 1w — 대화 나가기 · 삭제, 그룹 = 대화만 · 설정 연결 목록
    ("view", r#"{ key: "leave", label: "나가기", icon: "exit", danger: true, confirm: LEAVE_CONFIRM }"#, "#1x 대화 메뉴"),
    ("view", r#"{ key: "clear", label: "기록 삭제", icon: "trash", sub }"#, "#1x 대화 메뉴"),
    ("view", r#"[{ key: "clear_mine", label: "내 글만 삭제" }]"#, "#1x 기록 삭제"),
    ("view", r#"{ key: "clear_all", label: "모든 사용자 글 삭제" }"#, "#1x 기록 삭제"),
    ("view", r#"export const LEAVE_CONFIRM = "대화방이 목록에서 삭제됩니다.";"#, "#1x 나가기 확인(사용자 원문)"),
    ("view", r#"export const SESSION_NEW = "세션 생성";"#, "#1x 세션 생성"),
    ("view", r#"if (m.refused === "left") return "상대가 대화를 나갔습니다";"#, "#1w 거절 사유"),
    ("view", r#"if (c.left_at != null) return "나감";"#, "#1w 닫힌 대화"),
    ("view", r#"return c.peer_left_at != null ? "상대가 나감" : "";"#, "#1w 닫힌 대화"),
    ("app", r#""이 PC 의 대화 기록을 지웁니다. 대화는 남습니다.""#, "#1x 기록 삭제 확인"),
    ("app", r#""나와 상대 PC 의 대화 기록을 모두 지웁니다. 대화는 남습니다.""#, "#1x 기록 삭제 확인"),
    ("index.html", r#"<h1>세션 생성</h1>"#, "#1x 세션 생성"),
    ("index.html", r#"<button type="submit" class="primary">생성</button>"#, "#1x 세션 생성"),
    ("index.html", r#"<label for="ui_tile_size"><span class="ico ico-resize lead" aria-hidden="true"></span>타일 크기</label>"#, "#1x 타일 크기"),
    ("index.html", r#"<label for="notify_sound"><span class="ico ico-volume lead" aria-hidden="true"></span>효과음</label>"#, "#1x 효과음"),
    ("index.html", r#"<html lang="ko" data-theme="light">"#, "#1x 기본 테마 라이트"),
    ("index.html", r#"<label for="req_archive">archive 항목 보기</label>"#, "#1y 요청 archive(사용자 원문)"),
    ("index.html", r#"<label class="head-pick">기간 <select id="ui_peer_weeks" data-num></select></label>"#, "#1y 주고받은 메시지 기간"),
    ("view", r#"{ key: "unarchive", label: "archive 해제" }"#, "#1y 요청 archive"),
    ("view", r#"{ key: "archive", label: "archive" }"#, "#1y 요청 archive"),
    ("view", "label: `${n}주`", "#1y 기간 1 ~ 12주"),
    ("app", "if (await changeGroup(g.id, [c.id], [])) say(`그룹 ${g.name} 에 넣었습니다`);", "#1w 넣기 = 복사(다른 그룹 · 원래 자리 그대로) · #1z-8 단추"),
    ("app.css", ".ico-exit {", "#1w 아이콘"),
    ("app", "{ class: `has-mark pm-row pm-${kind}` },", "#1z-2 주고받은 메시지 시안 Peer"),
    ("app", r#"kind === "me" ? railFace(state.settings.me_icon, false, state.settings.me_color, true) : railFace(peer?.icon, false, peer?.color)"#, "#1z-2 아바타 → #1z-13 결정 12 머리글자 → #1z-15 결정 17 사람 = 레일 얼굴"),
    ("app", r#"goSettings("invite_accept", "invite_text")"#, "#1k 레일 + · #1r 수락 칸은 라우터가"),
    ("app", r#"mdHead("agents", agentTile(a.color), a.name,"#, "#1k 줄 타일 · #1z-7 로봇 타일(고른 색) · #1z-8 상세 머리"),
    ("view", "agent: conv.dir === \"in\" ? `내 ${conv.agent}` : conv.agent", "#1k 받은 대화만 '내 '"),
    ("app.css", ".md-split { flex: 1 1 auto; min-height: 420px; display: grid; grid-template-columns: 360px minmax(0, 1fr);", "#1z-8 목록 · 상세 2단"),
    ("app.css", ".set-tiles { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr));", "#1z-8 2열 타일"),
    ("app.css", "@media (max-width: 960px) {\n  .set-tiles, .md-cols { grid-template-columns: 1fr; }\n  .md-split { grid-template-columns: 1fr;", "#1z-8 좁은 창 1열 · 위아래"),
    ("app.css", "@media (max-width: 960px) {\n  .engine-tiles, .engine-blocks { grid-template-columns: 1fr; }", "#1z-9 좁은 창 도구 타일 · 블록 1열(정의 뒤에 — 앞에 두면 덮임)"),
    // 1z-9 — 실행 도구(시안 B): 등록 1단계 타일 → 2단계 폼(도구 칩 · 다시 고르기 · 좌우 옵션 블록 · Git 확인 건너뜀 스위치 기본 켬), 상세 도구 칩 · 블록
    ("index.html", r#"<span id="agent_new_step" class="hint">1 / 2 — 실행 도구 고르기</span>"#, "#1z-9 등록 1단계"),
    ("index.html", r#"<button type="button" id="agent_engine_claude" class="engine-tile" data-engine="claude">"#, "#1z-9 도구 타일"),
    ("index.html", r#"<button type="button" id="agent_engine_codex" class="engine-tile" data-engine="codex">"#, "#1z-9 도구 타일"),
    ("index.html", r#"<form id="agent_form" class="form-grid md-body" hidden>"#, "#1z-9 2단계 폼은 도구를 고른 뒤"),
    ("index.html", r#"<button type="button" id="agent_engine_back">도구 다시 고르기</button>"#, "#1z-9 도구 다시 고르기"),
    ("index.html", r#"<fieldset id="agent_claude_opts" class="engine-block">"#, "#1z-9 Claude 옵션 블록"),
    ("index.html", r#"<fieldset id="agent_codex_opts" class="engine-block">"#, "#1z-9 Codex 옵션 블록"),
    ("index.html", r#"<input type="checkbox" id="agent_skip_git" role="switch" checked><span><span class="ico ico-git lead" aria-hidden="true"></span>Git 저장소 아닌 폴더 허용</span>"#, "#1z-9 결정 2 스위치 기본 켬"),
    ("app", "if (!newEngine) return;", "#1z-9 도구 없는 등록 없음"),
    ("app", "등록 뒤 바꿀 수 없음 — 바꾸려면 새로 등록", "#1z-9 상세 실행 도구"),
    ("view", "`${engineLabel(engine)} 에이전트는 쓰지 않음`", "#1z-9 안 쓰는 블록 문구"),
    ("app.css", ".engine-block:disabled { border-style: dashed;", "#1z-9 안 쓰는 블록 점선"),
    ("app.css", r#"#settings_tree a[aria-current="page"] {"#, "#1z-8 고른 메뉴 표시"),
    ("app.css", ".rows li.pm-agent { background: var(--agent-soft); }", "#1z-2 에이전트 줄 바탕"),
    ("app", r#"model.value = draft?.model ?? p.model ?? "";"#, "#1z-2 R1 펼친 다른 사용자의 저장 전 값 보존"),
    ("app", "if (gsDrafts.get(p.id) === sent) gsDrafts.delete(p.id);", "#1z-2 R2 저장 응답 대기 중 고친 값 보존"),
    ("index.html", r#"title="초대 수락"><span class="ico ico-user-plus lead" aria-hidden="true"></span>사용자</button>"#, "#1z-2 레일 + 사용자 · #1z-10 + 글자 → 아이콘"),
    ("index.html", r#"title="새 그룹"><span class="ico ico-folder-plus lead" aria-hidden="true"></span>그룹</button>"#, "#1z-2 레일 + 그룹 · #1z-10 + 글자 → 아이콘"),
    ("index.html", "에이전트 응답</strong>", "#1z-2 에이전트 응답 화면 제목"),
    ("index.html", r#"<button id="home_view_preview" type="button" aria-pressed="true"><span class="ico ico-grid lead" aria-hidden="true"></span>미리보기</button>"#, "#1z-4 개요 보기 전환"),
    ("index.html", r#"<div id="home_preview" class="pv-list"></div>"#, "#1z-5 개요 미리보기 = 상대별 절"),
    ("app.css", ".pv-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, calc(256px * var(--pv-scale))), 1fr));", "#1z-5 카드 열 수 = 폭 · 미리보기 크기만큼 카드 폭(사용자 지시) · #1z-11 시안 C 256px"),
    ("app", "const secs = previewSections(groups, single, state.previewCols);", "#1z-5 '나' · 그룹은 절마다 한 줄"),
    ("app", "new ResizeObserver(refitPreview).observe($(\"home_preview\"));", "#1z-5 창 폭이 바뀌면 열 수를 다시 잰다"),
    ("app", "  refitPreview();", "미리보기 크기로 카드 폭이 바뀌면 열 수를 다시 잰다(사용자 지시)"),
    ("app.css", ".pv-msg { flex: none; max-width: 88%; padding: var(--pv-msg-pad); border-radius: var(--radius-lg); font-size: var(--fs-xs); line-height: var(--pv-lh-msg);", "#1z-11 결정 3 말풍선 = 시안 C 12px · 줄간격 1.5(사용자 지시 축소 11px 개정)"),
    ("app.css", ".pv-msg b { margin-right: var(--pv-gap-who); color: var(--muted); font-size: var(--pv-fs-who); font-weight: var(--fw-medium); }", "#1z-11 결정 3 말풍선 이름 = 시안 C 10.5px → #1z-13 결정 6 공용 토큰"),
    ("app", "text: `+${s.more}개 더 보기` })", "#1z-5 넘치면 \"+n개\" · #1z-11 시안 C 글"),
    ("app.css", ".pv-head strong { min-width: 0; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text); font-size: var(--fs-md); font-weight: var(--fw-medium); }", "#1z-11 결정 4 상대 이름 14px · 500 → #1z-13 결정 6 공용 토큰"),
    ("app.css", ".pv-sec + .pv-sec { border-top: 1px solid var(--border); }", "#1z-11 시안 C 절 사이 가는 선 → #1z-13 결정 6 공용 토큰"),
    ("app.css", ".pv-list { flex: 1 1 auto; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; padding: var(--pv-list-pad); background: var(--surface);", "#1z-11 시안 C 종이 바탕 → #1z-13 결정 6 · 개정 ③ 메인과 같은 흰 면(사용자 '더 연하게' 그대로)"),
    ("app.css", ".pv-msg.side-theirs { align-self: flex-start; background: var(--subtle); }", "#1z-11 결정 3 상대 글 = 흰 바탕 + 가는 테두리 → #1z-13 결정 12 사람 글 = 옅은 회색 면"),
    ("app.css", ".pv-msg.agent, .pv-msg.running { background: var(--surface); border: 1px solid var(--border); color: var(--text); }", "#1z-11 에이전트 · 실행 중 말풍선 → #1z-13 결정 12 에이전트 글 = 흰 면 + 가는 선"),
    ("app", r#"c.unread ? el("span", { class: "pv-new", text: `새 글 ${c.unread}` }) : "","#, "#1z-11 결정 6 안 읽음 = 머리 \"새 글 n\""),
    ("app", r#"c.last_state ? el("span", { class: `pv-state st-${c.last_state}`, text: stateText(c.last_state) }) : "","#, "#1z-11 상태 = 바닥 점 + 글자"),
    ("app", r#"el("span", { class: "pv-pres" }, el("span", { class: `st-dot st-${pres.key}`, "aria-hidden": "true" }), el("span", { text: p ? pres.text : "페어링 해제" })),"#, "#1z-11 머리 접속 = 점 + 글"),
    ("app", "    el(\"div\", { class: \"pv-head\" }, ...head),", "개요 미리보기 절 머리에 \"{이름} 개요 →\" 없음(사용자 지시)"),
    ("app.css", ".side-list li[data-conv] > .row-more { position: static; flex: none; align-self: center;", "대화 줄 ⋯ 이 시각을 가리지 않는다(사용자 지시)"),
    ("app", "if (state.railPeer == null && state.railGroup == null) pickRail(c.peer_id);", "'나' 개요에서 연 대화 → 레일 · 사이드바도 그 상대로(사용자 지시)"),
    ("app", "el(\"h2\", { class: \"side-title\" }, el(\"span\", { text: s.title })),", "에이전트 응답 사용자 머리에 대화 수 없음(사용자 지시)"),
    ("index.html", r#"<label id="home_zoom_pick" class="head-pick"><span class="ico ico-resize lead" aria-hidden="true"></span>미리보기 크기 <select id="ui_home_zoom" data-num><option value="100">100%</option><option value="125">125%</option><option value="150">150%</option><option value="175">175%</option><option value="200">200%</option></select></label>"#, "#1z-6 미리보기 크기 100 ~ 200%"),
    ("app.css", ".pv-body { height: calc(132px * var(--pv-scale));", "#1z-6 본문 높이 배율 · #1z-11 시안 C 132px"),
    ("app.css", ".pv-text { display: -webkit-box; -webkit-line-clamp: 2;", "#1z-6 말풍선 줄 수 → #1z-8 모든 크기 2줄 · 안쪽 칸에서 자름"),
    ("app", "$(\"home_zoom_pick\").hidden = !pv;", "#1z-6 미리보기에서만"),
    ("app", "applyHomeZoom(s.ui_home_zoom);", "#1z-6 R1 배율은 시작할 때 · 고를 때만(늦은 설정 응답이 되돌리지 않게)"),
    ("app", "  homeZoomSaver.push(Number(e.target.value));", "#1z-6 R2 배율 저장은 한 번에 하나 · 마지막 값"),
    ("app", "if (all) for (const id of state.previewMsgs.keys()) if (!convs.some((c) => c.id === id)) state.previewStale.add(id);", "#1z-5 R1 전체 무효화는 숨은 카드에도"),
    ("app", "api(\"GET\", `/api/conversations/${c.id}/messages?limit=${PREVIEW_MSGS}`)", "#1z-4 미리보기는 마지막 글만"),
    ("app", "api(\"PUT\", \"/api/settings\", { ui_home_view: v })", "#1z-4 보기 기억"),
    ("app", "$(\"home_intro\").hidden = groups.length > 0 || group != null;", "#1z-4 R2 첫 연결 안내는 보기 분기 전"),
    ("app", "    state.previewAll = true;\n    refreshHome();\n  };\n  es.onmessage", "#1z-4 R2 변경 신호 다시 붙으면(onopen) 미리보기 전부 새로"),
    ("app.css", "#rail { user-select: none; }", "#1z-2 레일 끌기 글자 선택 없음"),
    ("app", "railSlot(same.map(mid), across ? e.clientX : e.clientY)", "#1z-2 레일 놓을 자리 = 가장 가까운 타일"),
    ("app", r#"el("span", { class: "home-agent" }, speakerEl("agent"),"#, "#1z-2 개요 줄 에이전트 아이콘"),
    ("app", r#"rowTile("small", p?.icon, p?.color),"#, "#1z-2 개요 머리 사용자 아이콘 → #1z-13 결정 12 머리글자 → #1z-15 결정 17 레일 얼굴"),
    // 1l — 아이콘 · 말풍선 색 · 설정 정리
    ("index.html", r#"<dialog id="me_dialog" class="me-dialog" aria-labelledby="me_title">"#, "#1l 사용자 정보"),
    ("index.html", r#"<span class="ico ico-user" aria-hidden="true"></span>사용자 정보</h2>"#, "#1l 사용자 정보"),
    ("index.html", r#"<span id="me_fingerprint" class="mono">"#, "#1l 사용자 정보"),
    ("index.html", r#"<section id="agent_new" class="md-pane new-agent" aria-labelledby="agent_new_h" hidden>"#, "#1l 에이전트 등록 · #1z-8 상세 칸 폼"),
    ("index.html", r#"<label for="agent_prompt"><span class="ico ico-message lead" aria-hidden="true"></span>추가 지시</label><span class="help-wrap">"#, "#1l 도움말"),
    ("index.html", "추가 지시 사용 예", "#1l 도움말"),
    ("index.html", r#"<select id="ui_rail_order">"#, "#1l 타일 순서"),
    ("index.html", r#"<option value="peers_first">사용자 → 그룹</option>"#, "#1l 타일 순서"),
    ("index.html", r#"<option value="groups_first">그룹 → 사용자</option>"#, "#1l 타일 순서"),
    ("index.html", r#"<h2 id="screen_h">화면</h2>"#, "#1l 설정 · #1z-8 화면 머리"),
    ("index.html", r#"<button type="button" class="help" aria-label="도움말" aria-describedby="tip_conn">?</button>"#, "#1l 도움말"),
    ("index.html", "지문이 상대 화면(프로필 → 사용자 정보)과 같은지 전화나 대면으로 확인하세요.", "#1l 지문 확인"),
    ("index.html", r#"<strong><span class="ico ico-bot" aria-hidden="true"></span><span id="agents_title">에이전트</span></strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-swap" aria-hidden="true"></span><span id="peer_title"></span></strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-bell" aria-hidden="true"></span>내 활동</strong>"#, "#1l 아이콘"),
    ("index.html", r#"<strong><span class="ico ico-inbox" aria-hidden="true"></span>요청</strong>"#, "#1l 아이콘"),
    ("app", "function ico(name)", "#1l 아이콘"),
    ("app", r#"el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label }))"#, "#1l 아이콘"),
    ("app", r#"$("me_dialog").showModal();"#, "#1l 사용자 정보"),
    ("app", "function showAgentNew(on)", "#1l 에이전트 등록"),
    ("app", r#"if (r.sub === "agent_new") showAgentNew(true);"#, "#1l 에이전트 등록 · #1z-8 별칭 agent_new"),
    ("app", "function applyRailOrder(order)", "#1l 타일 순서"),
    ("app", r#""ui_rail_order","#, "#1l 타일 순서"),
    ("app", r#"item(dir < 0 ? "arrow-up" : "arrow-down", text"#, "#1l 아이콘"),
    ("view", r#"{ key: "me", label: "사용자 정보", icon: "user" }"#, "#1l 사용자 정보 · #1r 아이콘"),
    ("app.css", ".ico { display: inline-block; flex: none; width: 16px; height: 16px;", "#1l 아이콘"),
    ("app.css", ".help-wrap:hover .tip, .help:focus + .tip { display: block; }", "#1l 도움말"),
    // 1m — 컨텍스트
    ("index.html", r#"<button id="ctx_btn" type="button" class="tool-btn ctx-btn" aria-haspopup="menu""#, "#1m 컨텍스트"),
    ("index.html", r#"<span class="ico ico-gauge" aria-hidden="true"></span><span id="ctx_text">컨텍스트 알 수 없음</span>"#, "#1m 컨텍스트"),
    ("index.html", r#"<label for="auto_compact"><span class="ico ico-compress lead" aria-hidden="true"></span>자동 압축</label>"#, "#1m 자동 압축"),
    ("index.html", r#"<span class="hint">컨텍스트가 80% 를 넘으면 대화를 요약해 새 세션으로 이어 갑니다(이 PC 의 Claude 사용량을 씁니다)</span></span><input type="checkbox" id="auto_compact" role="switch">"#, "#1m 자동 압축 · #1z-8 타일 설명 · 스위치"),
    ("view", r#"{ key: "compact", label: "압축", icon: "compress""#, "#1m 컨텍스트"),
    ("view", r#"{ key: "clear", label: "비우기", icon: "reset""#, "#1m 컨텍스트"),
    ("view", "자동 압축(80%)", "#1m 컨텍스트"),
    ("view", "상대가 0.5.0 이상이어야 합니다", "#1m 컨텍스트"),
    ("view", "export const CONTEXT_LIMIT = 1_000_000;", "#1m 컨텍스트"),
    ("app", "/api/conversations/${conv.id}/context", "#1m 컨텍스트"),
    ("app", "\"auto_compact\",", "#1m 자동 압축"),
    ("app", "m.author === \"context\" ? contextEl(m) : messageEl(conv, m)", "#1m 구분선"),
    ("app.css", ".ctx-break {", "#1m 구분선"),
    ("app.css", ".ctx-btn[data-level=\"warn\"]", "#1m 컨텍스트"),
    ("app.css", ".ctx-btn[data-level=\"high\"]", "#1m 컨텍스트"),
    // 1o — 목록 나눔 · 말풍선 음영 · 레일 우클릭 · 설명 지우기
    ("view", r#"{ dir: "out", title: "내 대화" }"#, "#1o 목록 나눔"),
    ("view", r#"{ dir: "in", title: "에이전트 대화" }"#, "#1o 목록 나눔"),
    ("view", "export function splitByDir(", "#1o 목록 나눔"),
    ("view", "export function splitKey(", "#1o 목록 나눔"),
    ("app", "convListParts(s.rows, group)", "#1o 목록 나눔 · #1v 내 대화만"),
    ("app", "foldEl(`convs:${s.peer}`", "#1o 목록 나눔"),
    ("app", "starredSections(", "#1o 즐겨찾기 그대로"),
    ("app", r#"$("rail").addEventListener("contextmenu", (ev) => ev.preventDefault());"#, "#1o 레일 우클릭"),
    ("app.css", ".dir-head {", "#1o 목록 나눔"),
    ("index.html", r#"<p class="hint">이 PC 에서만 표시</p>"#, "#1o 설명 지우기"),
    ("app", r#"class: "small agent-new""#, "#1o 에이전트 새 대화"),
    ("app", r#"text: SESSION_NEW"#, "#1o 에이전트 새 대화 · #1x 세션 생성"),
    // 1n — 부모 도구 메뉴 사용법
    ("usage", "coworkAgent start [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent open [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent stop\n", "#1n 사용법"),
    ("usage", "종료 코드 8 = 시작 실패", "#1n 사용법"),
    // 1q — 요청 개요 · 글자 단추
    ("app", "function reqBoxEl(", "#1q 요청 개요"),
    ("app", r#"foldEl(`requests:${box}`, head, body, "peer-fold req-box", null, box === "mine")"#, "#1q 보낸 요청 처음 접힘"),
    ("app", "state.reqBox = state.reqBox === b && !reqBoard() ? null : b;", "#1q 탭 토글 · #1v 보드엔 개요 없음"),
    ("app", r#"onclick: target === "requests" && current ? () => loadRequests(true) : null"#, "#1q 메뉴 다시 누름 = 개요"),
    ("app", "bar.append(textButton(mv.label, REQ_BTN[mv.to]", "#1q 글자 단추"),
    ("view", r#"rejected: { tone: "no", icon: "x" }"#, "#1q 반려 ✕"),
    ("view", r#"accepted: { tone: "ok", icon: "check" }"#, "#1q 수용 ✓"),
    ("app.css", "button.text-btn.ok { color: var(--ok); }", "#1q 글자 단추"),
    ("app.css", "button.text-btn.no { color: var(--danger); }", "#1q 글자 단추"),
    ("app.css", ".ico-check {", "#1q 아이콘"),
    ("app.css", ".ico-x {", "#1q 아이콘"),
    ("index.html", r#"<nav id="act_tabs" class="tabs" aria-label="내 활동 종류"></nav>"#, "#1q 내 활동 탭"),
    ("view", r#"{ kind: "request", label: "받은 요청", icon: "arrow-down" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view", r#"{ kind: "mine", label: "내 요청", icon: "arrow-up" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view", r#"{ kind: "answer", label: "받은 답", icon: "chat" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view", "`내 요청 · ${state}`", "#1q 내 요청 행"),
    ("app", r#"api("GET", "/api/messages/peers")"#, "#1q 주고받은 메시지 상대별 요약"),
    ("app", r#"foldEl(`peer:${h.peer}`, head, body, "peer-fold peer-msgs", null, true)"#, "#1q 상대 절 처음 접힘"),
    ("app", "`/api/messages?side=both&peer=${peer}${q}`", "#1q 서버 페이지"),
    ("app", r#"button("더 보기", more, "small")"#, "#1q 더 보기"),
    ("app", r#"b.classList.add("bookmark", "row-mark");"#, "#1q 행 북마크"),
    ("view", "export const PEER_PAGE = 20;", "#1q 20건씩"),
    ("app.css", ".rows li.has-mark {", "#1q 행 북마크 자리"),
    // 1r — 내 응답 에이전트 · 들여쓰기 · ? · 초대 · 새 에이전트 줄 · 연필 · 아이콘
    ("index.html", r#"<span class="ico ico-swap" aria-hidden="true"></span>내 응답 에이전트</label>"#, "#1r 새 대화"),
    ("index.html", r#"<label for="chat_relay_cap" id="chat_relay_cap_label" class="sub-label" hidden>"#, "#1r 최대 응답 횟수 하위 줄"),
    ("index.html", "최대 응답 횟수</label>", "#1r 최대 응답 횟수"),
    ("app", r#"$("chat_relay_cap").hidden = $("chat_relay_cap_label").hidden = $("chat_relay_cap").disabled = off;"#, "#1r 고를 때만 · R1 숨기면 검사 제외"),
    ("app", "if (inviteBusy) return;", "#1r R1 초대 생성 한 번에 하나"),
    ("app", r#"$("window_btn").setAttribute("aria-disabled", "true");"#, "#1r R2 초점 잃지 않게 aria-disabled"),
    ("app", r#"$(pick.agents != null ? mdRowId("agents", pick.agents) : "agent_new_btn")?.focus();"#, "#1r R1 등록 뒤 초점(숨긴 폼에 남지 않게) · #1z-8 등록한 줄"),
    ("app.css", ".peer-fold > :not(summary) { margin-left: var(--sp-5); }", "#1r 접이식 들여쓰기 · #1s 세로선 없음"),
    ("index.html", r#"aria-describedby="tip_relay""#, "#1r 자동 왕복 ?"),
    ("index.html", "• 시작과 끝 — 왕복이 시작할 때와 끝날 때(상한 도달 · 실패 · 상대가 멈춤 등)만 알림", "#1r 자동 왕복 ?"),
    ("index.html", "• 메시지마다 — 시작 · 끝에 더해 상대 에이전트의 답이 올 때마다 알림", "#1r 자동 왕복 ?"),
    ("index.html", "• 알리지 않음 — 자동 왕복 알림 없음", "#1r 자동 왕복 ?"),
    ("view", r#"press: "window_btn""#, "#1r 초대 생성 누름"),
    ("app", "if (i.press) $(i.press).click();", "#1r 새로 만들기 초대 생성"),
    ("app", r#"showAccept(r.sub === "invite_accept");"#, "#1r 수락 칸 접기(다른 진입이면 닫음) · #1z-8"),
    ("index.html", r#"<strong id="agent_new_h">새 에이전트</strong>"#, "#1r 새 에이전트 · #1z-8 상세 칸"),
    ("index.html", r#"<button type="button" id="agent_new_cancel"><span class="ico ico-x lead" aria-hidden="true"></span>취소</button>"#, "#1r 새 에이전트 줄"),
    ("app.css", ".ico-gear {", "#1r 새 아이콘"),
    ("app.css", ".ico-title {", "#1r 새 아이콘"),
    ("index.html", r#"<span class="ico ico-arrow-down" aria-hidden="true"></span>받은 요청</a>"#, "#1r 요청 탭 아이콘"),
    ("index.html", r#"<span class="ico ico-arrow-up" aria-hidden="true"></span>보낸 요청</a>"#, "#1r 요청 탭 아이콘"),
    ("app", r#"}, ico(k.icon), el("span", { text: k.label })),"#, "#1r 내 활동 탭 아이콘"),
    // 1z-7 — 대화 유형 아이콘 · 색(시안 A), 내 에이전트 상세(한 격자 · 아이콘 색 · 닫기)
    ("index.html", r#"<span id="conv_kind" class="conv-kind kind-chip" role="img" hidden></span><strong id="conv_title"></strong>"#, "#1z-7 제목 앞 유형 아이콘"),
    ("view", r#"export const KIND_NAMES = { in: "내 에이전트 응답", out: "내가 문의", shared: "공유 대화" };"#, "#1z-7 유형 이름 · #1z-13 U8 공유도 아이콘(읽는 이름 = 사이드바 절 머리)"),
    ("app.css", ".pv-card.unseen { border-width: 2px; }", "#1z-7 안 읽음 표시 → #1z-11 결정 6 테두리 색 → #1z-13 결정 5 강조색 → #1z-14 결정 3 유형 색 2px"),
    ("app.css", r#".pv-card[data-kind="in"] { border-color: var(--card-agent); }"#, "#1z-14 결정 2 에이전트 대화 = 청록"),
    ("app.css", r#".pv-card[data-kind="out"] { border-color: var(--card-mine); }"#, "#1z-14 결정 2 내 대화 = 차분한 남색"),
    ("app.css", r#".pv-card[data-kind="shared"] { border-color: var(--card-watch); }"#, "#1z-14 결정 2 관전 대화 = 호박"),
    ("app.css", ".pv-card { display: flex; flex-direction: column; min-width: 0; overflow: hidden; background: var(--surface); border: 1px solid var(--border);", "#1z-7 카드 테두리 = 유형 색 → #1z-11 결정 6 한 색 → #1z-13 결정 6 공용 토큰"),
    ("app", r#"{ class: c.unread ? "pv-card unseen" : "pv-card", href: `#c/${c.id}`, "aria-label": `${[KIND_NAMES[c.dir], title].filter(Boolean).join(" · ")} 대화 열기`, onclick: () => followFromHome(c), "data-kind": c.dir },"#, "#1z-7 미리보기 카드 유형 · R1 읽는 이름에 유형"),
    ("app", r#"el("span", { class: `${cls} kind-chip`, role: "img", "aria-label": KIND_NAMES[c.dir] }, ico(n))"#, "#1z-7 R1 유형 칩 읽는 이름"),
    ("app", r#"{ class: meta.unread ? "unseen" : null, "data-kind": c.dir },"#, "#1z-7 개요 목록 줄 유형"),
    ("app", r#""data-section": section, "data-kind": c.dir },"#, "#1z-7 사이드바 줄 유형"),
    ("app", r#"el("span", { class: "label" }, lead("palette"), "아이콘 색"),"#, "#1z-7 아이콘 색 줄"),
    ("app", r#"el("span", { class: "label" }, lead("title"), "이름"),"#, "#1z-7 이름도 같은 격자"),
    ("app", r#"button("닫기", () => closePick(page), undefined, "x")"#, "#1z-7 닫기 · #1z-8 상세 닫기(고름 풀기)"),
    ("app", r#""aria-label": "상세 닫기", onclick: () => closePick(page) }, ico("x")),"#, "#1z-8 상세 머리 ✕"),
    ("app", r#"$(opener ?? mdRowId(page, was))?.focus();"#, "#1z-8 닫기 뒤 초점 = 연 곳"),
    ("app", r#"a.setAttribute("aria-current", a.dataset.page === r.page ? "page" : "false");"#, "#1z-8 고른 메뉴 aria-current"),
    ("app", r#"for (const n of settingsTree()) $(n.id).hidden = n.id !== r.page;"#, "#1z-8 고른 화면만"),
    ("view", r#"return { view: "settings", page: settingsTree().some((n) => n.id === m[1]) ? m[1] : "conn", sub: null };"#, "#1z-8 빈 · 모르는 = 연결"),
    ("app", r#"const agentDrafts = new Map();"#, "#1z-8 에이전트 저장 전 값"),
    ("app", r#"if (agentDrafts.get(a.id) === sent) agentDrafts.delete(a.id);"#, "#1z-8 저장 성공이면 초안 지움"),
    ("app", r#"api("GET", "/api/debug"), api("GET", "/api/debug/log")"#, "#1z-8 디버깅 API"),
    ("app", r#"debugCopy(diagText(debugInfo, state.me), "진단 정보 복사")"#, "#1z-8 진단 정보 복사"),
    ("index.html", r#"<pre id="debug_log" class="debug-log-box" tabindex="0" aria-label="로그 끝 200줄"></pre>"#, "#1z-8 로그 끝 200줄"),
    ("index.html", r#"<input type="checkbox" id="notify_inbound" role="switch">"#, "#1z-8 스위치"),
    ("app", r#"const parent = el("input", { type: "checkbox", role: "switch", title: PARENT_NOTE });"#, "#1z-8 상세 스위치"),
    ("app", "agentTile(a.color, \"small\"),", "#1z-7 등록된 에이전트 줄 로봇 타일"),
    ("view", r#"export const AGENT_COLORS = ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"];"#, "#1z-7 아이콘 색 6개"),
    // 1z-8 — 상단 바 버전 · 검색 스위치 · 유형 칩(S1) · 미리보기 2줄 · 세부정보 제목 줄 없음
    ("index.html", r#"<strong class="wordmark">coworkAgent<span id="top_version" class="wordmark-ver"></span></strong>"#, "#1z-8 상단 바 버전"),
    ("app", r#"$("top_version").textContent = `v${s.version}`;"#, "#1z-8 상단 바 버전"),
    ("index.html", r#"<label class="switch-line"><input type="checkbox" id="search_closed" role="switch">나간 대화 포함</label>"#, "#1z-8 검색 스위치"),
    ("index.html", "지운 대화는 검색되지 않습니다", "#1z-8 검색 안내"),
    ("app", "&closed=${$(\"search_closed\").checked}", "#1z-8 검색 스위치 → 서버"),
    ("app", r#"searchMark(row.mark),"#, "#1z-8 검색 줄 칩"),
    ("app.css", r#"input[type="checkbox"][role="switch"] { appearance: none;"#, "#1z-8 스위치 모양"),
    ("app", r#"el("span", { class: "pv-text" },"#, "#1z-8 말풍선 글자 칸"),
    ("app.css", "justify-content: flex-end; gap: var(--sp-3);", "#1z-8 말풍선 사이 여백"),
    ("index.html", r#"<h3><span class="ico ico-info lead" aria-hidden="true"></span>대화 정보</h3>"#, "#1z-10 대화 정보 절 아이콘"),
    ("app", r#"api("GET", `/api/conversations/${conv.id}/info`)"#, "#1z-10 대화 정보 = /info"),
    ("app", r#"lead("copy"), "복사")"#, "#1z-10 실행 명령 복사 단추"),
    ("app", "navigator.clipboard.writeText(text);", "#1z-10 복사 = 전체 명령"),
    ("app", "if (infoView.gen !== gen) return;\n        infoView.key = \"\";", "#1z-10 R1 /info 실패 뒤 다시 읽기(catch 에서 열쇠 비움)"),
    ("app", "if (infoView.gen !== gen) return;", "#1z-10 R2 /info 마지막 읽기 응답만(A → B → A 역전 막음)"),
    ("view", r#"{ icon: "terminal", label: "실행 명령","#, "#1z-10 결정 2 라벨"),
    ("app", "if (!h.querySelector(\".ico\")) h.prepend(lead(n.icon));", "#1z-10 설정 화면 제목 = 메뉴 아이콘"),
    ("app.css", ".set-head h2 .ico.lead { color: var(--muted); }", "#1z-10 설정 화면 제목 아이콘만 강조색 → #1z-13 결정 5 강조색은 상태 · 선택에만"),
    ("app.css", ".ico.lead { margin-right: var(--sp-2);", "#1z-10 글자 앞 아이콘"),
    ("app.css", ".md-detail:has(> .md-pane:not([hidden]), > .md-head) { align-self: start; max-height: 100%; }", "#1z-10 결정 8 상세 상자 = 내용 높이"),
    ("app.css", ".md-body { flex: 0 1 auto; min-height: 0;", "#1z-10 결정 8 단추 줄 = 내용 바로 아래"),
    ("app", r#""primary",
        "save","#, "#1z-10 에이전트 저장 아이콘"),
    ("app", r#""danger",
        "trash","#, "#1z-10 에이전트 삭제 아이콘"),
    ("app", r#"button("이름 바꾸기", (ev) => openGroupRename(g.id, ev.currentTarget), undefined, "pencil"),"#, "#1z-10 그룹 이름 바꾸기 아이콘"),
    ("index.html", r#"<button type="submit" class="primary"><span class="ico ico-send lead" aria-hidden="true"></span>보내기</button>"#, "#1z-10 보내기 아이콘"),
    // 1z-13 — Graphite: 흑연 주 단추 · 청록 강조 하나 · 카드 대신 선
    ("app.css", "button.primary { border: 0; background: var(--primary); color: var(--on-primary);", "#1z-13 결정 5 · 12 주 단추 = 흑연"),
    ("app.css", "  --accent: #2F6F6B;", "#1z-13 결정 12 강조 하나 = 차분한 청록"),
    ("app.css", "  --border-strong: #D4D4D8;", "#1z-13 결정 12 단추 · 입력창 테두리"),
    ("app.css", ".msg.agent .bubble { background: var(--surface); border-color: var(--border); }", "#1z-13 결정 12 에이전트 글 = 흰 면 + 가는 선"),
    ("app.css", "border: 1px solid transparent; border-radius: var(--radius-sm) var(--radius-bubble) var(--radius-bubble) var(--radius-bubble); background: var(--subtle);", "#1z-13 결정 12 사람 글 = 옅은 회색 면"),
    ("app.css", ".tag.agent-tag { background: transparent; color: var(--agent);", "#1z-13 P1 '에이전트' = 청록 작은 글자"),
    ("app.css", ".ask-kind label > span {", "#1z-13 결정 3 질의 / 요청 줄바꿈 뿌리 — 안쪽 아이콘 span 에 칸 여백이 걸리지 않게"),
    ("app.css", r#".seg button[aria-pressed="true"]::after"#, "#1z-13 결정 12 대화 보기 · 원문 보기 = 글자 탭 밑줄"),
    ("index.html", "<span class=\"hint kbd-hint\"><kbd>Enter</kbd> 보내기 · <kbd>Shift+Enter</kbd> 줄바꿈</span>\n              <div class=\"ask-kind\"", "#1z-13 결정 3 입력창 바 = 서식 | 키 안내 · 질의 / 요청 · 보내기"),
    ("index.html", r#"<button type="button" id="new_btn" aria-haspopup="menu""#, "#1z-13 결정 3 · 12 새로 만들기 = 테두리 단추(종 왼쪽 그대로)"),
    ("index.html", r#"<div class="md-foot"><span class="grow"></span><button type="submit" form="invite_form" class="primary">"#, "#1z-13 P3 단추 줄 = 오른쪽 한 묶음"),
    ("index.html", r#"<div id="agent_step2_foot" class="md-foot" hidden><span class="grow"></span><button type="submit" form="agent_form" class="primary">"#, "#1z-13 P3 단추 줄 = 오른쪽 한 묶음"),
    ("app", r#"el("div", { class: "md-foot" }, el("span", { class: "grow" }), ...actions, button("닫기""#, "#1z-13 P3 단추 줄 = 오른쪽 한 묶음 · 순서 그대로"),
    // 1z-13 stage 2 — 머리글자 네모 → #1z-15 개정 ③ 레일 얼굴
    ("app", "export function rowTile(cls, icon, color, group = false) {", "#1z-13 결정 12 줄 타일 = 머리글자 → #1z-15 결정 17 줄 타일 = 레일 얼굴"),
    ("app", r#"rowTile("small", g.icon, g.color, true)"#, "#1z-15 결정 17 그룹설정 줄 = 세 사람"),
    ("app", "railFace(state.settings.me_icon, false, state.settings.me_color, true), tileName(meName)", "#1z-13 결정 12 → #1z-14 결정 1 레일 나 = 사람 그림 → #1z-15 결정 4 고른 색(기본 흑연)"),
    ("app", "railFace(p.icon, false, p.color),", "#1z-13 결정 12 → #1z-14 결정 1 레일 상대 = 사람 그림 → #1z-15 결정 5 타일 색"),
    ("app", "export function railFace(icon, group = false, color, me = false) {", "#1z-14 결정 1 레일 타일 = 그림, 없으면 사람 · 사람 그룹 → #1z-15 결정 5 타일 색 · 18 '나' 흑연"),
    ("view", "  return tileTint(color) || \"me-tile\";", "#1z-15 결정 18 '나' 얼굴 기본 흑연"),
    ("view", "export function tileTint(color) {", "#1z-15 결정 1 · 5 타일 색 = 에이전트 여섯 색(tinted)"),
    ("app", "renderTileColors();", "#1z-15 결정 1 · 3 타일 편집 아이콘 색 단추"),
    ("app", r#"...(tileEdit.kind === "me" ? ["", ...AGENT_COLORS] : AGENT_COLORS).map((c) =>"#, "#1z-15 결정 4 · 6 '나' 는 흑연 단추 더"),
    ("index.html", r#"<span id="tile_colors" class="swatches" role="group" aria-label="아이콘 색"></span>"#, "#1z-15 결정 1 타일 편집 아이콘 색"),
    ("app", r#"el("div", { class: "avatar", "aria-hidden": "true" }, peerFace(conv, conv.dir === "shared" && m.owner !== true))"#, "#1z-15 결정 17 말풍선 상대 아바타 = 그 상대 레일 얼굴(공유 원 상대 = 무채)"),
    ("app", r#"el("div", { class: "avatar", "aria-hidden": "true" }, peerFace(conv))"#, "#1z-15 결정 17 요청 의견 아바타 = 레일 얼굴"),
    ("app", r#"class: `avatar-face st-${st.key}`, "aria-hidden": "true" }, railFace(state.settings.me_icon, false, state.settings.me_color, true)"#, "#1z-15 결정 17 · 18 프로필 얼굴 = 레일 '나'"),
    ("app.css", ".tile-shape.tinted { color: var(--on-dark); }", "#1z-15 결정 5 색 네모 흰 글자"),
    ("app.css", ".tile-shape.tinted.peer-3 { background: var(--peer-3); }", "#1z-15 결정 1 여섯 색 바탕"),
    ("app.css", ".tile-shape.me-tile { background: var(--rail-2); color: var(--on-dark); }", "#1z-13 결정 12 레일 나 = 흑연 → #1z-15 결정 18 '나' 얼굴 전부 흑연"),
    ("app.css", ".tile-shape svg { width: 70%; height: 70%; }", "#1z-15 결정 17 줄 · 말풍선 얼굴 그림 크기"),
    ("app", r#"<circle cx="5.6" cy="8.4" r="2.8" fill="currentColor" opacity=".45"></circle>"#, "#1z-15 결정 7 그룹 = 앞 사람 + 뒤 좌우 둘(왼쪽)"),
    ("app", r#"<circle cx="18.4" cy="8.4" r="2.8" fill="currentColor" opacity=".45"></circle>"#, "#1z-15 결정 7 그룹 = 앞 사람 + 뒤 좌우 둘(오른쪽)"),
    ("app.css", ".rail-btn .tile-shape svg { width: 22px; height: 22px; }", "#1z-14 결정 1 레일 그림 크기"),
    ("app.css", ".tile-shape { display: flex; align-items: center; justify-content: center; width: 100%; height: 100%; border-radius: inherit; background: var(--avatar-bg); color: var(--avatar-text);", "#1z-13 결정 12 머리글자 네모 = 무채 → #1z-15 결정 17 얼굴 네모 무채"),
    ("app.css", ".avatar { flex: 0 0 24px; height: 24px; border-radius: var(--radius-sm);", "#1z-13 결정 12 메시지 아바타 = 24px 네모"),
    // 1z-13 stage 3 — 대화 종류 = 무채 선 아이콘(U8 (가)), 대화 줄 납작하게
    ("view", r#"return conv.dir === "in" ? "bot" : conv.dir === "out" ? "user" : conv.dir === "shared" ? "eye" : null;"#, "#1z-13 U8 (가) · 결정 16 응답 = 로봇 · 문의 = 사람(사람 ↔ 에이전트 구분) · 공유 = 눈"),
    ("app.css", ".kind-chip { flex: none; display: inline-flex; align-items: center; justify-content: center; color: var(--faint); }", "#1z-13 U8 (가) 무채 선 아이콘(색 면 없음)"),
    ("app.css", ".tile-form { flex-direction: column; flex-wrap: nowrap; align-items: stretch;", "#1z-13 stage2-2 타일 편집 이름 칸이 사이드바 밖으로 넘치지 않게 — 전역 form 의 wrap 은 줄 폭을 입력 기본 폭(266px)으로 잡는다"),
    // 1z-13 stage 4 — 메뉴 안 켜기 항목 = 앱 스위치 모양(stage4-2)
    ("app.css", r#".menu button[role="menuitemcheckbox"]::before { content: ""; position: absolute;"#, "#1z-13 stage4-2 메뉴 스위치 = 설정 화면 스위치와 같은 손잡이"),
    // 1z-15 stage 4 — 세부정보 요청 최근 3건 · 더보기, 받은 쪽 멈춘 자동 왕복은 표시 없음
    ("view", "export const REQ_FOLD = 3;", "#1z-15 결정 13 요청 최근 3건"),
    ("view", r#"toggle: over ? (open ? "접기" : "더보기") : null"#, "#1z-15 결정 13 · 14 더보기 · 접기"),
    ("app", "if (reqView.conv !== conv.id) [reqView.conv, reqView.open] = [conv.id, false];", "#1z-15 결정 14 대화를 옮기면 다시 접힘"),
    ("view", "  if (conv.relay_blocked) return null;", "#1z-15 결정 12 '자동 왕복 거절 중' 표시 없음"),
    ("view", r#"export const RELAY_STOPPED = "멈춤 — 이 PC 가 멈춤(재개는 연 쪽)";"#, "#1z-15 결정 16 받은 쪽 멈춤 = 대화 정보 한 줄"),
    ("app", "convInfoRows({ ...d, relay_blocked: conv.relay_blocked }, conv.dir)", "#1z-15 결정 16 대화 정보에 멈춤 상태"),
];

/// 없어야 하는 문구 — (파일 키, 문구, 출처). `a+b` 는 두 파일 모두에 없어야 한다.
#[rustfmt::skip]
const ABSENT: &[(&str, &str, &str)] = &[
    ("index.html", "새 자동 왕복 대화의 상한 기본값", "#13 상한 라벨"),
    ("index.html+app", "페어링 받기", "#15 초대"),
    ("index.html+app", "초대 붙여넣기", "#15 초대"),
    ("index.html+app", "메신저", "#15 초대"),
    ("index.html", "대화 검색", "#15 제목 검색"),
    ("tray.rs", "내 정보 보기", "#17 트레이"),
    ("index.html+app+view", "tab_timeline", "#17 타임라인 탭 없음"),
    ("index.html+app+view", "panel_timeline", "#17 타임라인 탭 없음"),
    ("index.html+app+view", "#t/", "#17 타임라인 탭 없음"),
    ("index.html+app+view", "타임라인", "#17 타임라인 탭 없음"),
    ("index.html+app+view", "완료 표시", "#1d 없어진 문구"),
    ("index.html+app+view", "완료 취소", "#1d 없어진 문구"),
    ("index.html+app+view", "허용 도구", "#1d 없어진 문구"),
    ("index.html+app+view", "model_names", "#1d 없어진 문구"),
    ("view", r#"key: "copy""#, "#1f 작업 막대에 복사 없음"),
    ("index.html", r#"id="side_agents""#, "#1g 사이드바 에이전트 절 없음"),
    ("index.html", r#"id="act_peer""#, "#1g 상대 거르기 칸 없음"),
    ("app", "renderSideAgents", "#1g 사이드바 에이전트 절 없음"),
    ("index.html+app+view", "저장된 항목", "#1h 북마크 항목으로"),
    ("index.html", "tab_sent", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "panel_sent", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "sent_list", "#1j 보낸 메시지 탭 없음"),
    ("index.html", "conv_tag", "#1j 보냄 · 받음 태그 없음"),
    ("index.html", "보낸 메시지", "#1j 보낸 메시지 탭 없음"),
    ("app+view", "kindText", "#1j 방향 라벨 없음"),
    ("app+view", "사람 → 에이전트", "#1j 방향 라벨 없음"),
    ("app+view", "tag: conv.dir", "#1j 보냄 · 받음 태그 없음"),
    ("app+view", "★ 즐겨찾기", "#1j 별 절 없음"),
    ("app+view", "보낸 메시지", "#1j 보낸 메시지 탭 없음"),
    ("view", r#"key: "save""#, "#1j 북마크는 말풍선 아이콘 — 막대에 없음"),
    ("index.html+app+app.css", "focus_chip", "#1j 상단 집중 칩 없음"),
    ("index.html+app+app.css", "status-chip", "#1j 상단 집중 칩 없음"),
    ("index.html+app+app.css", "renderFocusChip", "#1j 상단 집중 칩 없음"),
    ("app.css", ".rail-btn.group { box-shadow", "#1j 겹친 카드 없음"),
    ("view", "집중 — 요청 받지 않음", "#1j 프로필 메뉴는 받기 가능 · 집중 고르기"),
    ("index.html", "주소 · 코드로 직접 추가", "#1j 직접 추가 칸 없음"),
    ("index.html", r#"id="add_form""#, "#1j 직접 추가 칸 없음"),
    ("index.html", "읽기 전용</span>", "#1j 읽기 전용 표시 없음"),
    ("index.html", r#"id="peer_group""#, "#1j 그룹 칸 없음"),
    ("index.html", r#"id="agent_group""#, "#1j 그룹 칸 없음"),
    ("app+view", r#"placeholder: "그룹 없음""#, "#1j 그룹 칸 없음"),
    ("app+view", "add_form", "#1j 직접 추가 칸 없음"),
    ("app+view", "groupFilter", "#1j 그룹 칸 없음"),
    ("app+view", "fillGroupSelect", "#1j 그룹 칸 없음"),
    // 연결 목록의 "읽기 전용" 표시는 없다 — #1o 공유 대화 머리(`logic/conv.js` `shareHead`) · 공유한 대화 상태 줄(#1p C11)은 별개.
    ("app", r#"class: "ro", text: "읽기 전용""#, "#1j 읽기 전용 표시 없음"),
    ("index.html", "상대 에이전트를 누르면 새 대화, 내 에이전트를 누르면 설정으로", "#1k 안내문 없음"),
    ("index.html", "주고받은 글 — 모든 대화, 최신순", "#1k 안내문 없음"),
    ("index.html+app+view", r#"label: "다시 보내기""#, "#1k 다시 보내기 없음"),
    ("index.html+app+view", r#"key === "resend""#, "#1k 다시 보내기 없음"),
    ("index.html+app+view", "다시 보내기로", "#1k 다시 보내기 없음"),
    ("index.html", r#"<section id="me" class="card">"#, "#1l 내 정보 카드 없음"),
    ("index.html", r#"id="agent_q""#, "#1l 에이전트 검색 없음"),
    ("index.html", "<summary>새 에이전트 등록</summary>", "#1l 새 에이전트 등록 없음"),
    ("index.html", r#"<details class="guide">"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", "settings-cols", "#1l 두 열 없음"),
    ("index.html", r#"<p class="hint">한쪽이 초대를 만들어 보내고"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", r#"<p class="hint">페어링된 상대가 부를 수 있는"#, "#1l 늘 보이는 안내 없음"),
    ("index.html", "\"내 정보 → 지문\"", "#1l 지문 확인"),
    ("index.html", r#"width="15" height="15""#, "#1l 아이콘 16px 한 벌"),
    ("app", "agent_q", "#1l 에이전트 검색 없음"),
    ("view", "자동 왕복 대화", "#1l 새로 만들기 메뉴에 자동 왕복 대화 없음"),
    ("view", r#"{ id: "me", label: "내 정보" }"#, "#1l 내 정보 → 사용자 정보"),
    ("app.css", "color-mix(", "#1l macOS 12 Safari 15 에 없는 color-mix"),
    ("app.css", "border-right: 3px solid var(--accent)", "#1p 말풍선 옆 띠 없음"),
    ("app.css", "border-left: 3px solid var(--tone", "#1p 말풍선 옆 띠 없음"),
    ("index.html", r#"<span class="hint mono kbd-hint">"#, "#1p 안내 한글은 본문 글꼴"),
    ("app.css", ".msg.side-mine .bubble { background: var(--accent); color: var(--on-accent);", "#1o 옛 말풍선"),
    ("app.css", "opacity: 0.12;", "#1o 옛 말풍선"),
    ("index.html+app", "상대별 대화 요약 — 대화 수 · 안 읽음 · 대기 요청 · 최근 활동, 누르면 그 대화로", "#1o 설명 지우기"),
    ("index.html+app", "실행 없이 모인 요청 — 받은 요청은 수용 · 반려 · 완료로 답하고, 보낸 요청은 상대가 바꾼 상태를 봅니다", "#1o 설명 지우기"),
    ("index.html+app", "☆ 로 넣은 대화방 — 이 PC 에만, 대화방 줄의 ☆ 로 넣고 뺍니다", "#1o 설명 지우기"),
    ("index.html+app", "메시지 작업 '북마크' 로 모은 글 — 이 PC 에만, 북마크한 순서 최신부터", "#1o 설명 지우기"),
    ("index.html+app", "— 메시지 작업 '북마크' 로 모읍니다", "#1o 설명 지우기"),
    ("index.html+app", "이 PC 에서만 보입니다. 그림은 가운데를 정사각형으로 잘라 작게 줄여 저장합니다.", "#1o 설명 지우기"),
    ("index.html+app", "의 에이전트 — 누르면 새 대화", "#1o 설명 지우기"),
    ("index.html+app", "text: `최근 ${formatTime(g.last)}`", "#1o 설명 지우기"),
    ("index.html+app", "text: `대화 ${g.convs}`", "#1o 설명 지우기"),
    // 1p — 문구 표(plan-copy)로 바뀐 잠금 문구의 옛 꼴.
    ("index.html", "대화 턴에 대한 상한 기본값", "#1p 문구"),
    ("index.html", "에서 만든 초대 메시지를 통째로 붙여 넣으세요", "#1p 문구"),
    ("index.html", r#"<label for="conv_search">"#, "#1p 문구 — 라벨과 같은 placeholder 하나"),
    ("index.html", "검색 — 메시지 · 대화 · 사람 · 에이전트", "#1p 문구"),
    ("index.html", "북마크 항목</strong>", "#1p 문구 T2"),
    ("index.html", "<h3>북마크 항목", "#1p 문구 T2"),
    ("view", r#"label: "북마크 항목""#, "#1p 문구 T2"),
    ("view", r#"label: "주고받은 글""#, "#1p 문구 T7"),
    ("app", r#""주고받은 글 없음""#, "#1p 문구 T7"),
    ("view", r#"label: "표시 이름 · 아이콘""#, "#1p 문구"),
    ("index.html+app+view", r#""환경설정""#, "#1p 문구 T5"),
    ("index.html+app+view", r#""실행 상세""#, "#1p 문구 T1"),
    ("index.html", ">실행 상세<", "#1p 문구 T1"),
    ("view", r#"title: "agent 대화""#, "#1p 문구 T3"),
    ("app", "에게 보내기 — markdown 가능", "#1p 문구"),
    ("web", "실행 없이 상대 요청 목록에 등록", "#1p 문구"),
    ("index.html", "편집 모드 — ", "#1p 문구"),
    ("web", "@ 로 상대 에이전트 부르기", "#1v 입력창 안내 삭제"),
    ("app", " 에게 보내기`", "#1v 입력창 placeholder = \"메시지 입력\" 고정(적대적 평가 R2)"),
    ("app", r#"$("reply_text").placeholder"#, "#1v placeholder 다시 넣지 않음(적대적 평가 R2)"),
    ("index.html", r#"id="edit_btn""#, "#1v 웹UI 편집 모드 삭제"),
    ("index.html", r#"id="edit_bar""#, "#1v 웹UI 편집 모드 삭제"),
    ("index.html", r#"id="scope_dialog""#, "#1v 웹UI 편집 모드 삭제"),
    ("app", "function setEditMode(", "#1v 웹UI 편집 모드 삭제"),
    ("app", r#""본문 편집""#, "#1v 웹UI 편집 모드 삭제"),
    ("app", "`메모: ${m.req_note}`", "#1v 메모는 피드백 말풍선으로"),
    ("app", "`메모: ${r.note}`", "#1z 메모 → 의견"),
    ("app", "요청 피드백 · ", "#1z 피드백 → 의견"),
    ("app", "피드백(선택)", "#1z 피드백 → 의견"),
    ("app", r#"text: "피드백""#, "#1z 피드백 → 의견"),
    ("index.html", r#"<option value="normal">보통</option>"#, "#1z 타일 크기 보통 → 크게"),
    ("app.css", "min-height: 76px", "#1z 입력창 글 칸 44px"),
    ("web", "연결 목록", "#1z 연결 목록 → 그룹설정"),
    ("app", r#"$(box).addEventListener("dragstart""#, "#1z 레일 HTML 끌기 없음"),
    ("app", r#""복사했다""#, "#1p 문구"),
    ("index.html", ">그룹 +<", "#1p 문구"),
    ("app", "대화는 지우지 않습니다", "#1p 문구"),
    ("app", "로 자리를 바꿉니다", "#1p 문구 — 자리 옮기기는 타일 메뉴 · 사용자 가이드"),
    ("app", "북마크 항목에 모았다", "#1p 문구"),
    ("app", "${name} — ${st.text}", "#1p 문구"),
    ("app", "상태 — 아이콘 테두리 색", "#1p 문구"),
    ("app", "사람 모양으로 보입니다", "#1p 문구"),
    ("view", r#"hint: "새 요청을 받는다""#, "#1p 문구"),
    ("view", r#"hint: "새 요청을 거절""#, "#1p 문구"),
    ("index.html", ">추가 · 승인<", "#1p 문구"),
    ("app", "체크하면 넣고, 풀면 뺀다", "#1p 문구"),
    ("view", "개가 함께 보입니다", "#1p 문구"),
    ("index.html", "받은 쪽이 붙여 넣으면 연결됩니다", "#1p 문구"),
    ("web", "화면 — 테마 · 타일 순서", "#1p 문구"),
    ("index.html", "프로필 → 사용자 정보 → 지문", "#1p 문구"),
    ("index.html", "자동 압축 — 컨텍스트 80% 넘으면", "#1p 문구"),
    ("view", "압축 — 요약 후 새 세션", "#1p 문구 Y"),
    ("view", "비우기 — 세션만", "#1p 문구 Y"),
    ("view", "${name} — ", "#1p 문구 Y — 컨텍스트 구분선은 ' · '"),
    ("view", ") — 다시 누르면 다시 요청", "#1p 문구 Y"),
    ("view", "요약 실행 실패 — ", "#1p 문구 Y"),
    ("view", "항상 사용 — ", "#1p 문구 Y"),
    ("view", "연결이 끊겼습니다 — ", "#1p 문구 Y"),
    // 1q — 옛 자리
    ("app", r#"const bar = el("div", { class: "actions" });"#, "#1q 요청 단추 우하단 자리 없음"),
    ("app", r#"reqBox: "theirs""#, "#1q 처음 화면 = 개요"),
    ("index.html", r#"<select id="act_kind">"#, "#1q 내 활동 거르기 = 탭"),
    ("index.html+view", "자동 왕복 시작", "#1q 내 활동에 자동 왕복 없음"),
    ("index.html+view", "사람 메시지</option>", "#1q 내 활동 옛 거르기"),
    // 1r — 옛 자리
    ("index.html", r#"<label for="chat_relay_agent">자동 왕복</label>"#, "#1r 새 대화에 자동 왕복 없음"),
    ("index.html", "자동 왕복 상한</label>", "#1r 새 대화에 자동 왕복 없음"),
    ("app", "요청으로는 자동 왕복을 열 수 없습니다", "#1r 새 대화에 자동 왕복 없음"),
    ("app", r#"text: "관리","#, "#1r 관리 = 연필"),
    ("app.css", ".req-box .peer-fold > summary { padding-left", "#1r 들여쓰기는 공통 규칙"),
    ("app", "showAccept(true);", "#1r 수락 칸은 라우터만 펼친다"),
    ("app", r#"$("window_btn").disabled = true;"#, "#1r R2 초대 생성 중에도 초점을 받는다"),
    // 1s — 접이식 하위 줄 세로선 없음(들여쓰기만)
    ("app.css", ".peer-fold > :not(summary) { margin-left: var(--sp-5); border-left", "#1s 세로선 제거"),
    // 1w — 상대 단위 그룹 소속 없음(D37 개정)
    ("app", "/api/groups/${id}/peers", "#1w 상대 소속 API 없음"),
    ("app", "/api/peers/${d.peer}/groups", "#1w 상대 소속 API 없음"),
    ("app", "peerGroupMenu", "#1w 레일 상대 타일 '그룹' 메뉴 없음"),
    ("app", r#"text: "그룹 ▾""#, "#1w 설정 '그룹 ▾' 없음"),
    ("view", "그룹에 없는 연결", "#1w '사용자 목록' 으로"),
    ("view", "NO_PEER_GROUP", "#1w '사용자 목록' 으로"),
    // 1x — 세션 생성 · 기록 삭제 · 나가기 · 사이드바 머리
    ("index.html+app+view", "새 대화\"", "#1x 새 대화 → 세션 생성"),
    ("index.html", "실행하지 않고 상대의 요청 목록에 올립니다", "#1x 요청 안내 지움"),
    ("index.html", r#"id="chat_text""#, "#1x 세션 생성에 메시지 없음"),
    ("index.html", r#"id="chat_ask_label""#, "#1x 세션 생성에 메시지 종류 없음"),
    ("app", "ask_hint", "#1x 요청 안내 지움"),
    ("view", r#"label: "삭제""#, "#1x 대화 메뉴 삭제 → 기록 삭제 · 나가기"),
    ("index.html+app+view", "이력유지", "#1x 나가기 하위 메뉴 없음"),
    ("index.html+app+view", "이력삭제", "#1x 기록 삭제로 하나"),
    ("app", r#""지문 " + state.me.fingerprint"#, "#1x 사이드바 머리 지문 없음"),
    ("app", "starButton(c, label.title),", "#1x 대화 줄 ☆ 없음"),
    ("app", r#"draggable: "true""#, "#1z-2 HTML 끌어 놓기 없음(설정 대화 줄도 포인터)"),
    ("app", r#"class: "room-preview""#, "#1z-2 대화 줄 2행(미리보기) 없음"),
    ("view", r#"label: "에이전트 대화""#, "#1z-2 메뉴 이름 에이전트 응답"),
    ("index.html", r#"aria-label="초대 수락">+</button>"#, "#1z-2 레일 + 글자"),
    ("app", "groupSettingsListEl(", "#1z-2 시안 A 목록 없음"),
    ("app", r#"kvTable([["이름", a.name]])"#, "#1z-7 이름 표 → 격자"),
    ("index.html", "이 PC 에만 저장", "#1z-8 세부정보 대화 정보 제목 줄 삭제"),
    ("app.css", "--pv-lines", "#1z-8 말풍선 줄 수 배율 없음"),
    ("app", "function manageItem(", "#1z-8 연필 펼침 → 목록 · 상세"),
    ("app", "gsGroupsEl", "#1z-8 그룹 카드 격자 없음"),
    ("app", "gsUsersEl", "#1z-8 사용자 표 → 연결 목록"),
    ("app", "{ join: true }", "#1z-8 설정 화면 대화 끌기 없음"),
    ("app", "gsJoin", "#1z-8 설정 화면 대화 끌기 없음"),
    ("app+view", "convCopy", "#1z-8 끌어 놓기 복사 함수 없음(넣기 단추)"),
    ("view", "USER_LIST", "#1z-8 사용자 목록 절 없음"),
    ("view", "children: [", "#1z-8 설정 메뉴 하위 없음"),
    ("index.html", r#"class="card"#, "#1z-8 설정 카드 한 열 없음"),
    ("app.css", "max-width: 1000px", "#1z-8 설정 본문 폭 제한 없음"),
    ("app", "scrollIntoView({ block: \"start\" });\n  applyFocus", "#1z-8 설정 절 스크롤 없음"),
    ("index.html+app", "info_defaults", "#1z-10 결정 1 실행 기본값 줄 없음"),
    ("index.html+app", "info_peer", "#1z-10 결정 1 상대 줄 없음"),
    ("index.html", ">+ 사용자<", "#1z-10 + 글자 → 아이콘"),
    ("app.css", "--pv-zoom", "#1z-11 결정 5 절 15% 확대 없음"),
    ("app", "pv-unread", "#1z-11 결정 6 카드 바닥 안 읽음 배지 없음"),
    ("app", r#"rowTile("small", p?.icon, s.name)"#, "#1z-11 미리보기 머리 상대 타일 없음(#1z-13 줄 타일 모양)"),
    ("app.css", ".pv-card.unseen { outline", "#1z-11 결정 6 안 읽음 윤곽선 없음"),
    // 1z-13 — Graphite
    ("index.html", r#"id="new_btn" class="primary""#, "#1z-13 결정 5 새로 만들기는 주 단추 색이 아니다"),
    ("app.css", "--pv-bg", "#1z-13 결정 6 미리보기 = 공용 토큰(종이 톤 없음)"),
    ("app.css", "--bubble-mine-alpha", "#1z-13 결정 12 말풍선 색 층 없음"),
    ("app.css", "--bubble-theirs-alpha", "#1z-13 결정 12 말풍선 색 층 없음"),
    ("app.css", ".msg[data-tone=", "#1z-13 결정 12 말풍선 색 층 없음"),
    ("app.css", ".msg.side-mine .bubble a { color: var(--sent); }", "#1z-13 결정 12 내 글 링크 = 전역 링크 색"),
    ("app.css", ".msg.side-theirs:not(.agent) .bubble { background: var(--recv-bg); }", "#1z-13 P4 상대 사람 글 주황 면 없음"),
    ("app.css", ".ask-kind span { display: inline-flex;", "#1z-13 결정 3 안쪽 아이콘까지 칸 여백을 받던 선택자"),
    ("app.css", "button:hover { border-color: var(--accent); }", "#1z-13 결정 5 올림은 강조색이 아니다"),
    ("app.css", r#"#act_tabs a[data-kind="request"][aria-current="page"] { background"#, "#1z-13 P4 내 활동 탭 = 글자 탭(종류 색 바탕 없음)"),
    // 1z-13 stage 2 — 사람 아이콘 아바타 · 타일 다섯 색 없음
    ("app+view", "colorOf", "#1z-13 결정 12 상대 · 그룹 타일 다섯 색 없음"),
    ("app", "tileColor", "#1z-13 결정 12 상대 타일 색 없음"),
    ("app", "groupColor", "#1z-13 결정 12 그룹 타일 색 없음"),
    ("app", "data-tone", "#1z-13 결정 12 말풍선 색 층 없음"),
    // 1z-13 stage 3 — 종류 색 · 테두리 · 점선 · 대화 줄 카드 없음
    ("app.css", "--conv-", "#1z-13 U8 (가) 종류 색 토큰 없음"),
    ("app.css", "var(--kind", "#1z-13 U8 (가) 종류 색 면 · 테두리 없음"),
    ("app.css", r#".pv-card[data-kind="shared"] { border-style"#, "#1z-13 U8 (가) 공유 점선 없음(#1z-14 카드 관전 테두리 = 호박 실선)"),
    ("app", "tileFace(state.settings.me_icon", "#1z-14 결정 1 레일은 머리글자 아님"),
    ("app.css", ".room-av", "#1z-13 개정 ⑥ 종류 칩 원 없음"),
    ("app.css", "#sidebar { background: var(--subtle); }", "#1z-13 개정 ② 사이드바 바탕 덮어쓰기 없음"),
    ("app.css", ".side-list li:not(.hint) { background: var(--surface);", "#1z-13 개정 ② 대화 줄 카드 없음"),
    ("app.css", "box-shadow: inset 0 0 0 1px var(--accent)", "#1z-13 P2 고른 줄 안쪽 테두리 없음"),
    // 1z-13 stage 4 — 상태 색 띠 · 손잡이 없는 메뉴 알약 없음
    ("app.css", "border-top: 3px solid var(--muted)", "#1z-13 stage4-1 요청 보드 칸 위 상태 색 띠 없음"),
    ("app.css", "border-left: 4px solid var(--muted)", "#1z-13 stage4-1 요청 보드 카드 왼쪽 상태 색 띠 없음"),
    ("app.css", "border-left: 3px solid var(--muted)", "#1z-13 stage4-1 의견 상자 왼쪽 상태 색 띠 없음"),
    ("app.css", ".menu button[aria-checked]::after", "#1z-13 stage4-2 손잡이 없는 메뉴 알약 없음"),
    ("app.css", ".avatar-face.st-offline { color:", "#1z-13 R1-1 연결 끊김 머리글자도 --avatar-text(상태는 점 — 흐린 글자 대비 4.16)"),
    ("index.html+app+view", "자동 왕복 거절 중", "#1z-15 결정 12 머리글 문구 없음"),
    ("index.html+app", "detail_relay", "#1z-15 결정 12 세부정보 '자동 왕복' 절 없음"),
    ("app+view", "initialOf", "#1z-15 결정 17 머리글자 네모 없음(레일 얼굴)"),
    ("app", "tileFace", "#1z-15 결정 17 머리글자 줄 타일 없음"),
    ("app", "peerTint", "#1z-15 결정 17 말풍선 색만 입히기 없음(얼굴째)"),
];

/// 정확한 개수 — (파일 키, 문구, 개수, 출처).
#[rustfmt::skip]
const COUNTS: &[(&str, &str, usize, &str)] = &[
    ("tray.rs", "MenuItem::new(", 4, "#17 정상 모드 열기 · 종료 + 오류 모드 오류 보기 · 종료"),
    ("index.html", r#"id="raw_btn""#, 1, "#17 원문 보기는 머리글 하나"),
    ("app.css", "\n.ico { ", 1, "#1l 아이콘 크기 정의는 한 곳(16px)"),
    ("app.css", "\n.md {", 1, "#1z-8 .md = 대화 본문 마크다운만(설정 목록 · 상세 2단은 .md-split — 이름이 겹치면 본문이 2단으로 갈림)"),
];

#[test]
fn literals_present() {
    let missing: Vec<String> = PRESENT
        .iter()
        // 줄바꿈이 든 문구 — 체크아웃이 CRLF(core.autocrlf)여도 같게 본다.
        .filter(|(key, lit, _)| !sources(key).iter().any(|(_, text)| text.replace("\r\n", "\n").contains(lit)))
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

/// 개수 표 + 개수로만 말할 수 있는 규칙 — '받은 메시지' 라벨 없음('주고받은 메시지' 안의 것만, #17 · #1p T7).
#[test]
fn literal_counts() {
    for (key, lit, n, why) in COUNTS {
        for (file, text) in sources(key) {
            assert_eq!(text.matches(lit).count(), *n, "{why} — {file}: {lit}");
        }
    }
    for (file, text) in sources("index.html+app+view") {
        assert_eq!(
            text.matches("받은 메시지").count(),
            text.matches("주고받은 메시지").count(),
            "#17 '받은 메시지' 라벨 없음 — {file}"
        );
    }
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

/// 토큰 대비 — 두 테마 × 36 짝(글자 · 강조 글자 · 주 단추 · 배지 · 레일 · 머리글자 · 상태(올림 바탕 · 칩 바탕 포함) · 위험 · 범주 칩) + 에이전트 아이콘 6색 흰 아이콘 = 78 경우 모두 4.5:1 이상.
#[test]
fn token_contrast() {
    const PAIRS: [(&str, &str); 36] = [
        ("text", "bg"), ("text", "surface"), ("text", "surface-2"), ("text", "subtle"), ("text", "accent-soft"),
        ("muted", "bg"), ("muted", "surface"), ("muted", "surface-2"), ("muted", "subtle"), ("muted", "accent-soft"),
        ("faint", "bg"), ("faint", "surface"), ("faint", "surface-2"), ("faint", "subtle"),
        ("accent", "bg"), ("accent", "surface"), ("accent", "surface-2"), ("accent", "subtle"), ("accent", "accent-soft"),
        ("on-primary", "primary"), ("on-primary", "primary-hover"), ("on-accent", "accent"),
        ("rail-text", "rail"), ("on-dark", "rail-2"), ("avatar-text", "avatar-bg"),
        ("run", "surface"), ("ok", "surface"), ("danger", "surface"), ("danger", "danger-soft"), ("on-danger", "danger"),
        // 컨텍스트 단추 경고 · 위험 글자 + 도구 단추 올림 바탕, 요청 태그(`.tag.ask`) 대기 · 완료 글자 + 칩 바탕
        ("run", "surface-2"), ("danger", "surface-2"), ("run", "subtle"), ("ok", "subtle"),
        ("sent", "sent-bg"), ("recv", "recv-bg"),
    ];
    let mut cases = 0;
    for dark in [false, true] {
        // `--on-dark` 는 두 테마가 같다(다크 짝 없음).
        let t = |name: &str| {
            token(&if dark && name != "on-dark" {
                format!("dark-{name}")
            } else {
                name.to_string()
            })
        };
        for (fg, bg) in PAIRS {
            let r = contrast(t(fg), t(bg));
            assert!(
                r >= 4.5,
                "{} {fg} / {bg} {r:.2}",
                if dark { "다크" } else { "라이트" }
            );
            cases += 1;
        }
    }
    for tile in ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"] {
        let r = contrast(token("on-dark"), token(tile));
        assert!(r >= 4.5, "에이전트 아이콘 {tile} {r:.2}");
        cases += 1;
    }
    // 미리보기 카드 테두리(글자가 아닌 선 — 3:1): 내 대화 남색(청록 = --accent · 호박 = --run 은 위 4.5:1 짝)
    for dark in [false, true] {
        let p = if dark { "dark-" } else { "" };
        for bg in ["bg", "surface"] {
            let r = contrast(token(&format!("{p}card-mine")), token(&format!("{p}{bg}")));
            assert!(r >= 3.0, "카드 테두리 card-mine / {bg} {r:.2}");
            cases += 1;
        }
    }
    // 타일 색(나 · 상대 · 그룹 — 에이전트 여섯 색, 두 테마 같은 값) 위 흰 머리글자(4.5:1)
    for c in ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"] {
        let r = contrast(token("on-dark"), token(c));
        assert!(r >= 4.5, "머리글자 on-dark / {c} {r:.2}");
        cases += 1;
    }
    assert_eq!(cases, 88);
}
