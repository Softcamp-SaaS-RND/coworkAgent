// 셸 — 주소 · 사이드 내비 · 메뉴 · 진입 토큰.
import { convRows, convSections } from "./conv.js";
import { SETTINGS_ALIAS, settingsTree } from "./settings.js";

/// 화면 위치(해시) — `#c/{id}` 대화(`/m/{글}` = 그 글 위치) · `#s/{id}` 그 대화 상대에게 보낸 글 · `#p/{id}` 상대와 주고받은 글 ·
/// `#activity` 내 활동 · `#requests` 요청 목록 · `#starred` 즐겨찾기 · `#saved` 북마크 · `#agents` 에이전트 · `#settings`(`/{화면}` = 그 화면 · `/{별칭}` = `SETTINGS_ALIAS`, 빈 · 모르는 = 연결) ·
/// `#new` 세션 생성. 그 밖은 안내 화면.
export function route(hash) {
  const h = (hash ?? "").replace(/^#/, "");
  let m = /^c\/(\d+)(?:\/m\/(\d+))?$/.exec(h);
  if (m) return m[2] ? { view: "conversation", id: Number(m[1]), msg: Number(m[2]) } : { view: "conversation", id: Number(m[1]) };
  m = /^s\/(\d+)$/.exec(h);
  if (m) return { view: "sent", id: Number(m[1]) };
  m = /^p(?:\/(\d+))?$/.exec(h);
  if (m) return { view: "peer", id: m[1] ? Number(m[1]) : null };
  if (h === "agents/convs") return { view: "agentConvs" };
  if (["activity", "requests", "starred", "saved", "agents", "new"].includes(h)) return { view: h };
  m = /^settings(?:\/(\w+))?$/.exec(h);
  if (m) {
    if (SETTINGS_ALIAS[m[1]]) return { view: "settings", page: SETTINGS_ALIAS[m[1]], sub: m[1] };
    return { view: "settings", page: settingsTree().some((n) => n.id === m[1]) ? m[1] : "conn", sub: null };
  }
  return { view: "home" };
}

/// 배경 새로 고침을 미룰 초점 — 입력칸에서 쓰는 중일 때만(`tag` = 초점 요소 `tagName`). 상대별 절 머리(`summary`) · 단추에 초점이 남아도 다시
/// 그린다(절을 접은 뒤 목록이 멈추지 않게).
export function holdsRender(tag) {
  return ["INPUT", "TEXTAREA", "SELECT"].includes(tag);
}

/// 사이드바 내비 — '나'(`tile` = `null`) = 전부를 상대별로 묶어 보는 메뉴, 상대 타일 = 그 상대만(내 활동은 '나' 에만).
/// 그룹 타일(`tile` = `"group"`) = 개요(그룹 요약) · 요청 · 즐겨찾기 · 북마크 — 내 활동('나' 에만) · 에이전트(대화 단위 아님) 없음.
/// 세 절 — 상단 개요(모든 타일 — 상대는 그 상대 개요) / 중단 내 활동 · 요청 / 하단 즐겨찾기 · 북마크 · 에이전트(사용자가 모은 · 등록한 것), 절 사이 `{ sep: true }`
/// (빈 절은 건너뛴다). 내 활동은 안 본 수(`count`), 요청은 받은 요청 중 대기 수(`waiting`).
/// '에이전트' = 링크 없는 머리(`head`) + 들여쓴 하위(`sub`) — 등록된 에이전트 · 에이전트 응답('나' 에만, 상대가 연 대화, 수 = 안 읽음 `agentUnread`).
export function sideNav(tile, unseen = 0, waiting = 0, starred = 0, agentUnread = 0) {
  const me = tile == null;
  const group = tile === "group";
  // '주고받은 메시지' 메뉴는 없다(내 활동과 겹친다 — 옛 주소 `#p` 화면은 남는다).
  // `icon` = `app.css` `.ico-{이름}`(화면 머리 제목과 같은 아이콘).
  const top = [{ label: "개요", href: "#", icon: "home" }];
  const mid = [
    ...(me ? [{ label: "내 활동", href: "#activity", count: unseen, icon: "bell" }] : []),
    { label: "요청", href: "#requests", count: waiting, icon: "inbox" },
  ];
  const end = [
    { label: "즐겨찾기", href: "#starred", count: starred, icon: "star" },
    { label: "북마크", href: "#saved", icon: "bookmark" },
    ...(group
      ? []
      : [
          { label: "에이전트", icon: "bot", head: true },
          { label: "등록된 에이전트", href: "#agents", icon: "bot", sub: true },
          // 에이전트 응답 = 내 에이전트가 어느 사용자에게 답했는지 — '나' 에만.
          ...(me ? [{ label: "에이전트 응답", href: "#agents/convs", icon: "chat", sub: true, count: agentUnread }] : []),
        ]),
  ];
  return [top, mid, end].filter((s) => s.length).flatMap((s, i) => (i ? [{ sep: true }, ...s] : s));
}

/// 세션 생성 이름 — 상단 + 메뉴 · 화면 제목 · 사이드바 · 에이전트 줄 단추가 쓴다.
export const SESSION_NEW = "세션 생성";

/// 상단 바 "+ 새로 만들기" 메뉴 — 각 화면과 그 화면에서 초점을 줄 칸 · 아이콘 · 함께 누를 단추(`press`). 자동 왕복은 세션 생성 화면 안에서(메뉴에 없다).
export function newMenu() {
  return [
    { label: SESSION_NEW, href: "#new", focus: "chat_peer", icon: "chat" },
    { label: "초대 생성", href: "#settings/invite_make", focus: "window_btn", icon: "plus", press: "window_btn" },
    { label: "초대 수락", href: "#settings/invite_accept", focus: "invite_text", icon: "enter" },
    { label: "에이전트 등록", href: "#settings/agent_new", focus: "agent_engine_claude", icon: "bot" },
  ];
}

/// 상단 바 프로필 메뉴 — 상태 고르기(받기 가능 · 집중 — 하나만 켜짐) · 앱과 연결 끊김(자동 표시, 고를 수 없음) | 사용자 정보(팝업) · 프로필 편집('나' 타일 편집) ·
/// 알림 일시 중지(켜짐 표시) · 설정. `state` = 그 상태의 색(`presenceOf` 열쇠), 상태가 아닌 항목은 `icon`.
export function profileMenu(paused, focus, offline = false) {
  return [
    { key: "avail", label: "받기 가능", hint: "새 요청을 받습니다", state: "online", radio: true, checked: focus !== true },
    { key: "focus", label: "집중", hint: "새 요청을 거절합니다", state: "focus", radio: true, checked: focus === true },
    { key: "offline", label: "앱과 연결 끊김", hint: "자동 표시", state: "offline", info: true, current: offline === true },
    { sep: true },
    { key: "me", label: "사용자 정보", icon: "user" },
    { key: "tile", label: "프로필 편집", icon: "pencil" },
    { key: "pause", label: "알림 일시 중지", icon: "bell", checked: paused === true },
    { key: "settings", label: "설정", icon: "gear", href: "#settings" },
  ];
}

/// 즐겨찾기 화면 — ★ 대화방만, 대화방 절 그대로(`paired` 를 주면 상대별 절). 빈 절은 뺀다 — 하나도 없으면 빈 배열.
export function starredSections(convs, peerNames, paired = null) {
  return convSections(
    convRows(convs.filter((c) => c.starred === true), peerNames, null),
    paired,
    peerNames,
  ).filter((s) => s.rows.length);
}

/// 사이드바 머리 부제 — '나' = 이름(별칭이면 호스트 더), 상대 = 별칭이면 "{이름} · {호스트}" 아니면 호스트. 지문 · "페어링 됨" · 접속 문구는 없다(접속은 레일 점).
export function sideSub(kind, info, alias) {
  if (kind === "me") return [info.name, alias ? info.host : ""].filter(Boolean).join(" · ");
  return info.alias ? `${info.name} · ${info.host}` : info.host;
}

/// 탭 메모리 토큰(쿠키 없음). 주소의 `t`(64 hex)가 있으면 그것, 없으면 탭에 저장된 값. `clean` = 주소창에서 `t` 를 지울지.
export function entryToken(search, stored) {
  const hex = (t) => typeof t === "string" && /^[0-9a-f]{64}$/.test(t);
  const t = new URLSearchParams(search ?? "").get("t");
  if (hex(t)) return { token: t, clean: true };
  return { token: hex(stored) ? stored : null, clean: t !== null };
}

/// 토큰이 없거나 거절됐을 때 화면 위 안내.
export const DISCONNECTED = "연결이 끊겼습니다. 트레이 아이콘 메뉴 '열기' 로 다시 여세요.";
