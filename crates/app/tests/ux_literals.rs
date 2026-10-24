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
    ("web", "글 종류", "#18 문구"),
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
    ("app.js", r#"iconButton(COPY_SVG, "복사""#, "#1f 복사"),
    ("app.js", r#""복사했습니다""#, "#1f 복사"),
    ("view.js", r#""(편집됨)""#, "#1f 편집됨"),
    // 1g — 레일 범위('나' 메뉴 · 상대 타일)
    ("view.js", r##"{ label: "요청", href: "#requests", count: waiting, icon: "inbox" },"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "북마크", href: "#saved", icon: "bookmark" }"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "즐겨찾기", href: "#starred", count: starred, icon: "star" }"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "에이전트", icon: "bot", head: true }"##, "#1v 메뉴 머리"),
    ("view.js", r##"{ label: "등록된 에이전트", href: "#agents", icon: "bot", sub: true }"##, "#1v 하위 메뉴"),
    ("view.js", r##"...(me ? [{ label: "에이전트 응답", href: "#agents/convs", icon: "chat", sub: true, count: agentUnread }] : [])"##, "#1v 하위 메뉴 · #1z-2 에이전트 응답 '나' 에만"),
    ("index.html", r#"<button id="req_view_list" type="button" aria-pressed="true"><span class="ico ico-list lead" aria-hidden="true"></span>목록</button>"#, "#1v 요청 보기"),
    ("index.html", r#"<button id="req_view_board" type="button" aria-pressed="false"><span class="ico ico-grid lead" aria-hidden="true"></span>보드</button>"#, "#1v 요청 보기"),
    ("app.js", r#"`요청 의견 · ${formatTime(f.at)}`"#, "#1v 의견 말풍선(#1z 용어)"),
    ("app.js", r#"placeholder: "의견(선택) · 요청자에게 보입니다""#, "#1z 의견 칸"),
    ("app.js", r#"text: "의견" })"#, "#1z 요청 카드 의견"),
    ("app.js", r#"class: "tag agent-tag", text: "에이전트""#, "#1z 에이전트 칩"),
    ("app.js", r#"el("div", { class: "avatar agent-av", "aria-hidden": "true" }, ico("bot"))"#, "#1z 에이전트 아이콘"),
    ("app.css", "--chat-gutter: max(48px, 8%);", "#1z 말풍선 영역 · 입력창 좌우 여백"),
    ("app.css", "padding: var(--sp-5) var(--chat-gutter) 0;", "#1z 말풍선 영역 여백 · 스크롤 밖 간격"),
    ("app.css", ".composer { margin: var(--sp-5) var(--chat-gutter) var(--sp-4);", "#1z 입력창 여백"),
    ("index.html", r#"<div id="rail_resize" role="separator""#, "#1z 레일 손잡이"),
    ("index.html", r#"<option value="normal">크게</option>"#, "#1z 타일 크기 표시"),
    ("view.js", r#"{ id: "groups", label: "그룹설정", icon: "users" },"#, "#1z 그룹설정 · #1z-8 메뉴 8개"),
    ("index.html", r#"<h2 id="groups_h">그룹설정<span"#, "#1z 그룹설정 · #1z-8 화면 머리"),
    ("app.js", "onpointerdown: tileGrab,", "#1z 레일 타일 포인터 끌기"),
    ("app.js", "function renderConnPage(force) {", "#1z-8 연결 = 사용자 목록 · 상세(시안 B)"),
    ("app.js", "function renderGroupPage(force) {", "#1z-8 그룹설정 = 그룹 목록 · 상세(시안 B)"),
    ("app.js", r#"el("img", { src: icon, alt: "", draggable: "false" })"#, "#1z R2 타일 그림 기본 끌기 없음"),
    ("app.js", r#"$("rail").addEventListener("dragstart", (ev) => ev.preventDefault());"#, "#1z R2 레일 HTML 끌기 막음"),
    ("app.js", r#"if (p) n.replaceWith(gsDot(p));"#, "#1z R2 그룹설정 접속 점 갱신"),
    ("view.js", r##"[{ label: "개요", href: "#", icon: "home" }]"##, "#1g 메뉴"),
    ("view.js", r##"{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }"##, "#1g 메뉴"),
    ("index.html", r#"<ol id="home_summary" class="rows"></ol>"#, "#1g 개요"),
    ("app.js", "{ ui_folded: keys }", "#1g 접힘"),
    ("app.js", "foldSaver.push([...state.folded])", "#1g 접힘"),
    ("app.js", "`대기 요청 ${g.waiting}`", "#1g 개요"),
    // 1i — 레일 그룹 · 타일 메뉴 · 끌어 놓기
    ("index.html", r#"id="rail_group_add""#, "#1i 그룹"),
    ("index.html", r#"<span class="ico ico-folder-plus lead" aria-hidden="true"></span>그룹</button>"#, "#1i 그룹 · #1z-10 + 글자 → 아이콘"),
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
    ("app.js", r#"text: dir < 0 ? "Alt + ↑" : "Alt + ↓""#, "#1i 자리 옮기기"),
    ("view.js", "번째로 옮김", "#1i 자리 옮기기"),
    // 1h — 안정화(북마크 · 개요 · 구분선 · 프로필 알약)
    ("index.html", r#"<strong><span class="ico ico-bookmark" aria-hidden="true"></span>북마크</strong>"#, "#1h 북마크"),
    ("index.html", r#"<h3><span class="ico ico-bookmark lead" aria-hidden="true"></span>북마크 <span id="detail_saved_n""#, "#1h 북마크"),
    ("index.html", r#"<strong><span class="ico ico-home" aria-hidden="true"></span><span id="home_title">개요</span></strong>"#, "#1h 개요"),
    ("app.js", r#"act(on ? "북마크" : "북마크 해제""#, "#1h 북마크"),
    ("app.js", r#""북마크했습니다""#, "#1h 북마크"),
    ("app.js", r#"el("li", { class: "nav-sep", role: "separator" })"#, "#1h 구분선"),
    ("app.js", r#"el("span", { class: "avatar-name", text: name })"#, "#1h 프로필"),
    ("app.js", "`${name} · ${st.text}`", "#1h 프로필"),
    ("app.js", r#": "개요";"#, "#1h 개요"),
    ("app.js", r#"peerAct("연결 설정 저장""#, "#1h 북마크가 아닌 저장"),
    ("app.js", r#"iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크""#, "#1h 북마크"),
    ("app.css", ".side-nav .nav-sep {", "#1h 구분선"),
    ("app.css", ".avatar-name { max-width: 12em;", "#1h 프로필"),
    // 1j — 메신저형 대화 · 메뉴 · 상태 색 · 레일 A안 · 상대 그룹 · 설정
    ("index.html", r#"<button id="view_btn" type="button" aria-pressed="true"><span class="ico ico-chat lead" aria-hidden="true"></span>대화 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<button id="raw_btn" type="button" aria-pressed="false"><span class="ico ico-code lead" aria-hidden="true"></span>원문 보기</button>"#, "#1j 머리"),
    ("index.html", r#"<span id="saved_n" class="count"></span>"#, "#1j 머리"),
    ("index.html", r#"id="star_btn" class="star-btn" aria-label="즐겨찾기""#, "#1j 별"),
    ("app.js", "side-${side}", "#1j 말풍선"),
    ("app.js", r#"class: "day-break""#, "#1j 말풍선"),
    ("app.js", r#"kindChip(c, "room-av agent-av"),"#, "#1z-2 대화 줄 아이콘 · #1z-7 유형별(로봇 · 사람)"),
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
    ("app.js", r#"el("span", { text: "그룹" })"#, "#1j 상대 그룹"),
    ("app.js", r#"convAct(
                        "넣기","#, "#1z-8 그룹 넣기 = 단추(설정 끌기 없음)"),
    ("app.js", r#"convAct(
                    "빼기","#, "#1z-8 그룹 빼기 = 단추"),
    ("app.js", "function openGroupRename(", "#1j 상대 그룹"),
    ("app.js", "function groupDeleteForm(", "#1j 상대 그룹"),
    // 1k — 다듬기
    ("index.html", r#"<button type="button" id="accept_btn" aria-expanded="false" aria-controls="invite_accept">"#, "#1k 설정 · #1z-8 연결 머리 단추"),
    ("index.html", r#"<section id="invite_accept" class="md-pane" aria-labelledby="invite_accept_h" hidden>"#, "#1k 설정 · #1z-8 초대 수락 = 상세 칸"),
    ("index.html", r#"<button type="button" id="agent_new_btn" class="primary" aria-expanded="false" aria-controls="agent_new"><span class="ico ico-plus" aria-hidden="true"></span>에이전트 등록</button>"#, "#1k 설정"),
    ("index.html", "초대를 만들어 보내면 상대가 붙여 넣어 연결합니다. 5분 안에 들어온 첫 요청 1건은 승인 없이 등록되니 모르는 연결은 연결에서 지우세요.", "#1k 설정(#1z-8 사용자 표 = 연결)"),
    ("app.js", "function mdRow(page, id, face, name, sub, ...end) {", "#1k 관리 줄 → #1z-8 목록 줄 고르기"),
    ("app.js", r#"class: "md-row", "aria-pressed": String(pick[page] === id), onclick: () => setPick(page, id) },"#, "#1z-8 줄 고르기(연필 없음 — #1r 연필 개정)"),
    ("app.js", "`${a.name} 편집`", "#1k 에이전트"),
    ("app.js", "function editAgent(", "#1k 에이전트"),
    ("app.js", r#""data-peer-sub": String(p.id)"#, "#1k 관리 줄"),
    ("app.js", "function peerSub(", "#1k 관리 줄"),
    ("app.js", r#"$("conn_user").replaceChildren(...(p ? userDetail(p, convs) : []));"#, "#1z-2 사용자 펼침 → #1z-8 오른쪽 상세"),
    // 1w — 대화 나가기 · 삭제, 그룹 = 대화만 · 설정 연결 목록
    ("view.js", r#"{ key: "leave", label: "나가기", icon: "exit", danger: true, confirm: LEAVE_CONFIRM }"#, "#1x 대화 메뉴"),
    ("view.js", r#"{ key: "clear", label: "기록 삭제", icon: "trash", sub }"#, "#1x 대화 메뉴"),
    ("view.js", r#"[{ key: "clear_mine", label: "내 글만 삭제" }]"#, "#1x 기록 삭제"),
    ("view.js", r#"{ key: "clear_all", label: "모든 사용자 글 삭제" }"#, "#1x 기록 삭제"),
    ("view.js", r#"export const LEAVE_CONFIRM = "대화방이 목록에서 삭제됩니다.";"#, "#1x 나가기 확인(사용자 원문)"),
    ("view.js", r#"export const SESSION_NEW = "세션 생성";"#, "#1x 세션 생성"),
    ("view.js", r#"if (m.refused === "left") return "상대가 대화를 나갔습니다";"#, "#1w 거절 사유"),
    ("view.js", r#"if (c.left_at != null) return "나감";"#, "#1w 닫힌 대화"),
    ("view.js", r#"return c.peer_left_at != null ? "상대가 나감" : "";"#, "#1w 닫힌 대화"),
    ("app.js", r#""이 PC 의 대화 기록을 지웁니다. 대화는 남습니다.""#, "#1x 기록 삭제 확인"),
    ("app.js", r#""나와 상대 PC 의 대화 기록을 모두 지웁니다. 대화는 남습니다.""#, "#1x 기록 삭제 확인"),
    ("index.html", r#"<h1>세션 생성</h1>"#, "#1x 세션 생성"),
    ("index.html", r#"<button type="submit" class="primary">생성</button>"#, "#1x 세션 생성"),
    ("index.html", r#"<label for="ui_tile_size"><span class="ico ico-resize lead" aria-hidden="true"></span>타일 크기</label>"#, "#1x 타일 크기"),
    ("index.html", r#"<label for="notify_sound"><span class="ico ico-volume lead" aria-hidden="true"></span>효과음</label>"#, "#1x 효과음"),
    ("index.html", r#"<html lang="ko" data-theme="light">"#, "#1x 기본 테마 라이트"),
    ("index.html", r#"<label for="req_archive">archive 항목 보기</label>"#, "#1y 요청 archive(사용자 원문)"),
    ("index.html", r#"<label class="head-pick">기간 <select id="ui_peer_weeks" data-num></select></label>"#, "#1y 주고받은 메시지 기간"),
    ("view.js", r#"{ key: "unarchive", label: "archive 해제" }"#, "#1y 요청 archive"),
    ("view.js", r#"{ key: "archive", label: "archive" }"#, "#1y 요청 archive"),
    ("view.js", "label: `${n}주`", "#1y 기간 1 ~ 12주"),
    ("app.js", "if (await changeGroup(g.id, [c.id], [])) say(`그룹 ${g.name} 에 넣었습니다`);", "#1w 넣기 = 복사(다른 그룹 · 원래 자리 그대로) · #1z-8 단추"),
    ("app.css", ".ico-exit {", "#1w 아이콘"),
    ("app.js", "{ class: `has-mark pm-row pm-${kind}` },", "#1z-2 주고받은 메시지 시안 Peer"),
    ("app.js", "{ class: `pm-av pm-av-${kind}`, role: \"img\", \"aria-label\": s.label, title: s.label }", "#1z-2 원형 아바타"),
    ("app.js", r#"goSettings("invite_accept", "invite_text")"#, "#1k 레일 + · #1r 수락 칸은 라우터가"),
    ("app.js", r#"mdHead("agents", agentTile(a.color), a.name,"#, "#1k 줄 타일 · #1z-7 로봇 타일(고른 색) · #1z-8 상세 머리"),
    ("view.js", "agent: conv.dir === \"in\" ? `내 ${conv.agent}` : conv.agent", "#1k 받은 대화만 '내 '"),
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
    ("app.js", "if (!newEngine) return; // 1단계에서 도구를 골라야 폼이 보인다", "#1z-9 도구 없는 등록 없음"),
    ("app.js", "등록 뒤 바꿀 수 없음 — 바꾸려면 새로 등록", "#1z-9 상세 실행 도구"),
    ("view.js", "`${engineLabel(engine)} 에이전트는 쓰지 않음`", "#1z-9 안 쓰는 블록 문구"),
    ("app.css", ".engine-block:disabled { border-style: dashed;", "#1z-9 안 쓰는 블록 점선"),
    ("app.css", r#"#settings_tree a[aria-current="page"] {"#, "#1z-8 고른 메뉴 표시"),
    ("app.css", ".rows li.pm-agent { background: var(--agent-soft); }", "#1z-2 에이전트 줄 바탕"),
    ("app.js", r#"model.value = draft?.model ?? p.model ?? "";"#, "#1z-2 R1 펼친 다른 사용자의 저장 전 값 보존"),
    ("app.js", "if (gsDrafts.get(p.id) === sent) gsDrafts.delete(p.id);", "#1z-2 R2 저장 응답 대기 중 고친 값 보존"),
    ("index.html", r#"title="초대 수락"><span class="ico ico-user-plus lead" aria-hidden="true"></span>사용자</button>"#, "#1z-2 레일 + 사용자 · #1z-10 + 글자 → 아이콘"),
    ("index.html", r#"title="새 그룹"><span class="ico ico-folder-plus lead" aria-hidden="true"></span>그룹</button>"#, "#1z-2 레일 + 그룹 · #1z-10 + 글자 → 아이콘"),
    ("index.html", "에이전트 응답</strong>", "#1z-2 에이전트 응답 화면 제목"),
    ("index.html", r#"<button id="home_view_preview" type="button" aria-pressed="true"><span class="ico ico-grid lead" aria-hidden="true"></span>미리보기</button>"#, "#1z-4 개요 보기 전환"),
    ("index.html", r#"<div id="home_preview" class="pv-list"></div>"#, "#1z-5 개요 미리보기 = 상대별 절"),
    ("app.css", ".pv-cards { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(100%, calc(256px * var(--pv-scale))), 1fr));", "#1z-5 카드 열 수 = 폭 · 미리보기 크기만큼 카드 폭(사용자 지시) · #1z-11 시안 C 256px"),
    ("app.js", "const secs = previewSections(groups, single, state.previewCols);", "#1z-5 '나' · 그룹은 절마다 한 줄"),
    ("app.js", "new ResizeObserver(refitPreview).observe($(\"home_preview\"));", "#1z-5 창 폭이 바뀌면 열 수를 다시 잰다"),
    ("app.js", "  refitPreview();", "미리보기 크기로 카드 폭이 바뀌면 열 수를 다시 잰다(사용자 지시)"),
    ("app.css", ".pv-msg { flex: none; max-width: 88%; padding: var(--pv-msg-pad); border-radius: var(--radius-lg); font-size: var(--fs-xs); line-height: var(--pv-lh-msg);", "#1z-11 결정 3 말풍선 = 시안 C 12px · 줄간격 1.5(사용자 지시 축소 11px 개정)"),
    ("app.css", ".pv-msg b { margin-right: var(--pv-gap-who); color: var(--pv-muted); font-size: var(--pv-fs-who); font-weight: var(--fw-medium); }", "#1z-11 결정 3 말풍선 이름 = 시안 C 10.5px"),
    ("app.js", "text: `+${s.more}개 더 보기` })", "#1z-5 넘치면 \"+n개\" · #1z-11 시안 C 글"),
    ("app.css", ".pv-head strong { min-width: 0; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--pv-text); font-size: var(--fs-md); font-weight: var(--fw-medium); }", "#1z-11 결정 4 상대 이름 14px · 500"),
    ("app.css", ".pv-sec + .pv-sec { border-top: 1px solid var(--pv-line); }", "#1z-11 시안 C 절 사이 가는 선"),
    ("app.css", ".pv-list { flex: 1 1 auto; min-height: 0; overflow-y: auto; display: flex; flex-direction: column; padding: var(--pv-list-pad); background: var(--pv-bg);", "#1z-11 시안 C 종이 바탕"),
    ("app.css", "  --pv-bg: var(--dark-pv-bg);", "#1z-11 다크 = 어두운 종이 톤"),
    ("app.css", ".pv-msg.side-theirs { align-self: flex-start; background: var(--pv-theirs); border: 1px solid var(--pv-theirs-line); }", "#1z-11 결정 3 상대 글 = 흰 바탕 + 가는 테두리"),
    ("app.css", ".pv-msg.agent, .pv-msg.running { background: var(--pv-agent-bg); border: 0; color: var(--pv-agent); }", "#1z-11 에이전트 · 실행 중 말풍선"),
    ("app.js", r#"c.unread ? el("span", { class: "pv-new", text: `새 글 ${c.unread}` }) : "","#, "#1z-11 결정 6 안 읽음 = 머리 \"새 글 n\""),
    ("app.js", r#"c.last_state ? el("span", { class: `pv-state st-${c.last_state}`, text: stateText(c.last_state) }) : "","#, "#1z-11 상태 = 바닥 점 + 글자"),
    ("app.js", r#"el("span", { class: "pv-pres" }, el("span", { class: `st-dot st-${pres.key}`, "aria-hidden": "true" }), el("span", { text: p ? pres.text : "페어링 해제" })),"#, "#1z-11 머리 접속 = 점 + 글"),
    ("app.js", "    el(\"div\", { class: \"pv-head\" }, ...head),", "개요 미리보기 절 머리에 \"{이름} 개요 →\" 없음(사용자 지시)"),
    ("app.css", ".side-list li[data-conv] > .row-more { position: static; flex: none; align-self: center;", "대화 줄 ⋯ 이 시각을 가리지 않는다(사용자 지시)"),
    ("app.js", "if (state.railPeer == null && state.railGroup == null) pickRail(c.peer_id);", "'나' 개요에서 연 대화 → 레일 · 사이드바도 그 상대로(사용자 지시)"),
    ("app.js", "el(\"h2\", { class: \"side-title\" }, el(\"span\", { text: s.title })),", "에이전트 응답 사용자 머리에 대화 수 없음(사용자 지시)"),
    ("index.html", r#"<label id="home_zoom_pick" class="head-pick"><span class="ico ico-resize lead" aria-hidden="true"></span>미리보기 크기 <select id="ui_home_zoom" data-num><option value="100">100%</option><option value="125">125%</option><option value="150">150%</option><option value="175">175%</option><option value="200">200%</option></select></label>"#, "#1z-6 미리보기 크기 100 ~ 200%"),
    ("app.css", ".pv-body { height: calc(132px * var(--pv-scale));", "#1z-6 본문 높이 배율 · #1z-11 시안 C 132px"),
    ("app.css", ".pv-text { display: -webkit-box; -webkit-line-clamp: 2;", "#1z-6 말풍선 줄 수 → #1z-8 모든 크기 2줄 · 안쪽 칸에서 자름"),
    ("app.js", "$(\"home_zoom_pick\").hidden = !pv;", "#1z-6 미리보기에서만"),
    ("app.js", "    applyHomeZoom(s.ui_home_zoom); // 미리보기 크기(#1z-6)도 시작할 때만", "#1z-6 R1 배율은 시작할 때 · 고를 때만(늦은 설정 응답이 되돌리지 않게)"),
    ("app.js", "  homeZoomSaver.push(Number(e.target.value));", "#1z-6 R2 배율 저장은 한 번에 하나 · 마지막 값"),
    ("app.js", "if (all) for (const id of state.previewMsgs.keys()) if (!convs.some((c) => c.id === id)) state.previewStale.add(id);", "#1z-5 R1 전체 무효화는 숨은 카드에도"),
    ("app.js", "api(\"GET\", `/api/conversations/${c.id}/messages?limit=${PREVIEW_MSGS}`)", "#1z-4 미리보기는 마지막 글만"),
    ("app.js", "api(\"PUT\", \"/api/settings\", { ui_home_view: v })", "#1z-4 보기 기억"),
    ("app.js", "$(\"home_intro\").hidden = groups.length > 0 || group != null; // 두 보기 공통(#1z-4 R2)", "#1z-4 R2 첫 연결 안내는 보기 분기 전"),
    ("app.js", "// 끊긴 동안의 변경 신호는 다시 오지 않는다 — 붙을 때마다 미리보기 글을 전부 새로(#1z-4 R2).", "#1z-4 R2 변경 신호 다시 붙으면 미리보기 전부 새로"),
    ("app.css", "#rail { user-select: none; }", "#1z-2 레일 끌기 글자 선택 없음"),
    ("app.js", "railSlot(same.map(mid), across ? e.clientX : e.clientY)", "#1z-2 레일 놓을 자리 = 가장 가까운 타일"),
    ("app.js", r#"el("span", { class: "home-agent" }, speakerEl("agent"),"#, "#1z-2 개요 줄 에이전트 아이콘"),
    ("app.js", "rowTile(`small ${colorOf(g.peer)}`, p?.icon),", "#1z-2 개요 머리 사용자 아이콘"),
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
    ("app.js", "function ico(name)", "#1l 아이콘"),
    ("app.js", r#"el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label }))"#, "#1l 아이콘"),
    ("app.js", r#""data-tone": colorOf(conv.peer_id)"#, "#1l 말풍선 색"),
    ("app.js", r#"$("me_dialog").showModal();"#, "#1l 사용자 정보"),
    ("app.js", "function showAgentNew(on)", "#1l 에이전트 등록"),
    ("app.js", r#"if (r.sub === "agent_new") showAgentNew(true);"#, "#1l 에이전트 등록 · #1z-8 별칭 agent_new"),
    ("app.js", "function applyRailOrder(order)", "#1l 타일 순서"),
    ("app.js", r#""ui_rail_order","#, "#1l 타일 순서"),
    ("app.js", r#"item(dir < 0 ? "arrow-up" : "arrow-down", text"#, "#1l 아이콘"),
    ("view.js", r#"{ key: "me", label: "사용자 정보", icon: "user" }"#, "#1l 사용자 정보 · #1r 아이콘"),
    ("app.css", ".ico { display: inline-block; flex: none; width: 16px; height: 16px;", "#1l 아이콘"),
    ("app.css", r#".msg[data-tone="peer-1"] { --tone: var(--peer-1); }"#, "#1l 말풍선 색"),
    ("app.css", ".msg.side-theirs .bubble::before { content: \"\"; position: absolute; inset: 0; z-index: -1; border-radius: inherit; background: var(--tone, var(--wait)); opacity: var(--bubble-theirs-alpha);", "#1l 말풍선 색 · #1p 층 토큰"),
    ("app.css", ".msg.side-mine .bubble { isolation: isolate; background: var(--surface); color: var(--text);", "#1l 말풍선 색"),
    ("app.css", ".help-wrap:hover .tip, .help:focus + .tip { display: block; }", "#1l 도움말"),
    // 1m — 컨텍스트
    ("index.html", r#"<button id="ctx_btn" type="button" class="tool-btn ctx-btn" aria-haspopup="menu""#, "#1m 컨텍스트"),
    ("index.html", r#"<span class="ico ico-gauge" aria-hidden="true"></span><span id="ctx_text">컨텍스트 알 수 없음</span>"#, "#1m 컨텍스트"),
    ("index.html", r#"<label for="auto_compact"><span class="ico ico-compress lead" aria-hidden="true"></span>자동 압축</label>"#, "#1m 자동 압축"),
    ("index.html", r#"<span class="hint">컨텍스트가 80% 를 넘으면 대화를 요약해 새 세션으로 이어 갑니다(이 PC 의 Claude 사용량을 씁니다)</span></span><input type="checkbox" id="auto_compact" role="switch">"#, "#1m 자동 압축 · #1z-8 타일 설명 · 스위치"),
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
    ("app.js", "convListParts(s.rows, group)", "#1o 목록 나눔 · #1v 내 대화만"),
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
    ("app.js", r#"text: SESSION_NEW"#, "#1o 에이전트 새 대화 · #1x 세션 생성"),
    // 1n — 부모 도구 메뉴 사용법
    ("usage", "coworkAgent start [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent open [--parent <PID>]", "#1n 사용법"),
    ("usage", "coworkAgent stop\n", "#1n 사용법"),
    ("usage", "종료 코드 8 = 시작 실패", "#1n 사용법"),
    // 1q — 요청 개요 · 글자 단추
    ("app.js", "function reqBoxEl(", "#1q 요청 개요"),
    ("app.js", r#"foldEl(`requests:${box}`, head, body, "peer-fold req-box", null, box === "mine")"#, "#1q 보낸 요청 처음 접힘"),
    ("app.js", "state.reqBox = state.reqBox === b && !reqBoard() ? null : b;", "#1q 탭 토글 · #1v 보드엔 개요 없음"),
    ("app.js", r#"onclick: target === "requests" && current ? () => loadRequests(true) : null"#, "#1q 메뉴 다시 누름 = 개요"),
    ("app.js", "bar.append(textButton(mv.label, REQ_BTN[mv.to]", "#1q 글자 단추"),
    ("view.js", r#"rejected: { tone: "no", icon: "x" }"#, "#1q 반려 ✕"),
    ("view.js", r#"accepted: { tone: "ok", icon: "check" }"#, "#1q 수용 ✓"),
    ("app.css", "button.text-btn.ok { color: var(--ok); }", "#1q 글자 단추"),
    ("app.css", "button.text-btn.no { color: var(--danger); }", "#1q 글자 단추"),
    ("app.css", ".ico-check {", "#1q 아이콘"),
    ("app.css", ".ico-x {", "#1q 아이콘"),
    ("index.html", r#"<nav id="act_tabs" class="tabs" aria-label="내 활동 종류"></nav>"#, "#1q 내 활동 탭"),
    ("view.js", r#"{ kind: "request", label: "받은 요청", icon: "arrow-down" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view.js", r#"{ kind: "mine", label: "내 요청", icon: "arrow-up" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view.js", r#"{ kind: "answer", label: "받은 답", icon: "chat" },"#, "#1q 내 활동 탭 · #1r 아이콘"),
    ("view.js", "`내 요청 · ${state}`", "#1q 내 요청 행"),
    ("app.js", r#"api("GET", "/api/messages/peers")"#, "#1q 주고받은 메시지 상대별 요약"),
    ("app.js", r#"foldEl(`peer:${h.peer}`, head, body, "peer-fold peer-msgs", null, true)"#, "#1q 상대 절 처음 접힘"),
    ("app.js", "`/api/messages?side=both&peer=${peer}${q}`", "#1q 서버 페이지"),
    ("app.js", r#"button("더 보기", more, "small")"#, "#1q 더 보기"),
    ("app.js", r#"b.classList.add("bookmark", "row-mark");"#, "#1q 행 북마크"),
    ("view.js", "export const PEER_PAGE = 20;", "#1q 20건씩"),
    ("app.css", ".rows li.has-mark {", "#1q 행 북마크 자리"),
    // 1r — 내 응답 에이전트 · 들여쓰기 · ? · 초대 · 새 에이전트 줄 · 연필 · 아이콘
    ("index.html", r#"<span class="ico ico-swap" aria-hidden="true"></span>내 응답 에이전트</label>"#, "#1r 새 대화"),
    ("index.html", r#"<label for="chat_relay_cap" id="chat_relay_cap_label" class="sub-label" hidden>"#, "#1r 최대 응답 횟수 하위 줄"),
    ("index.html", "최대 응답 횟수</label>", "#1r 최대 응답 횟수"),
    ("app.js", r#"$("chat_relay_cap").hidden = $("chat_relay_cap_label").hidden = $("chat_relay_cap").disabled = off;"#, "#1r 고를 때만 · R1 숨기면 검사 제외"),
    ("app.js", "if (inviteBusy) return;", "#1r R1 초대 생성 한 번에 하나"),
    ("app.js", r#"$("window_btn").setAttribute("aria-disabled", "true");"#, "#1r R2 초점 잃지 않게 aria-disabled"),
    ("app.js", r#"$(pick.agents != null ? mdRowId("agents", pick.agents) : "agent_new_btn")?.focus(); // 숨긴 폼에 초점이 남지 않게"#, "#1r R1 등록 뒤 초점 · #1z-8 등록한 줄"),
    ("app.css", ".peer-fold > :not(summary) { margin-left: var(--sp-5); }", "#1r 접이식 들여쓰기 · #1s 세로선 없음"),
    ("index.html", r#"aria-describedby="tip_relay""#, "#1r 자동 왕복 ?"),
    ("index.html", "• 시작과 끝 — 왕복이 시작할 때와 끝날 때(상한 도달 · 실패 · 상대가 멈춤 등)만 알림", "#1r 자동 왕복 ?"),
    ("index.html", "• 메시지마다 — 시작 · 끝에 더해 상대 에이전트의 답이 올 때마다 알림", "#1r 자동 왕복 ?"),
    ("index.html", "• 알리지 않음 — 자동 왕복 알림 없음", "#1r 자동 왕복 ?"),
    ("view.js", r#"press: "window_btn""#, "#1r 초대 생성 누름"),
    ("app.js", "if (i.press) $(i.press).click();", "#1r 새로 만들기 초대 생성"),
    ("app.js", r#"showAccept(r.sub === "invite_accept");"#, "#1r 수락 칸 접기(다른 진입이면 닫음) · #1z-8"),
    ("index.html", r#"<strong id="agent_new_h">새 에이전트</strong>"#, "#1r 새 에이전트 · #1z-8 상세 칸"),
    ("index.html", r#"<button type="button" id="agent_new_cancel"><span class="ico ico-x lead" aria-hidden="true"></span>취소</button>"#, "#1r 새 에이전트 줄"),
    ("app.css", ".ico-gear {", "#1r 새 아이콘"),
    ("app.css", ".ico-title {", "#1r 새 아이콘"),
    ("index.html", r#"<span class="ico ico-arrow-down" aria-hidden="true"></span>받은 요청</a>"#, "#1r 요청 탭 아이콘"),
    ("index.html", r#"<span class="ico ico-arrow-up" aria-hidden="true"></span>보낸 요청</a>"#, "#1r 요청 탭 아이콘"),
    ("app.js", r#"}, ico(k.icon), el("span", { text: k.label })),"#, "#1r 내 활동 탭 아이콘"),
    // 1z-7 — 대화 유형 아이콘 · 색(시안 A), 내 에이전트 상세(한 격자 · 아이콘 색 · 닫기)
    ("index.html", r#"<span id="conv_kind" class="conv-kind kind-chip" role="img" hidden></span><strong id="conv_title"></strong>"#, "#1z-7 제목 앞 유형 아이콘"),
    ("view.js", r#"return conv.dir === "in" ? "bot" : conv.dir === "out" ? "user" : null;"#, "#1z-7 응답 = 로봇 · 문의 = 사람 · 공유 = 없음"),
    ("view.js", r#"export const KIND_NAMES = { in: "내 에이전트 응답", out: "내가 문의" };"#, "#1z-7 유형 이름"),
    ("app.css", "  --conv-in: var(--agent);", "#1z-7 응답 = 청록"),
    ("app.css", "  --conv-out: var(--accent);", "#1z-7 문의 = 파랑"),
    ("app.css", "  --conv-shared: var(--faint);", "#1z-7 공유 = 회색"),
    ("app.css", ".conv-head[data-kind] { border-bottom: 2px solid var(--kind); }", "#1z-7 머리 아래 선"),
    ("app.css", ".side-list li[data-kind] { border-color: var(--kind); }", "#1z-7 사이드바 줄 테두리"),
    ("app.css", ".pv-card.unseen { border-color: var(--pv-unseen); }", "#1z-7 안 읽음 표시 → #1z-11 결정 6 테두리 색(윤곽선 대신)"),
    ("app.css", ".pv-card { display: flex; flex-direction: column; min-width: 0; overflow: hidden; background: var(--pv-card); border: 1px solid var(--pv-line);", "#1z-7 카드 테두리 = 유형 색 → #1z-11 결정 6 한 색"),
    ("app.css", r#".pv-card[data-kind="in"] .pv-kind { color: var(--pv-kind-in); }"#, "#1z-11 결정 6 유형 = 아이콘 색"),
    ("app.js", r#"{ class: c.unread ? "pv-card unseen" : "pv-card", href: `#c/${c.id}`, "aria-label": `${[KIND_NAMES[c.dir], title].filter(Boolean).join(" · ")} 대화 열기`, onclick: () => followFromHome(c), "data-kind": c.dir },"#, "#1z-7 미리보기 카드 유형 · R1 읽는 이름에 유형"),
    ("app.js", r#"el("span", { class: `${cls} kind-chip`, role: "img", "aria-label": KIND_NAMES[c.dir] }, ico(n))"#, "#1z-7 R1 유형 칩 읽는 이름"),
    ("app.css", r#".conv-head[data-kind="shared"] { border-bottom-style: dashed; }"#, "#1z-7 R1 공유 머리는 아래 선만"),
    ("app.js", r#"{ class: meta.unread ? "unseen" : null, "data-kind": c.dir },"#, "#1z-7 개요 목록 줄 유형"),
    ("app.js", r#""data-section": section, "data-kind": c.dir },"#, "#1z-7 사이드바 줄 유형"),
    ("app.js", r#"el("span", { class: "label" }, lead("palette"), "아이콘 색"),"#, "#1z-7 아이콘 색 줄"),
    ("app.js", r#"el("span", { class: "label" }, lead("title"), "이름"),"#, "#1z-7 이름도 같은 격자"),
    ("app.js", r#"button("닫기", () => closePick(page), undefined, "x")"#, "#1z-7 닫기 · #1z-8 상세 닫기(고름 풀기)"),
    ("app.js", r#""aria-label": "상세 닫기", onclick: () => closePick(page) }, ico("x")),"#, "#1z-8 상세 머리 ✕"),
    ("app.js", r#"$(opener ?? mdRowId(page, was))?.focus();"#, "#1z-8 닫기 뒤 초점 = 연 곳"),
    ("app.js", r#"a.setAttribute("aria-current", a.dataset.page === r.page ? "page" : "false");"#, "#1z-8 고른 메뉴 aria-current"),
    ("app.js", r#"for (const n of settingsTree()) $(n.id).hidden = n.id !== r.page;"#, "#1z-8 고른 화면만"),
    ("view.js", r#"return { view: "settings", page: settingsTree().some((n) => n.id === m[1]) ? m[1] : "conn", sub: null };"#, "#1z-8 빈 · 모르는 = 연결"),
    ("app.js", r#"const agentDrafts = new Map();"#, "#1z-8 에이전트 저장 전 값"),
    ("app.js", r#"if (agentDrafts.get(a.id) === sent) agentDrafts.delete(a.id);"#, "#1z-8 저장 성공이면 초안 지움"),
    ("app.js", r#"api("GET", "/api/debug"), api("GET", "/api/debug/log")"#, "#1z-8 디버깅 API"),
    ("app.js", r#"debugCopy(diagText(debugInfo, state.me), "진단 정보 복사")"#, "#1z-8 진단 정보 복사"),
    ("index.html", r#"<pre id="debug_log" class="debug-log-box" tabindex="0" aria-label="로그 끝 200줄"></pre>"#, "#1z-8 로그 끝 200줄"),
    ("index.html", r#"<input type="checkbox" id="notify_inbound" role="switch">"#, "#1z-8 스위치"),
    ("app.js", r#"const parent = el("input", { type: "checkbox", role: "switch", title: PARENT_NOTE });"#, "#1z-8 상세 스위치"),
    ("app.js", "agentTile(a.color, \"small\"),", "#1z-7 등록된 에이전트 줄 로봇 타일"),
    ("view.js", r#"export const AGENT_COLORS = ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"];"#, "#1z-7 아이콘 색 6개"),
    // 1z-8 — 상단 바 버전 · 검색 스위치 · 유형 칩(S1) · 미리보기 2줄 · 세부정보 제목 줄 없음
    ("index.html", r#"<strong class="wordmark">coworkAgent<span id="top_version" class="wordmark-ver"></span></strong>"#, "#1z-8 상단 바 버전"),
    ("app.js", r#"$("top_version").textContent = `v${s.version}`;"#, "#1z-8 상단 바 버전"),
    ("index.html", r#"<label class="switch-line"><input type="checkbox" id="search_closed" role="switch">나간 대화 포함</label>"#, "#1z-8 검색 스위치"),
    ("index.html", "지운 대화는 검색되지 않습니다", "#1z-8 검색 안내"),
    ("app.js", "&closed=${$(\"search_closed\").checked}", "#1z-8 검색 스위치 → 서버"),
    ("app.js", r#"searchMark(row.mark),"#, "#1z-8 검색 줄 칩"),
    ("app.css", r#"input[type="checkbox"][role="switch"] { appearance: none;"#, "#1z-8 스위치 모양"),
    ("app.js", r#"el("span", { class: "pv-text" },"#, "#1z-8 말풍선 글자 칸"),
    ("app.css", "justify-content: flex-end; gap: var(--sp-3);", "#1z-8 말풍선 사이 여백"),
    ("index.html", r#"<h3><span class="ico ico-info lead" aria-hidden="true"></span>대화 정보</h3>"#, "#1z-10 대화 정보 절 아이콘"),
    ("app.js", r#"api("GET", `/api/conversations/${conv.id}/info`)"#, "#1z-10 대화 정보 = /info"),
    ("app.js", r#"lead("copy"), "복사")"#, "#1z-10 실행 명령 복사 단추"),
    ("app.js", "navigator.clipboard.writeText(text);", "#1z-10 복사 = 전체 명령"),
    ("app.js", r#"infoView.key = ""; // 실패한 읽기는 다음 그리기"#, "#1z-10 R1 /info 실패 뒤 다시 읽기"),
    ("app.js", "if (infoView.gen !== gen) return;", "#1z-10 R2 /info 마지막 읽기 응답만(A → B → A 역전 막음)"),
    ("view.js", r#"{ icon: "terminal", label: "실행 명령","#, "#1z-10 결정 2 라벨"),
    ("app.js", "if (!h.querySelector(\".ico\")) h.prepend(lead(n.icon));", "#1z-10 설정 화면 제목 = 메뉴 아이콘"),
    ("app.css", ".set-head h2 .ico.lead { color: var(--accent); }", "#1z-10 설정 화면 제목 아이콘만 강조색"),
    ("app.css", ".ico.lead { margin-right: var(--sp-2);", "#1z-10 글자 앞 아이콘"),
    ("app.css", ".md-detail:has(> .md-pane:not([hidden]), > .md-head) { align-self: start; max-height: 100%; }", "#1z-10 결정 8 상세 상자 = 내용 높이"),
    ("app.css", ".md-body { flex: 0 1 auto; min-height: 0;", "#1z-10 결정 8 단추 줄 = 내용 바로 아래"),
    ("app.js", r#""primary",
        "save","#, "#1z-10 에이전트 저장 아이콘"),
    ("app.js", r#""danger",
        "trash","#, "#1z-10 에이전트 삭제 아이콘"),
    ("app.js", r#"button("이름 바꾸기", (ev) => openGroupRename(g.id, ev.currentTarget), undefined, "pencil"),"#, "#1z-10 그룹 이름 바꾸기 아이콘"),
    ("index.html", r#"<button type="submit" class="primary"><span class="ico ico-send lead" aria-hidden="true"></span>보내기</button>"#, "#1z-10 보내기 아이콘"),
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
    ("web", "@ 로 상대 에이전트 부르기", "#1v 입력창 안내 삭제"),
    ("app.js", " 에게 보내기`", "#1v 입력창 placeholder = \"메시지 입력\" 고정(적대적 평가 R2)"),
    ("app.js", r#"$("reply_text").placeholder"#, "#1v placeholder 다시 넣지 않음(적대적 평가 R2)"),
    ("index.html", r#"id="edit_btn""#, "#1v 웹UI 편집 모드 삭제"),
    ("index.html", r#"id="edit_bar""#, "#1v 웹UI 편집 모드 삭제"),
    ("index.html", r#"id="scope_dialog""#, "#1v 웹UI 편집 모드 삭제"),
    ("app.js", "function setEditMode(", "#1v 웹UI 편집 모드 삭제"),
    ("app.js", r#""본문 편집""#, "#1v 웹UI 편집 모드 삭제"),
    ("app.js", "`메모: ${m.req_note}`", "#1v 메모는 피드백 말풍선으로"),
    ("app.js", "`메모: ${r.note}`", "#1z 메모 → 의견"),
    ("app.js", "요청 피드백 · ", "#1z 피드백 → 의견"),
    ("app.js", "피드백(선택)", "#1z 피드백 → 의견"),
    ("app.js", r#"text: "피드백""#, "#1z 피드백 → 의견"),
    ("index.html", r#"<option value="normal">보통</option>"#, "#1z 타일 크기 보통 → 크게"),
    ("app.css", "min-height: 76px", "#1z 입력창 글 칸 44px"),
    ("web", "연결 목록", "#1z 연결 목록 → 그룹설정"),
    ("app.js", r#"$(box).addEventListener("dragstart""#, "#1z 레일 HTML 끌기 없음"),
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
    // 1q — 옛 자리
    ("app.js", r#"const bar = el("div", { class: "actions" });"#, "#1q 요청 단추 우하단 자리 없음"),
    ("app.js", r#"reqBox: "theirs""#, "#1q 처음 화면 = 개요"),
    ("index.html", r#"<select id="act_kind">"#, "#1q 내 활동 거르기 = 탭"),
    ("index.html+view.js", "자동 왕복 시작", "#1q 내 활동에 자동 왕복 없음"),
    ("index.html+view.js", "사람 메시지</option>", "#1q 내 활동 옛 거르기"),
    // 1r — 옛 자리
    ("index.html", r#"<label for="chat_relay_agent">자동 왕복</label>"#, "#1r 새 대화에 자동 왕복 없음"),
    ("index.html", "자동 왕복 상한</label>", "#1r 새 대화에 자동 왕복 없음"),
    ("app.js", "요청으로는 자동 왕복을 열 수 없습니다", "#1r 새 대화에 자동 왕복 없음"),
    ("app.js", r#"text: "관리","#, "#1r 관리 = 연필"),
    ("app.css", ".req-box .peer-fold > summary { padding-left", "#1r 들여쓰기는 공통 규칙"),
    ("app.js", "showAccept(true);", "#1r 수락 칸은 라우터만 펼친다"),
    ("app.js", r#"$("window_btn").disabled = true;"#, "#1r R2 초대 생성 중에도 초점을 받는다"),
    // 1s — 접이식 하위 줄 세로선 없음(들여쓰기만)
    ("app.css", ".peer-fold > :not(summary) { margin-left: var(--sp-5); border-left", "#1s 세로선 제거"),
    // 1w — 상대 단위 그룹 소속 없음(D37 개정)
    ("app.js", "/api/groups/${id}/peers", "#1w 상대 소속 API 없음"),
    ("app.js", "/api/peers/${d.peer}/groups", "#1w 상대 소속 API 없음"),
    ("app.js", "peerGroupMenu", "#1w 레일 상대 타일 '그룹' 메뉴 없음"),
    ("app.js", r#"text: "그룹 ▾""#, "#1w 설정 '그룹 ▾' 없음"),
    ("view.js", "그룹에 없는 연결", "#1w '사용자 목록' 으로"),
    ("view.js", "NO_PEER_GROUP", "#1w '사용자 목록' 으로"),
    // 1x — 세션 생성 · 기록 삭제 · 나가기 · 사이드바 머리
    ("index.html+app.js+view.js", "새 대화\"", "#1x 새 대화 → 세션 생성"),
    ("index.html", "실행하지 않고 상대의 요청 목록에 올립니다", "#1x 요청 안내 지움"),
    ("index.html", r#"id="chat_text""#, "#1x 세션 생성에 메시지 없음"),
    ("index.html", r#"id="chat_ask_label""#, "#1x 세션 생성에 메시지 종류 없음"),
    ("app.js", "ask_hint", "#1x 요청 안내 지움"),
    ("view.js", r#"label: "삭제""#, "#1x 대화 메뉴 삭제 → 기록 삭제 · 나가기"),
    ("index.html+app.js+view.js", "이력유지", "#1x 나가기 하위 메뉴 없음"),
    ("index.html+app.js+view.js", "이력삭제", "#1x 기록 삭제로 하나"),
    ("app.js", r#""지문 " + state.me.fingerprint"#, "#1x 사이드바 머리 지문 없음"),
    ("app.js", "starButton(c, label.title),", "#1x 대화 줄 ☆ 없음"),
    ("app.js", r#"draggable: "true""#, "#1z-2 HTML 끌어 놓기 없음(설정 대화 줄도 포인터)"),
    ("app.js", r#"class: "room-preview""#, "#1z-2 대화 줄 2행(미리보기) 없음"),
    ("view.js", r#"label: "에이전트 대화""#, "#1z-2 메뉴 이름 에이전트 응답"),
    ("index.html", r#"aria-label="초대 수락">+</button>"#, "#1z-2 레일 + 글자"),
    ("app.js", "groupSettingsListEl(", "#1z-2 시안 A 목록 없음"),
    ("app.js", r#"kvTable([["이름", a.name]])"#, "#1z-7 이름 표 → 격자"),
    ("index.html", "이 PC 에만 저장", "#1z-8 세부정보 대화 정보 제목 줄 삭제"),
    ("app.css", "--pv-lines", "#1z-8 말풍선 줄 수 배율 없음"),
    ("app.js", "function manageItem(", "#1z-8 연필 펼침 → 목록 · 상세"),
    ("app.js", "gsGroupsEl", "#1z-8 그룹 카드 격자 없음"),
    ("app.js", "gsUsersEl", "#1z-8 사용자 표 → 연결 목록"),
    ("app.js", "{ join: true }", "#1z-8 설정 화면 대화 끌기 없음"),
    ("app.js", "gsJoin", "#1z-8 설정 화면 대화 끌기 없음"),
    ("app.js+view.js", "convCopy", "#1z-8 끌어 놓기 복사 함수 없음(넣기 단추)"),
    ("view.js", "USER_LIST", "#1z-8 사용자 목록 절 없음"),
    ("view.js", "children: [", "#1z-8 설정 메뉴 하위 없음"),
    ("index.html", r#"class="card"#, "#1z-8 설정 카드 한 열 없음"),
    ("app.css", "max-width: 1000px", "#1z-8 설정 본문 폭 제한 없음"),
    ("app.js", "scrollIntoView({ block: \"start\" });\n  applyFocus", "#1z-8 설정 절 스크롤 없음"),
    ("index.html+app.js", "info_defaults", "#1z-10 결정 1 실행 기본값 줄 없음"),
    ("index.html+app.js", "info_peer", "#1z-10 결정 1 상대 줄 없음"),
    ("index.html", ">+ 사용자<", "#1z-10 + 글자 → 아이콘"),
    ("app.css", "--pv-zoom", "#1z-11 결정 5 절 15% 확대 없음"),
    ("app.js", "pv-unread", "#1z-11 결정 6 카드 바닥 안 읽음 배지 없음"),
    ("app.js", "rowTile(`small ${colorOf(s.peer)}`, p?.icon),", "#1z-11 미리보기 머리 상대 타일 없음"),
    ("app.css", ".pv-card.unseen { outline", "#1z-11 결정 6 안 읽음 윤곽선 없음"),
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
    for (file, text) in sources("index.html+app.js+view.js") {
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
