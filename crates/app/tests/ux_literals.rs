//! 화면 문구 · favicon 링크 literal(plan 해석 1) — 웹UI 원본(빌드 없는 정적 파일)을 그대로 읽는다.
use an_net::invite::Invite;

const INDEX: &str = include_str!("../web/index.html");
const APP: &str = include_str!("../web/app.js");
const VIEW: &str = include_str!("../web/view.js");
const TRAY: &str = include_str!("../src/tray.rs");
const CSS: &str = include_str!("../web/app.css");

/// 알림 절의 자동 왕복 상한 라벨 = 원문 "대화 턴에 대한 상한 기본값".
#[test]
fn turn_label_literal() {
    assert!(INDEX.contains(r#"<label for="relay_cap_default">대화 턴에 대한 상한 기본값</label>"#));
    assert!(!INDEX.contains("새 자동 왕복 대화의 상한 기본값"));
}

/// "페어링 받기" → "초대 생성", "초대 붙여넣기" → "초대 수락", 수락 안내는 '초대 생성' 기준, "메신저" 없음 — 화면 · 초대 텍스트 모두.
#[test]
fn invite_labels_without_messenger() {
    for label in [
        r#"id="window_btn" class="primary big">"#,
        r#"<label for="invite_text">초대 수락</label>"#,
        "'초대 생성' 에서 만든 초대 메시지를 통째로 붙여 넣으세요",
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for (file, text) in [("index.html", INDEX), ("app.js", APP)] {
        for gone in ["페어링 받기", "초대 붙여넣기", "메신저"] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
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

/// 탭 아이콘 = 실행 파일 · 트레이와 같은 아이콘(`GET /favicon.ico`).
#[test]
fn favicon_link_present() {
    assert!(INDEX.contains(r#"<link rel="icon" href="/favicon.ico">"#));
}

/// "대화 검색" → "제목 검색"(사이드바 대화 목록은 제목으로만 거른다).
#[test]
fn title_search_label() {
    assert!(INDEX.contains(r#"<label for="conv_search">제목 검색</label>"#));
    assert!(!INDEX.contains("대화 검색"));
}

/// 트레이 메뉴는 "열기" · "종료" 만 — "내 정보 보기" 제거(오류 모드의 "오류 보기" 는 그대로).
#[test]
fn tray_menu_open_quit_only() {
    for item in [r#"MenuItem::new("열기""#, r#"MenuItem::new("종료""#] {
        assert!(TRAY.contains(item), "{item}");
    }
    assert!(!TRAY.contains("내 정보 보기"));
    assert_eq!(
        TRAY.matches("MenuItem::new(").count(),
        4,
        "정상 모드 열기 · 종료 + 오류 모드 오류 보기 · 종료"
    );
}

/// 시안 v4 의 영역별 라벨(plan 설계 "문구 literal" 목록 전부)이 화면 원본(index.html · app.js · view.js)에 글자 그대로 있다. 원문 보기 버튼은 하나.
#[test]
fn workspace_labels() {
    let all = [INDEX, APP, VIEW].concat();
    for label in [
        "검색 — 메시지 · 대화 · 사람 · 에이전트",
        "+ 새로 만들기",
        "새 대화",
        "초대 생성",
        "초대 수락",
        "에이전트 등록",
        "표시 이름 · 아이콘",
        "알림 일시 중지",
        "환경설정",
        "내 활동",
        "북마크 항목",
        "에이전트",
        "주고받은 글",
        "제목 검색",
        "즐겨찾기",
        "제목 편집",
        "대화 보기",
        "원문 보기",
        "세부정보",
        "북마크",
        "복사",
        "굵게",
        "목록",
        "코드",
        "Enter 보내기 · Shift+Enter 줄바꿈",
        "보내기",
        " 에게 보내기 — markdown 가능",
        "실행 상세",
        "닫기",
        "진행",
        "요청 보냄",
        "대기열",
        "실행 완료",
        "응답 받음",
        "대화 정보",
        "상대 에이전트",
        "실행 기본값",
        "이 PC 에만 저장",
        "자동 왕복",
        "멈춤",
    ] {
        assert!(all.contains(label), "{label}");
    }
    assert_eq!(
        INDEX.matches(r#"id="raw_btn""#).count(),
        1,
        "원문 보기는 머리글 하나"
    );
}

/// '받은 글' 라벨 없음 — '내 활동' 이 대신한다('주고받은 글' 안의 것만 남는다).
#[test]
fn no_received_label() {
    for (file, text) in [("index.html", INDEX), ("app.js", APP), ("view.js", VIEW)] {
        assert_eq!(
            text.matches("받은 글").count(),
            text.matches("주고받은 글").count(),
            "{file}"
        );
    }
}

/// 타임라인 탭 없음 — 요청 흐름은 실행 상세 패널(D26 개정).
#[test]
fn no_timeline_tab() {
    for (file, text) in [("index.html", INDEX), ("app.js", APP), ("view.js", VIEW)] {
        for gone in ["tab_timeline", "panel_timeline", "#t/", "타임라인"] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
}

/// 18 의 화면 문구(plan 설계 "문구 literal" 목록)가 화면 원본에 글자 그대로 있다.
#[test]
fn focus_mention_labels() {
    let all = [INDEX, APP, VIEW].concat();
    for label in [
        "받기 가능",
        "집중",
        "방금 통신",
        "집중 중",
        "통신 기록 없음",
        "상대가 집중 중",
        "글 종류",
        "질의",
        "요청",
        "에이전트 멘션",
        "@ 로 상대 에이전트 부르기",
    ] {
        assert!(all.contains(label), "{label}");
    }
}

/// #1d 의 화면 문구 — 요청 목록 · 상태 넷 · 입력창 안내 · 도구 안내 · 모델 드롭다운. 없어진 완료 표시 문구는 없다.
#[test]
fn request_list_labels() {
    let all = [INDEX, APP, VIEW].concat();
    for label in [
        "받은 요청",
        "보낸 요청",
        r#"wait: "대기", accepted: "수용", rejected: "반려", done: "완료""#,
        "실행 없이 상대 요청 목록에 등록 — 상대가 수용 · 반려합니다",
        "탐색 도구(읽기 전용) 항상 사용",
        r#"<select id="claude_model"></select>"#,
        r#"export const MODELS = ["opus", "sonnet", "haiku"];"#,
    ] {
        assert!(all.contains(label), "{label}");
    }
    for gone in ["완료 표시", "완료 취소", "허용 도구", "model_names"] {
        assert!(!all.contains(gone), "{gone}");
    }
}

/// 이력 정리(#1f) — 복사 아이콘 · 입력창 위 "편집" · 편집 막대 · 반영 범위 세 선택지(intent 결정 literal).
#[test]
fn tidy_literals() {
    let edit = INDEX.find(r#"id="edit_btn""#).expect("편집 단추");
    let reply = INDEX.find(r#"<form id="reply_form""#).expect("입력창");
    assert!(edit < reply, "편집 단추는 입력창 위");
    for label in [
        r#"aria-pressed="false" title="편집 모드 — 끝난 내 글을 골라 삭제 · 편집">편집</button>"#,
        r#"<span id="edit_count">0개 선택</span>"#,
        r#"<button id="edit_delete" type="button" class="danger" disabled>삭제</button>"#,
        r#"<button id="edit_done" type="button" class="primary">완료</button>"#,
        r#"<button value="mine">내 창에서만</button>"#,
        r#"<button value="both">상대 창에도</button>"#,
        r#"<button value="cancel">취소</button>"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        r#"iconButton(COPY_SVG, "복사""#,
        r#""복사했다""#,
        r#"iconButton(PENCIL_SVG, "본문 편집""#,
    ] {
        assert!(APP.contains(code), "{code}");
    }
    assert!(!VIEW.contains(r#"key: "copy""#), "작업 막대에 복사 없음");
    assert!(VIEW.contains(r#""(편집됨)""#));
}

/// 레일 범위(#1g) — '나' 메뉴 6개 · 상대 타일 4개(intent 결정 literal), 사이드바 에이전트 절 · 내 활동 상대 거르기 칸 없음, 접힘 설정.
#[test]
fn rail_scope_literals() {
    for code in [
        r##"{ label: "요청", href: "#requests", count: waiting, icon: "inbox" },"##,
        r##"{ label: "북마크 항목", href: "#saved", icon: "bookmark" }"##,
        r##"{ label: "주고받은 글", href: "#p", icon: "swap" }"##,
        r##"{ label: "즐겨찾기", href: "#starred", count: starred, icon: "star" }"##,
        r##"{ label: "에이전트", href: "#agents", icon: "bot" }"##,
        r##"[{ label: "개요", href: "#", icon: "home" }]"##,
        r##"{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }"##,
    ] {
        assert!(VIEW.contains(code), "{code}");
    }
    for gone in [r#"id="side_agents""#, r#"id="act_peer""#] {
        assert!(!INDEX.contains(gone), "{gone}");
    }
    assert!(!APP.contains("renderSideAgents"));
    assert!(INDEX.contains(r#"<ol id="home_summary" class="rows"></ol>"#));
    for code in [
        "{ ui_folded: keys }",
        "foldSaver.push([...state.folded])",
        "`대기 요청 ${g.waiting}`",
    ] {
        assert!(APP.contains(code), "{code}");
    }
}

/// 레일 그룹(#1i) — "그룹 +" · "대화 추가" · "그룹에서 빼기" · 대화 ⋯ 메뉴 · 떠 있는 상자 · 타일 메뉴("타일 편집" · "이미지 지우기" · "그룹 삭제").
#[test]
fn rail_group_literals() {
    for (file, text, label) in [
        ("index.html", INDEX, r#"id="rail_group_add""#),
        ("index.html", INDEX, ">그룹 +</button>"),
        ("index.html", INDEX, r#"id="conv_more_btn""#),
        (
            "index.html",
            INDEX,
            r#"<div id="float" class="popover float" hidden></div>"#,
        ),
        ("app.js", APP, r#"text: "대화 추가""#),
        ("app.js", APP, r#"text: "그룹에서 빼기""#),
        ("app.js", APP, r#"role: "menuitemcheckbox""#),
        ("view.js", VIEW, "같은 이름의 그룹이 있습니다"),
        ("index.html", INDEX, r#"id="tile_more_btn""#),
        (
            "index.html",
            INDEX,
            r#"<label for="tile_alias" id="tile_alias_label">표시 이름</label>"#,
        ),
        ("app.js", APP, r#"item("pencil", "타일 편집""#),
        ("app.js", APP, r#"item("image-off", "이미지 지우기""#),
        ("app.js", APP, r#""그룹 삭제","#),
        ("app.js", APP, "그룹만 지웁니다 — 대화는 지우지 않습니다."),
        ("app.js", APP, r#""contextmenu""#),
        (
            "index.html",
            INDEX,
            r#"<p id="order_live" class="sr-only" aria-live="polite"></p>"#,
        ),
        ("app.js", APP, r#"draggable: "true""#),
        ("app.js", APP, "Alt + ↑ / ↓ 로 자리를 바꿉니다"),
        ("view.js", VIEW, "번째로 옮김"),
    ] {
        assert!(text.contains(label), "{file}: {label}");
    }
}

/// 안정화(#1h) — "저장된 항목" → "북마크 항목" · 메시지 작업 "북마크"(편집 · 설정의 "저장" 은 그대로) · 첫 화면 "개요" · 메뉴 절 구분선 · 긴 타일 글자 ·
/// 그룹 겹친 카드 · 프로필 알약.
#[test]
fn stabilization_literals() {
    for (file, text) in [("index.html", INDEX), ("app.js", APP), ("view.js", VIEW)] {
        assert!(!text.contains("저장된 항목"), "{file} 에 저장된 항목");
    }
    for label in [
        r#"<strong><span class="ico ico-bookmark" aria-hidden="true"></span>북마크 항목</strong>"#,
        "<h3>북마크 항목 ",
        r#"<strong><span class="ico ico-home" aria-hidden="true"></span><span id="home_title">개요</span></strong>"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        r#"act(on ? "북마크" : "북마크 해제""#,
        r#""북마크 — 북마크 항목에 모았다(이 PC 에만)""#,
        r#"el("li", { class: "nav-sep", role: "separator" })"#,
        r#"el("span", { class: "avatar-name", text: name })"#,
        "`${name} — ${st.text} · 프로필`",
        r#": "개요";"#,
        // 북마크가 아닌 "저장" 은 그대로 — 편집 모드 · 연결 설정
        r#"askScope("고친 본문 저장")"#,
        r#"peerAct("연결 설정 저장""#,
    ] {
        assert!(APP.contains(code), "{code}");
    }
    assert!(APP.contains(r#"iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크""#));
    for rule in [".side-nav .nav-sep {", ".avatar-name { max-width: 12em;"] {
        assert!(CSS.contains(rule), "{rule}");
    }
}

/// 메신저형 대화(#1j) — 말풍선 · 대화 보기 / 원문 보기 · 머리 북마크 n · 말풍선 아이콘(북마크 · 복사) · 대화방 줄 별.
/// "보낸 메시지" 탭 · 보냄 / 받음 태그 · 방향 라벨(사람 → 에이전트) · 막대의 북마크 단추 · '★ 즐겨찾기' 절은 없다.
#[test]
fn messenger_view_literals() {
    for label in [
        r#"<button id="view_btn" type="button" aria-pressed="true">대화 보기</button>"#,
        r#"<button id="raw_btn" type="button" aria-pressed="false">원문 보기</button>"#,
        r#"<span id="saved_n" class="count"></span>"#,
        r#"id="star_btn" class="star-btn" aria-label="즐겨찾기""#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        "side-${side}",
        r#"class: "day-break""#,
        r#"class: "room-preview""#,
        "function starButton(",
        "function bookmarkButton(",
    ] {
        assert!(APP.contains(code), "{code}");
    }
    for gone in [
        "tab_sent",
        "panel_sent",
        "sent_list",
        "conv_tag",
        "보낸 메시지",
    ] {
        assert!(!INDEX.contains(gone), "index.html 에 {gone}");
    }
    for (file, text) in [("app.js", APP), ("view.js", VIEW)] {
        for gone in [
            "kindText",
            "사람 → 에이전트",
            r#"tag: conv.dir"#,
            "★ 즐겨찾기",
            "보낸 메시지",
        ] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
    assert!(
        !VIEW.contains(r#"key: "save""#),
        "북마크는 말풍선 아이콘 — 막대에 없음"
    );
    for rule in [".msg.side-mine", ".day-break", ".seg", ".star-btn"] {
        assert!(CSS.contains(rule), "{rule}");
    }
}

/// 메뉴 · 상태 색 · 레일 아이콘 A안(#1j) — 즐겨찾기 화면 · 프로필 상태 고르기 · 앱과 연결 끊김 · 사람 / 사람 + 그림자 모양 + 아래 이름(레일 72px) ·
/// 타일 메뉴 위로 · 아래로. 없어진 것 — 상단 "받기 가능" 칩 · 1h 글자 타일 · 겹친 카드.
#[test]
fn menu_status_rail_literals() {
    for label in [
        r#"<section id="starred" class="view" hidden>"#,
        r#"<strong><span class="ico ico-star" aria-hidden="true"></span>즐겨찾기</strong>"#,
        r#"<div id="starred_list" class="starred-list"></div>"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        r#""상태 — 아이콘 테두리 색""#,
        r#""menuitemradio""#,
        "const OFFLINE_AFTER = 2;",
        "function keepFocus(",
        "function stepMove(",
        r#"["위로 옮기기", -1],"#,
        r#"["아래로 옮기기", 1],"#,
        r#"class: "tile-name""#,
        "tileFace(g.icon, true)",
        "이미지를 지웠습니다 — 사람 모양으로 보입니다",
        r##"go("#");"##,
    ] {
        assert!(APP.contains(code), "{code}");
    }
    for code in [
        r#"{ key: "avail", label: "받기 가능", hint: "새 요청을 받는다""#,
        r#"{ key: "focus", label: "집중", hint: "새 요청을 거절""#,
        r#"{ key: "offline", label: "앱과 연결 끊김", hint: "자동 표시""#,
        r#""starred", "saved""#,
    ] {
        assert!(VIEW.contains(code), "{code}");
    }
    for rule in [
        "grid-template-columns: 72px ",
        ".st-online { --st: var(--ok); }",
        ".st-focus { --st: var(--run); }",
        ".st-offline { --st: var(--danger); }",
        ".rail-btn .tile-name {",
        ".msg { gap: 10px; max-width: 90%; }",
    ] {
        assert!(CSS.contains(rule), "{rule}");
    }
    for (file, text) in [("index.html", INDEX), ("app.js", APP), ("app.css", CSS)] {
        for gone in [
            "focus_chip",
            "status-chip",
            "tile-text",
            "tileText",
            "renderFocusChip",
        ] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
    assert!(!VIEW.contains("export function tileText"));
    assert!(!CSS.contains(".rail-btn.group { box-shadow"));
    assert!(
        !VIEW.contains("집중 — 요청 받지 않음"),
        "프로필 메뉴는 받기 가능 · 집중 고르기"
    );
}

/// 상대 그룹 · 설정 정리(#1j) — "페어링" 카드 · 초대 두 단추 같은 모양 · 연결 목록 = 레일 그룹별 절(끌기 · "그룹 ▾" · "+ 그룹 추가" · 이름 바꾸기 ·
/// 삭제) · 타일 메뉴 그룹 체크. 없어진 것 — "주소 · 코드로 직접 추가" · "읽기 전용" 표시 · 연결 · 에이전트의 그룹 칸("그룹 없음").
#[test]
fn peer_group_settings_literals() {
    for label in [
        "<h2>페어링<span class=\"help-wrap\">",
        r#"<button type="submit" class="primary">추가 · 승인</button>"#,
        r#"<button type="button" id="peer_group_add">+ 그룹 추가</button>"#,
        r#"<section id="peers" class="card">"#,
        r#"<p id="invite_msg" role="status"></p>"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        r#"text: "그룹 ▾","#,
        r#"el("span", { text: "그룹 — 체크하면 넣고, 풀면 뺀다" })"#,
        r#"ico("plus"), el("span", { text: "새 그룹에 넣기…" })"#,
        "/api/groups/${id}/peers",
        r#""data-peer-section""#,
        "function openGroupRename(",
        "function groupDeleteForm(",
        "peerMove(d.peer, d.from, to, state.groups)",
    ] {
        assert!(APP.contains(code), "{code}");
    }
    for code in [
        r#"export const NO_PEER_GROUP = "그룹에 없는 연결";"#,
        "에 넣었습니다 — ${peer}${josa(peer, \"과\", \"와\")}의 대화 ${n}개가 함께 보입니다",
    ] {
        assert!(VIEW.contains(code), "{code}");
    }
    for gone in [
        "주소 · 코드로 직접 추가",
        r#"id="add_form""#,
        "읽기 전용</span>",
        r#"id="peer_group""#,
        r#"id="agent_group""#,
    ] {
        assert!(!INDEX.contains(gone), "index.html 에 {gone}");
    }
    for (file, text) in [("app.js", APP), ("view.js", VIEW)] {
        for gone in ["그룹 없음", "add_form", "groupFilter", "fillGroupSelect"] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
    // 연결 목록의 "읽기 전용" 표시는 없다 — #1o 공유 대화 머리(`view.js` `shareHead`)는 별개.
    assert!(!APP.contains(r#""읽기 전용""#), "app.js 에 \"읽기 전용\"");
}

/// 다듬기(#1k) — 에이전트 화면 · 주고받은 글 안내문 없음 · 내 에이전트 연필 · 주고받은 글 상대 타일 · 에이전트 칩 · "다시 보내기" 없음 · 레일 + → 페어링 ·
/// 설정 두 열(페어링 단추 둘 · 관리 줄 · 줄 타일 · 에이전트 등록).
#[test]
fn polish_literals() {
    for gone in [
        "상대 에이전트를 누르면 새 대화, 내 에이전트를 누르면 설정으로",
        "주고받은 글 — 모든 대화, 최신순",
    ] {
        assert!(!INDEX.contains(gone), "index.html 에 {gone}");
    }
    for (file, text) in [("index.html", INDEX), ("app.js", APP), ("view.js", VIEW)] {
        for gone in [
            r#"label: "다시 보내기""#,
            r#"key === "resend""#,
            "다시 보내기로",
        ] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
    for label in [
        r#"<button type="button" id="accept_btn" class="primary big" aria-expanded="false" aria-controls="invite_accept">"#,
        r#"<section id="invite_accept" class="sub" hidden>"#,
        r#"<button type="button" id="agent_new_btn" class="primary" aria-expanded="false" aria-controls="agent_new">에이전트 등록</button>"#,
        "한쪽이 초대를 만들어 보내고, 받은 쪽이 붙여 넣으면 연결됩니다.",
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for code in [
        "function manageItem(",
        r#"text: "관리","#,
        "`${a.name} 편집`",
        "function editAgent(",
        r#""data-peer-sub": String(p.id)"#,
        "function peerSub(",
        "`peer${p.id}_${s.group ?? \"none\"}`",
        r#"rowEls(rs, "주고받은 글 없음", { tile: true })"#,
        r#"class: "chip agent-chip""#,
        r#"goSettings("conn", "invite_text")"#,
        r#"rowTile("agent-tile", null)"#,
    ] {
        assert!(APP.contains(code), "{code}");
    }
    // 받은 대화만 '내 ' — 보낸 대화 · 공유 대화(#1o)는 에이전트 이름 그대로.
    assert!(VIEW.contains("agent: conv.dir === \"in\" ? `내 ${conv.agent}` : conv.agent"));
    for rule in [".pair-btns {", ".manage-head {", ".rows a.row-with-tile {"] {
        assert!(CSS.contains(rule), "{rule}");
    }
}

/// 아이콘 · 말풍선 색 · 설정 정리(#1l) — 메뉴 · 화면 제목 · 설정 트리 · ⋯ · 새로 만들기 아이콘(16px 한 벌), 상대 글 = 타일 색, 설정 '?' 도움말 · 한 열 ·
/// 사용자 정보 팝업 · 등록 칸 보이기 · 숨기기 · 타일 순서. 없어진 것 — 내 정보 카드 · 에이전트 검색 · "새 에이전트 등록" · 늘 보이는 카드 안내 · 두 열.
#[test]
fn icons_settings_literals() {
    for label in [
        r#"<dialog id="me_dialog" class="me-dialog" aria-labelledby="me_title">"#,
        r#"<span class="ico ico-user" aria-hidden="true"></span>사용자 정보</h2>"#,
        r#"<span id="me_fingerprint" class="mono">"#,
        r#"<div id="agent_new" class="new-agent" hidden>"#,
        r#"<label for="agent_prompt">추가 지시</label><span class="help-wrap">"#,
        "추가 지시 사용 예",
        r#"<select id="ui_rail_order">"#,
        r#"<option value="peers_first">사용자 → 그룹</option>"#,
        r#"<option value="groups_first">그룹 → 사용자</option>"#,
        "<h2>화면 — 테마 · 타일 순서</h2>",
        r#"<button type="button" class="help" aria-label="도움말" aria-describedby="tip_conn">?</button>"#,
        "\"프로필 → 사용자 정보 → 지문\"",
        r#"<strong><span class="ico ico-bot" aria-hidden="true"></span><span id="agents_title">에이전트</span></strong>"#,
        r#"<strong><span class="ico ico-swap" aria-hidden="true"></span><span id="peer_title"></span></strong>"#,
        r#"<strong><span class="ico ico-bell" aria-hidden="true"></span>내 활동</strong>"#,
        r#"<strong><span class="ico ico-inbox" aria-hidden="true"></span>요청</strong>"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for gone in [
        r#"<section id="me" class="card">"#,
        r#"id="agent_q""#,
        "<summary>새 에이전트 등록</summary>",
        r#"<details class="guide">"#,
        "settings-cols",
        r#"<p class="hint">한쪽이 초대를 만들어 보내고"#,
        r#"<p class="hint">페어링된 상대가 부를 수 있는"#,
        "\"내 정보 → 지문\"",
        r#"width="15" height="15""#,
    ] {
        assert!(!INDEX.contains(gone), "index.html 에 {gone}");
    }
    for code in [
        "function ico(name)",
        r#"el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label }))"#,
        r#""data-tone": colorOf(conv.peer_id)"#,
        r#"$("me_dialog").showModal();"#,
        "function showAgentNew(on)",
        r#"if (i.focus === "agent_name") showAgentNew(true);"#,
        "function applyRailOrder(order)",
        r#""ui_rail_order","#,
        r#"item(dir < 0 ? "arrow-up" : "arrow-down", text"#,
    ] {
        assert!(APP.contains(code), "{code}");
    }
    assert!(!APP.contains("agent_q"));
    assert!(
        !VIEW.contains("자동 왕복 대화"),
        "새로 만들기 메뉴에 자동 왕복 대화 없음"
    );
    assert!(VIEW.contains(r#"{ key: "me", label: "사용자 정보" }"#));
    assert!(!VIEW.contains(r#"{ id: "me", label: "내 정보" }"#));
    for rule in [
        ".ico { display: inline-block; flex: none; width: 16px; height: 16px;",
        r#".msg[data-tone="peer-1"] { --tone: var(--peer-1); }"#,
        ".msg.side-theirs .bubble::before { content: \"\"; position: absolute; inset: 0; z-index: -1; border-radius: inherit; background: var(--tone, var(--wait)); opacity: 0.07;",
        ".msg.side-mine .bubble { isolation: isolate; background: var(--surface); color: var(--text);",
        ".help-wrap:hover .tip, .help:focus + .tip { display: block; }",
    ] {
        assert!(CSS.contains(rule), "{rule}");
    }
    assert!(
        !CSS.contains("color-mix("),
        "macOS 12 Safari 15 에 없는 color-mix 안 씀"
    );
    assert_eq!(
        CSS.matches(
            "
.ico { "
        )
        .count(),
        1,
        "아이콘 크기 정의는 한 곳(16px)"
    );
}

/// 컨텍스트 표시 · 압축 · 비우기(#1m) — 대화 머리 단추 · 메뉴 · 구분선 · 설정 자동 압축.
#[test]
fn context_literals() {
    for label in [
        r#"<button id="ctx_btn" type="button" class="tool-btn ctx-btn" aria-haspopup="menu""#,
        r#"<span class="ico ico-gauge" aria-hidden="true"></span><span id="ctx_text">컨텍스트 알 수 없음</span>"#,
        r#"<label for="auto_compact">자동 압축 — 컨텍스트 80% 넘으면</label>"#,
        r#"<input type="checkbox" id="auto_compact">"#,
    ] {
        assert!(INDEX.contains(label), "{label}");
    }
    for label in [
        "압축 — 요약 후 새 세션",
        "비우기 — 세션만, 이력은 그대로",
        "자동 압축(80%)",
        "상대가 0.5.0 이상이어야 합니다",
        "export const CONTEXT_LIMIT = 1_000_000;",
    ] {
        assert!(VIEW.contains(label), "{label}");
    }
    for label in [
        "/api/conversations/${conv.id}/context",
        "\"auto_compact\",",
        "m.author === \"context\" ? contextEl(m) : messageEl(conv, m)",
    ] {
        assert!(APP.contains(label), "{label}");
    }
    for rule in [
        ".ctx-break {",
        ".ctx-btn[data-level=\"warn\"]",
        ".ctx-btn[data-level=\"high\"]",
    ] {
        assert!(CSS.contains(rule), "{rule}");
    }
}

/// 대화방 목록 나눔 · 말풍선 음영(#1o) — 머리 문구 · 나눔 함수 · 접힘 열쇠 · 즐겨찾기 그대로 · 층 0.14 / 0.07 · 오른쪽 띠, 옛 규칙 없음.
#[test]
fn chat_list_split_literals() {
    for (file, text, lits) in [
        (
            "view.js",
            VIEW,
            &[
                r#"{ dir: "out", title: "내 대화" }"#,
                r#"{ dir: "in", title: "agent 대화" }"#,
                "export function splitByDir(",
                "export function splitKey(",
            ][..],
        ),
        (
            "app.js",
            APP,
            &[
                "splitByDir(s.rows)",
                "foldEl(`convs:${s.peer}`",
                "starredSections(",
                r#"$("rail").addEventListener("contextmenu", (ev) => ev.preventDefault());"#,
            ][..],
        ),
        (
            "app.css",
            CSS,
            &[
                ".msg.side-mine .bubble { isolation: isolate; background: var(--surface); color: var(--text); border: 1px solid var(--border); border-right: 3px solid var(--accent); }",
                ".msg.side-mine .bubble::before { content: \"\"; position: absolute; inset: 0; z-index: -1; border-radius: inherit; background: var(--accent); opacity: 0.14;",
                "background: var(--tone, var(--wait)); opacity: 0.07;",
                ".msg.side-mine .bubble a { color: var(--sent); }",
                ".dir-head {",
            ][..],
        ),
    ] {
        for lit in lits {
            assert!(text.contains(lit), "{file} 에 {lit}");
        }
    }
    for gone in [
        ".msg.side-mine .bubble { background: var(--accent); color: var(--on-accent);",
        "opacity: 0.12;",
        "color-mix(",
    ] {
        assert!(!CSS.contains(gone), "app.css 에 {gone}");
    }
}

/// CSS 토큰 `--{name}: #RRGGBB;` → (r, g, b).
fn token(name: &str) -> [f64; 3] {
    let key = format!("--{name}: #");
    let at = CSS.find(&key).unwrap_or_else(|| panic!("토큰 {name}")) + key.len();
    let hex = &CSS[at..at + 6];
    [0, 2, 4].map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap() as f64)
}

/// 규칙(선택자로 시작하는 줄)의 `opacity` 값.
fn rule_opacity(selector: &str) -> f64 {
    let line = CSS
        .lines()
        .find(|l| l.starts_with(selector))
        .unwrap_or_else(|| panic!("규칙 {selector}"));
    let v = line.split("opacity: ").nth(1).unwrap();
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

/// 말풍선 글자 대비(#1o A3) — 2 테마 × (내 글 1 + 상대 글 타일 5색) × 글자 3(본문 · muted · 링크) = 36 경우 모두 4.5:1 이상. 바탕 = 색 층 × 불투명도 + surface.
#[test]
fn bubble_contrast() {
    let mine = rule_opacity(".msg.side-mine .bubble::before");
    let theirs = rule_opacity(".msg.side-theirs .bubble::before");
    assert_eq!((mine, theirs), (0.14, 0.07), "시안 Ⓐ 층 불투명도");
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

/// 내용이 곧 설명인 메뉴의 설명 지우기(#1o 사용자 지시) — 개요 · 요청 · 즐겨찾기 · 북마크 머리 설명 · 개요 카드의 접속 · 대화 수 · 최근, 에이전트 "새 대화" 단추,
/// 타일 편집 안내 = "이 PC 에서만 표시".
#[test]
fn hint_cleanup_literals() {
    for gone in [
        "상대별 대화 요약 — 대화 수 · 안 읽음 · 대기 요청 · 최근 활동, 누르면 그 대화로",
        "실행 없이 모인 요청 — 받은 요청은 수용 · 반려 · 완료로 답하고, 보낸 요청은 상대가 바꾼 상태를 봅니다",
        "☆ 로 넣은 대화방 — 이 PC 에만, 대화방 줄의 ☆ 로 넣고 뺍니다",
        "메시지 작업 '북마크' 로 모은 글 — 이 PC 에만, 북마크한 순서 최신부터",
        "— 메시지 작업 '북마크' 로 모읍니다",
        "이 PC 에서만 보입니다. 그림은 가운데를 정사각형으로 잘라 작게 줄여 저장합니다.",
        "의 에이전트 — 누르면 새 대화",
        "text: `최근 ${formatTime(g.last)}`",
        "text: `대화 ${g.convs}`",
    ] {
        for (file, text) in [("index.html", INDEX), ("app.js", APP)] {
            assert!(!text.contains(gone), "{file} 에 {gone}");
        }
    }
    assert!(INDEX.contains(r#"<p class="hint">이 PC 에서만 표시</p>"#));
    assert!(APP.contains(r#"class: "small agent-new""#) && APP.contains(r#"text: "새 대화""#));
}
/// 부모 툴 메뉴용 명령(#1n) — 사용법 세 줄 · 종료 코드 8.
#[test]
fn control_usage_literals() {
    for line in [
        "coworkAgent start [--parent <PID>]",
        "coworkAgent open [--parent <PID>]",
        "coworkAgent stop
",
        "종료 코드 8 = 시작 실패",
    ] {
        assert!(an_app::cli::USAGE.contains(line), "{line}");
    }
}
