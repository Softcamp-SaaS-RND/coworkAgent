//! 화면 문구 · favicon 링크 literal(plan 해석 1) — 웹UI 원본(빌드 없는 정적 파일)을 그대로 읽는다.
use an_net::invite::Invite;

const INDEX: &str = include_str!("../web/index.html");
const APP: &str = include_str!("../web/app.js");
const VIEW: &str = include_str!("../web/view.js");
const TRAY: &str = include_str!("../src/tray.rs");

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
        r#"<h2>초대 생성</h2>"#,
        r#"<button type="button" id="window_btn">초대 생성</button>"#,
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
        "자동 왕복 대화",
        "초대 생성",
        "초대 수락",
        "에이전트 등록",
        "표시 이름 · 아이콘",
        "알림 일시 중지",
        "환경설정",
        "내 활동",
        "저장된 항목",
        "에이전트",
        "주고받은 글",
        "제목 검색",
        "★ 즐겨찾기",
        "즐겨찾기",
        "제목 편집",
        "보낸 메시지",
        "원문 보기",
        "세부정보",
        "저장",
        "복사",
        "다시 보내기",
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
        "집중 — 요청 받지 않음",
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
        r##"{ label: "요청", href: "#requests", count: waiting },"##,
        r##"{ label: "저장된 항목", href: "#saved" },"##,
        r##"{ label: "주고받은 글", href: tile == null ? "#p" : `#p/${tile}` },"##,
        r##"{ label: "에이전트", href: "#agents" },"##,
        r##"[{ label: "대화", href: "#" }, { label: "내 활동", href: "#activity", count: unseen }, ...scoped]"##,
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
        ("app.js", APP, r#"item("타일 편집""#),
        ("app.js", APP, r#"item("이미지 지우기""#),
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
