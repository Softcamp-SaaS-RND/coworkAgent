// 화면 조립 — 시안 v4(Nocturne) 5영역: 상단 바(통합 검색 · + 새로 만들기 · 알림 · 프로필) · 레일 · 사이드바(내비 · 대화방 ·
// 에이전트) · 메인(대화 · 내 활동 · 북마크 항목 · 에이전트 · 설정 트리) · 실행 상세. 값은 textContent 로만 넣는다(HTML 주입
// 없음) — 원격에서 온 이름 · 본문도 같다. HTML 문서 본문만 sandbox iframe(srcdoc · CSP)으로. 판단 로직은 view.js(node 테스트).
import {
  entryToken,
  DISCONNECTED,
  stateText,
  peerState,
  convLabel,
  splitKey,
  shareHead,
  shareCandidates,
  MAX_OBSERVERS,
  shortName,
  josa,
  authorLabel,
  bubbleSide,
  dayBreaks,
  relayText,
  keepsConversation,
  convRows,
  tileKeeps,
  formatDuration,
  formatTime,
  route,
  Latest,
  previewWindows,
  previewBubbles,
  previewKey,
  previewStaleOf,
  previewTabs,
  PREVIEW_MSGS,
  settingsTree,
  settingsGroupSections,
  convCopy,
  convClosed,
  convMenuItems,
  LEAVE_CONFIRM,
  SESSION_NEW,
  sideSub,
  composeDefaults,
  requestNotice,
  inboxRows,
  peerSummaryRows,
  pageMore,
  peerMsgsSig,
  mergePage,
  PEER_PAGE,
  isHtmlDoc,
  htmlSrcdoc,
  markdownBlocks,
  presenceOf,
  starredSections,
  displayName,
  sideNav,
  newMenu,
  profileMenu,
  convSections,
  convMeta,
  activityRows,
  ACT_KINDS,
  insertMarkdown,
  enterSends,
  speakerIcon,
  RAIL_W,
  railSize,
  noteCount,
  groupSettingsList,
  pastSlop,
  railSlot,
  peerLeaves,
  tileUnread,
  avatarKind,
  timeline,
  runSteps,
  messageActions,
  editedTag,
  convListParts,
  visibleConvRows,
  agentConvSections,
  boardColumns,
  canDrop,
  reqFeedback,
  searchGroups,
  lastRun,
  contextSize,
  contextLabel,
  contextBusy,
  ctxMenu,
  contextNote,
  presence,
  refusedText,
  progressText,
  outboxText,
  runsSummary,
  portText,
  messagesSigOf,
  askTag,
  requestRows,
  reqBoxes,
  reqHead,
  peerWeeksOptions,
  REQ_BTN,
  mentionQuery,
  mentionTarget,
  hasMention,
  mentionToken,
  preview,
  TOOLS_NOTE,
  PARENT_NOTE,
  MODELS,
  EFFORTS,
  peerGroups,
  peerSummary,
  waitingByPeer,
  SendLatest,
  holdsRender,
  sortByPos,
  inGroup,
  groupPeers,
  scopeQuery,
  groupNameError,
  membershipDiff,
  colorOf,
  moveInOrder,
  stepTarget,
  dropKind,
  withOrder,
  movedText,
  groupLeaves,
  groupUnread,
  orderToApply,
  loadersFor,
  mergeChanges,
  OneAtATime,
} from "/view.js";

const $ = (id) => document.getElementById(id);

// 탭 메모리 토큰 — 트레이 "열기" 가 준 `/?t=` 를 가져가고 주소창에서 지운다(쿠키 없음: 쿠키는 포트를 가리지 않는다).
const TOKEN_KEY = "an_token";
const entry = entryToken(location.search, sessionStorage.getItem(TOKEN_KEY));
if (entry.clean) history.replaceState(null, "", "/" + location.hash);
if (entry.token) sessionStorage.setItem(TOKEN_KEY, entry.token);
const token = entry.token;

/// 토큰이 없거나 거절됨 — 화면 위에 다시 열기 안내(한 번만).
function disconnected() {
  if ($("disconnected")) return;
  document.body.prepend(el("div", { id: "disconnected", class: "disconnected", role: "alert", text: DISCONNECTED }));
}
if (!token) {
  disconnected();
  throw new Error("토큰 없음");
}

/// JSON 을 받거나, 실패 사유를 담은 Error 를 던진다.
async function api(method, path, body) {
  const init = { method, headers: { Authorization: `Bearer ${token}` } };
  if (body !== undefined) {
    init.headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  let res;
  try {
    res = await fetch(path, init);
  } catch (e) {
    netResult(false);
    throw e;
  }
  netResult(true);
  if (res.status === 401) disconnected();
  const text = await res.text();
  let data = null;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    // JSON 이 아닌 오류 본문(요청 형식 거부 등)은 그대로 사유로.
    if (res.ok) throw new Error("응답이 JSON 이 아닙니다");
    throw new Error(`HTTP ${res.status} ${text.slice(0, 200)}`);
  }
  if (!res.ok) throw new Error(data?.error ?? "HTTP " + res.status);
  return data;
}

/// 앱(이 PC 서버)과의 연결(#1j) — 요청이 닿지 못한 것(fetch 실패)이 연달아 `OFFLINE_AFTER` 번이면 끊김(프로필 붉음), 한 번이라도 닿으면 되돌린다.
/// HTTP 오류 · 401 은 닿은 것이다(401 은 위 다시 열기 안내). 5초 새로 고침이 계속 묻는다.
const OFFLINE_AFTER = 2;

function netResult(ok) {
  const was = appOffline();
  state.netFails = ok ? 0 : state.netFails + 1;
  if (was !== appOffline()) renderAvatar();
}

function appOffline() {
  return state.netFails >= OFFLINE_AFTER;
}

/// 요소 만들기 — `text` 는 textContent, `class` · 속성, `on*` 는 이벤트.
/// 아이콘(#1l) — `app.css` `.ico-{이름}` 모양 · 크기 16px 한 벌. 읽기 도구에는 숨긴다(옆 글자가 이름).
function ico(name) {
  return el("span", { class: `ico ico-${name}`, "aria-hidden": "true" });
}

function el(tag, props = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (v == null || v === false) continue;
    if (k === "text") e.textContent = v;
    else if (k === "class") e.className = v;
    else if (k.startsWith("on")) e.addEventListener(k.slice(2), v);
    else e.setAttribute(k, v === true ? "" : v);
  }
  e.append(...children);
  return e;
}

function button(label, onClick, cls) {
  return el("button", { type: "button", text: label, onclick: onClick, class: cls });
}

/// `label` 작업을 하고 결과를 `out`(상태 줄)에 적는다.
async function act(label, fn, out = "status_msg", after) {
  $(out).textContent = label + " …";
  try {
    const done = await fn();
    $(out).textContent = typeof done === "string" ? done : label + " 완료";
  } catch (e) {
    $(out).textContent = label + " 실패: " + e.message;
  }
  if (after) await after();
}

const state = {
  me: null,
  peers: [],
  peerNames: new Map(),
  agents: [],
  convs: [],
  settings: {},
  railPeer: null, // 레일에서 고른 상대(null = 나 = 전부)
  railGroup: null, // 레일에서 고른 그룹(#1i — 고르면 railPeer = null)
  groups: [], // 레일 그룹(`GET /api/groups` — 이 PC 에만, 레일 순서)
  current: null, // 열린 대화(요약)
  open: new Set(), // 펼쳐 둔 연결 · 에이전트(다시 그려도 유지)
  unseen: 0, // 안 본 내 활동 — 고른 타일 범위(사이드바 내비)
  unseenAll: 0, // 안 본 내 활동 — 전부(종 점)
  reqWaiting: 0, // 받은 요청 중 대기 — 고른 타일 범위(사이드바 '요청')
  waitingBy: new Map(), // 받은 요청 중 대기 — 상대별('나' 의 '대화' 요약, #1g)
  folded: new Set(), // 접은 상대별 절 열쇠 `{화면}:{상대}`(#1g, 설정 `ui_folded` — 시작할 때 읽는다)
  homeView: "preview", // 개요 보기(#1z-4) — `preview` · `list`(설정 `ui_home_view` — 시작할 때 읽는다)
  previewTab: new Map(), // 개요 미리보기 — 상대 id → 고른 대화 id(저장하지 않는다, 없으면 최근 활동 대화)
  previewMsgs: new Map(), // 개요 미리보기 — 대화 id → `{key, ms, seq}` 마지막으로 받은 글(`previewKey` · 받은 차례)
  previewStale: new Set(), // 개요 미리보기 — 다시 읽을 대화 id(변경 신호 · 탭 고름 · 읽기 실패)
  previewAll: true, // 개요 미리보기 — 창의 대화를 전부 다시 읽기(개요에 들어옴 · 보기 바꿈 · 전체 변경 신호)
  reqBox: null, // '요청' 화면 — `null` = 개요(#1q), 탭 = 받은(`theirs`) · 보낸(`mine`)
  peerAgents: new Map(), // 상대 id → 그 PC 의 에이전트 이름(`null` = 받지 못함) — 타일을 고를 때 받는다(원격 왕복이라 새로 고침마다 묻지 않는다)
  detailOpen: window.innerWidth >= 1280, // 실행 상세 — 넓은 창이면 열고 시작(저장하지 않는다)
  focusAfter: null, // 다음 화면을 그린 뒤 초점을 줄 칸 id
  peerHeads: [], // 주고받은 메시지 상대 절 머리(#1q)
  peerPages: new Map(), // 상대 id → 펼친 절에 받은 글(#1q — 20건씩)
  peerLoads: new Map(), // 상대 id → 그 상대 글 받기 차례(#1q R2 — 한 번에 하나)
  actKind: "all", // 내 활동 거르기 탭(#1q) — `ACT_KINDS` 의 `kind`
  actFresh: new Set(), // 내 활동을 열 때 안 본 항목 — 본 것으로 표시한 뒤 다시 그려도 강조를 남긴다
  msgs: [], // 열린 대화의 글(`msgsOf` = 그 대화 id) — 머리글 부제 · 실행 상세가 쓴다
  msgsOf: null,
  pickMsg: null, // 실행 상세에서 고른 글(요청 또는 그 답) — 없으면 마지막 요청
  focusMsg: null, // 대화를 그린 뒤 갈 글(`#c/{대화}/m/{글}`) — 늦게 그려져도 그때 간다
  mention: null, // 입력창 멘션 후보 — `{start, items, active}`(열려 있을 때만)
  mentionWarned: null, // 후보와 맞지 않는 멘션이라 한 번 막은 글 — 같은 글을 다시 보내면 멘션 없이 보낸다
  netFails: 0, // 연달아 닿지 못한 요청 수(#1j — 앱과 연결 끊김)
};
const latest = new Latest();
const previewLatest = new Latest(); // 개요 미리보기 — 늦게 온 글 읽기는 버린다

// ---- 뷰어 ----

function inlineEls(list) {
  return list.map((x) =>
    x.t === "code" ? el("code", { text: x.v }) : x.t === "b" ? el("strong", { text: x.v }) : x.t === "i" ? el("em", { text: x.v }) : x.v,
  );
}

/// markdown 블록 → 요소(정해진 태그만, 값은 textContent).
function markdownEl(text) {
  const box = el("div", { class: "md" });
  for (const b of markdownBlocks(text)) {
    if (b.type === "h") box.append(el("h" + Math.min(b.level + 1, 6), {}, ...inlineEls(b.inlines)));
    else if (b.type === "p") box.append(el("p", {}, ...inlineEls(b.inlines)));
    else if (b.type === "quote") box.append(el("blockquote", {}, ...inlineEls(b.inlines)));
    else if (b.type === "code") box.append(el("pre", {}, el("code", { text: b.text })));
    else if (b.type === "ul" || b.type === "ol") box.append(el(b.type, {}, ...b.items.map((it) => el("li", {}, ...inlineEls(it)))));
    else if (b.type === "table")
      box.append(
        el(
          "table",
          {},
          el("thead", {}, el("tr", {}, ...b.head.map((c) => el("th", {}, ...inlineEls(c))))),
          el("tbody", {}, ...b.rows.map((r) => el("tr", {}, ...r.map((c) => el("td", {}, ...inlineEls(c)))))),
        ),
      );
  }
  return box;
}

/// 본문 뷰어 — HTML 문서면 sandbox iframe(스크립트 · 폼 · 같은 출처 차단, CSP 로 외부 리소스 차단), 아니면 markdown.
function bodyEl(text) {
  if (isHtmlDoc(text)) {
    const frame = el("iframe", { class: "html-doc", sandbox: "", title: "HTML 본문(격리)", referrerpolicy: "no-referrer" });
    frame.srcdoc = htmlSrcdoc(text);
    return frame;
  }
  return markdownEl(text);
}

/// 본문 = 뷰어 + 원문 — 머리글 "원문 보기" 버튼(`#conversation.show-raw`)이 CSS 로 둘 중 하나만 보인다(다시 그리지 않음 · iframe 유지).
function bodyPair(text, cls = "") {
  const view = bodyEl(text);
  view.classList.add("body-view");
  if (cls) view.classList.add(cls);
  return [view, el("pre", { class: "raw", text })];
}

/// "대화 보기" · "원문 보기"(#1j) — 둘 중 하나가 눌린 상태.
function showRaw(on) {
  $("conversation").classList.toggle("show-raw", on);
  $("raw_btn").setAttribute("aria-pressed", String(on));
  $("view_btn").setAttribute("aria-pressed", String(!on));
}
$("raw_btn").addEventListener("click", () => showRaw(true));
$("view_btn").addEventListener("click", () => showRaw(false));

// ---- 상태 · 내 정보 · 테마 ----

async function showStatus() {
  const KEYS = ["version", "data_dir", "db"];
  try {
    const s = await api("GET", "/api/status");
    for (const k of KEYS) $(k).textContent = s[k];
  } catch (e) {
    for (const k of KEYS) $(k).textContent = "오류: " + e.message;
  }
}

async function showMe() {
  try {
    state.me = await api("GET", "/api/me");
    const me = state.me;
    $("me_name").textContent = me.name;
    $("me_host").textContent = me.host;
    $("me_fingerprint").textContent = me.fingerprint;
    $("me_addrs").textContent = me.addrs.length ? me.addrs.join(", ") : "포트 " + me.port;
    const ports = portText(me);
    $("me_ui_port").textContent = ports.ui;
    $("me_peer_port").textContent = ports.peer;
    $("me_conns").textContent = ports.conns;
    renderSideHead();
  } catch (e) {
    for (const id of ["me_name", "me_host", "me_fingerprint", "me_addrs", "me_ui_port", "me_peer_port", "me_conns", "side_name"]) $(id).textContent = "오류: " + e.message;
  }
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme === "dark" ? "dark" : "light";
}

/// 레일 타일 크기(#1x) — 작게 = 작은 아이콘 + 오른쪽 이름 한 줄(레일이 넓어진다, `app.css` `#app.tile-small`).
function applyTileSize(size) {
  $("app").classList.toggle("tile-small", size === "small");
  $("rail_resize").setAttribute("aria-valuenow", String(size === "small" ? RAIL_W.small : RAIL_W.normal));
}

/// 레일 타일 순서(#1l) — 상대 묶음(상대 타일 · +)과 그룹 묶음(그룹 타일 · 그룹 +)의 앞뒤. DOM 을 옮겨 Tab 순서도 화면과 같다(CSS `order` 는 어긋남).
function applyRailOrder(order) {
  const peers = [$("rail_pcs"), $("rail_add")];
  const groups = [$("rail_groups"), $("rail_group_add")];
  const [first, second] = order === "groups_first" ? [groups, peers] : [peers, groups];
  $("rail_me").after(...first, $("rail_sep"), ...second);
}

// ---- 피어 · 레일 ----

async function loadPeers(force = false) {
  const stamp = orderStamp("peer");
  try {
    state.peers = keepOrder("peer", await api("GET", "/api/peers"), stamp);
  } catch (e) {
    $("peer_msg").textContent = "그룹설정 오류: " + e.message;
    return;
  }
  state.peerNames = new Map(state.peers.map((p) => [p.id, `${displayName(p)} · ${p.host}`]));
  renderRail();
  renderSideHead();
  renderPeerList(force);
  fillPeerSelects();
  // 표시 이름이 바뀌었을 수 있다 — 목록 · 열린 대화 머리글도 새 이름으로.
  renderConvList();
  refreshHome();
  if (state.current && route(location.hash).view === "conversation") renderConvHead(state.current);
}

const PERSON_SVG =
  '<svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true"><circle cx="12" cy="8" r="4" fill="currentColor"></circle><path d="M4 21c0-4.4 3.6-7 8-7s8 2.6 8 7z" fill="currentColor"></path></svg>';
const GROUP_SVG =
  '<svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true"><circle cx="15.5" cy="7.5" r="3.4" fill="currentColor" opacity=".45"></circle><path d="M10 20c.3-3.8 2.8-6 5.5-6s5.3 2.2 5.5 6z" fill="currentColor" opacity=".45"></path><circle cx="9.5" cy="9" r="3.8" fill="currentColor"></circle><path d="M2.5 21.5c0-4.1 3.1-6.6 7-6.6s7 2.5 7 6.6z" fill="currentColor"></path></svg>';

/// 타일 얼굴(#1j A안) — 그림이 있으면 그림, 없으면 사람(나 · 상대) · 사람 + 그림자(그룹) 모양. 누구인지는 아래 이름(`tileName`). 그림은 고정 문자열만.
function tileFace(icon, group = false) {
  if (icon) return el("img", { src: icon, alt: "", draggable: "false" }); // 그림 기본 끌기가 레일 포인터 끌기를 가로채지 않게(#1z R2)
  const s = el("span", { class: "tile-shape", "aria-hidden": "true" });
  s.innerHTML = group ? GROUP_SVG : PERSON_SVG;
  return s;
}

/// 타일 아래 이름(#1j) — 11px · 넘치면 말줄임, 전체 이름은 단추 `title`.
function tileName(name) {
  return el("span", { class: "tile-name", "aria-hidden": "true", text: name });
}

/// 다시 그리기 전 초점이 그 안의 항목(`data-drag`)에 있었으면 다시 그린 같은 항목(같은 자리의 링크 · ⋯ · ☆)에 되돌린다(#1j — 타일을 누르면 레일이
/// 다시 그려져 초점이 body 로 떨어져 Alt + ↑ / ↓ 가 듣지 않았다).
function keepFocus(box, draw) {
  const a = document.activeElement;
  const d = box.contains(a) ? dragInfo(a) : null;
  const sub = d && a !== d.node ? ["a", ".row-more", ".star-btn"].find((s) => a.matches(s)) : null;
  draw();
  if (!d || d.node.isConnected) return;
  const n = box.querySelector(dragSel(d));
  (sub ? n?.querySelector(sub) : n)?.focus();
}

/// 타일 색 — 상대 id 로 다섯 색을 돈다(#1i — 자리를 옮겨도 색이 타일을 따라간다).
function tileColor(p) {
  return colorOf(p.id);
}

/// 레일 타일을 끄는 중(#1z R1) — 그동안 `renderRail` 은 건너뛴다(서명을 적지 않으니 끝나고 부르면 그린다).
let railDragging = false;

function renderRail() {
  if (railDragging) return;
  const paired = state.peers.filter((p) => p.paired);
  if (state.railPeer != null && !paired.some((p) => p.id === state.railPeer)) {
    state.railPeer = null;
    closeTileEdit(); // 그 상대에 묶인 편집
  }
  const unread = tileUnread(state.convs); // 상대 타일에서 보이는 대화만(#1z-2 — 에이전트 응답은 '나' 에만)
  if (state.railGroup != null && !state.groups.some((g) => g.id === state.railGroup)) state.railGroup = null;
  const sig = [
    state.railPeer,
    state.railGroup,
    state.settings.me_alias,
    state.settings.me_icon,
    paired.map((p) => [p.id, displayName(p), p.host, p.icon, unread.get(p.id) ?? 0, presence(p).text]),
    state.groups.map((g) => [g.id, g.name, g.icon, groupUnread(state.convs, g), inGroup(state.convs, g).length]),
  ];
  if (unchanged($("rail"), JSON.stringify(sig))) return;
  const me = $("rail_me");
  me.setAttribute("aria-pressed", String(state.railPeer == null && state.railGroup == null));
  me.setAttribute("aria-label", state.settings.me_alias || "나");
  me.title = state.settings.me_alias || "나";
  me.replaceChildren(tileFace(state.settings.me_icon), tileName(state.settings.me_alias || "나"));
  keepFocus($("rail"), () => drawRailTiles(paired, unread));
}

/// 상대 · 그룹 타일(#1j A안 — 모양 + 아래 이름).
function drawRailTiles(paired, unread) {
  $("rail_pcs").replaceChildren(
    ...paired.map((p) => {
      const n = unread.get(p.id) ?? 0;
      const pr = presence(p);
      return el(
        "button",
        {
          type: "button",
          class: `rail-btn ${tileColor(p)}`,
          "aria-label": `${displayName(p)} · ${p.host}${n ? ` · 안 읽음 ${n}` : ""} · ${pr.text}`,
          "aria-pressed": String(state.railPeer === p.id),
          "data-peer": String(p.id),
          "data-drag": "peer",
          "data-id": String(p.id),
          title: displayName(p),
          onpointerdown: tileGrab,
          onclick: () => pickTile(p.id),
          oncontextmenu: (ev) => tileMenuEvent(ev, { kind: "peer", id: p.id }),
        },
        tileFace(p.icon),
        tileName(displayName(p)),
        n ? el("span", { class: "rail-badge", "aria-hidden": "true", text: n > 99 ? "99+" : String(n) }) : "",
        el("span", { class: `rail-dot ${pr.key}`, "aria-hidden": "true" }),
      );
    }),
  );
  // 그룹 타일(#1i) — 상대 타일 아래 구분선 뒤, 사람 + 그림자 모양 또는 그림, 배지 = 그룹 대화의 안 읽음 합.
  $("rail_sep").hidden = false;
  $("rail_groups").replaceChildren(
    ...state.groups.map((g) => {
      const n = groupUnread(state.convs, g);
      return el(
        "button",
        {
          type: "button",
          class: `rail-btn group ${groupColor(g)}`,
          "data-group": String(g.id),
          "data-drag": "group",
          "data-id": String(g.id),
          title: `그룹 ${g.name}`,
          onpointerdown: tileGrab,
          "aria-label": `그룹 ${g.name} · 대화 ${inGroup(state.convs, g).length}${n ? ` · 안 읽음 ${n}` : ""}`,
          "aria-pressed": String(state.railGroup === g.id),
          onclick: () => pickGroup(g.id),
          oncontextmenu: (ev) => tileMenuEvent(ev, { kind: "group", id: g.id }),
        },
        tileFace(g.icon, true),
        tileName(g.name),
        n ? el("span", { class: "rail-badge", "aria-hidden": "true", text: n > 99 ? "99+" : String(n) }) : "",
      );
    }),
  );
}

/// 그룹 타일 색 — 그룹 id 로 다섯 색을 돈다.
function groupColor(g) {
  return colorOf(g.id);
}

/// 레일 그룹 읽기(#1i) — 레일 · 사이드바 · 요약을 다시 그린다.
async function loadGroups() {
  const stamp = orderStamp("group");
  try {
    state.groups = keepOrder("group", await api("GET", "/api/groups"), stamp);
  } catch (e) {
    $("status_msg").textContent = "그룹 목록 오류: " + e.message;
    return;
  }
  renderRail();
  renderSideHead();
  renderSideNav();
  renderConvList();
  refreshHome();
  renderPeerList(); // 그룹설정의 그룹 절(#1j)
}

/// 고른 그룹(없으면 `null`).
function railGroupOf() {
  return state.groups.find((g) => g.id === state.railGroup) ?? null;
}

/// 고른 레일 범위의 대화 — 상대 = 그 상대, 그룹 = 그 그룹 대화, '나' = 전부.
function scopedConvs() {
  const g = railGroupOf();
  if (g) return inGroup(state.convs, g);
  return state.railPeer == null ? state.convs : state.convs.filter((c) => c.peer_id === state.railPeer);
}

/// 그룹 타일 누름 — 열린 대화가 그 그룹 것이면 그대로, 아니면 메인 = 그룹 요약(`#`).
function pickGroup(id) {
  const changed = state.railGroup !== id;
  if (changed) closeTileEdit();
  state.railPeer = null;
  state.railGroup = id;
  renderRail();
  renderSideHead();
  const g = railGroupOf();
  const keep = ["conversation", "sent"].includes(route(location.hash).view) && state.current && inGroup([state.current], g).length > 0;
  if (!keep) state.current = null;
  renderConvList();
  if (changed) state.unseen = 0;
  renderSideNav();
  if (changed) loadCounts();
  if (!keep) go("#");
}

/// 레일 선택 — 다른 PC 를 고르면 열린 대화를 놓는다(입력창이 이전 상대를 가리킨 채 남지 않게). 화면 이동은 부른 쪽이.
/// 상대 연결 확인 — 끝나면 접속 상태를 다시 읽는다(실패는 조용히 — 표시가 "n분 전 통신" 으로 남는다).
function pingPeers(ids) {
  if (!ids.length) return;
  Promise.allSettled(ids.map((id) => api("POST", `/api/peers/${id}/ping`))).then(() => loadPeers());
}

function pickRail(id) {
  const changed = state.railPeer !== id || state.railGroup != null;
  if (changed) closeTileEdit();
  state.railPeer = id;
  state.railGroup = null;
  renderRail();
  renderSideHead();
  if (state.current && !keepsConversation(state.current, id)) state.current = null;
  renderConvList();
  if (changed) state.unseen = 0; // 새 범위의 수는 곧 다시 읽는다
  renderSideNav();
  // 그 상대 연결 확인(#1c — 주기 확인 없음, 60초 안에 통신했으면 서버가 보내지 않는다).
  if (changed && id != null) pingPeers([id]);
  if (changed) loadCounts();
}

/// 타일 누름 — 사이드바 = 그 타일의 대화 목록('나' = 상대별 절). 열린 대화가 그 타일 것이면 그대로, 아니면 메인 = 그 타일 개요('나' = 상대별
/// 요약, 상대 = 그 상대 하나의 요약 — #1j). 상대 타일을 누르면 그 PC 의 에이전트 목록은 다음에 볼 때 다시 받는다.
function pickTile(id) {
  if (id != null) state.peerAgents.delete(id);
  pickRail(id);
  if (!tileKeeps(route(location.hash).view, state.current, id)) go("#");
}

/// 해시로 이동 — 이미 그 해시면 hashchange 가 없으므로 바로 다시 그린다(빈 해시 = `#`).
function go(hash) {
  if ((location.hash || "#") === hash) render();
  else location.hash = hash;
}

function renderSideHead() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const g = railGroupOf();
  if (g) {
    $("side_name").textContent = g.name;
    $("side_sub").textContent = `그룹 · 대화 ${inGroup(state.convs, g).length} · 이 PC 에만`;
  } else if (p) {
    // 접속 상태는 레일 점(#1x) — 부제에 "페어링 됨" · 접속 문구를 쓰지 않는다.
    $("side_name").textContent = displayName(p);
    $("side_sub").textContent = sideSub("peer", p);
  } else if (state.me) {
    const alias = state.settings.me_alias;
    $("side_name").textContent = alias || "나 · " + state.me.host;
    $("side_sub").textContent = sideSub("me", state.me, alias);
  }
}

$("rail_me").addEventListener("click", () => pickTile(null));

/// 상단 바 프로필 단추(#1j) — 알약: 아이콘(없으면 사람 모양) + 표시 이름 한 번. 아이콘 테두리 · 점 = 내 상태(`presenceOf` — 받기 가능 초록 · 집중 주황 ·
/// 앱과 연결 끊김 붉음). 이름은 CSS 로 약 12자에서 말줄임, 전체는 `title`.
function renderAvatar() {
  const name = state.settings.me_alias || "나";
  const icon = state.settings.me_icon;
  const st = presenceOf(state.settings.focus, appOffline());
  const face = el("span", { class: `avatar-face st-${st.key}`, "aria-hidden": "true" });
  if (icon) face.append(el("img", { src: icon, alt: "" }));
  else face.innerHTML = PERSON_SVG;
  face.append(el("span", { class: "avatar-dot" }));
  $("avatar_btn").title = `${name} · ${st.text}`;
  $("avatar_btn").setAttribute("aria-label", `프로필 · ${st.text}`);
  $("avatar_btn").replaceChildren(face, el("span", { class: "avatar-name", text: name }));
}

/// 사이드바 내비 — 고른 타일 범위(`view.js` `sideNav`), 지금 화면은 강조. '개요' 는 대화 화면에서도. 절 사이 구분선(#1h).
function renderSideNav() {
  const view = route(location.hash).view;
  const scoped = scopedConvs();
  const starred = scoped.filter((c) => c.starred === true).length;
  // 에이전트 응답(#1v · #1z-2 — '나' 에만) = 그 범위 상대가 연 대화의 안 읽음 합 — 대화 목록에서 빠져 배지가 여기로 온다.
  const agentUnread = scoped.filter((c) => c.dir === "in").reduce((n, c) => n + (c.unread ?? 0), 0);
  if (unchanged($("side_nav"), JSON.stringify([view, state.railPeer, state.railGroup, state.unseen, state.reqWaiting, starred, agentUnread]))) return;
  $("side_nav").replaceChildren(
    ...sideNav(state.railGroup != null ? "group" : state.railPeer, state.unseen, state.reqWaiting, starred, agentUnread).map((n) => {
      if (n.sep) return el("li", { class: "nav-sep", role: "separator" });
      if (n.head) return el("li", { class: "nav-head" }, el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label })));
      const target = route(n.href).view;
      const current = target === view || (target === "home" && (view === "conversation" || view === "sent"));
      return el(
        "li",
        { class: n.sub ? "nav-sub" : null },
        el(
          "a",
          {
            href: n.href,
            class: current ? "current" : null,
            "aria-current": current ? "page" : null,
            // 요청 화면에서 메뉴 '요청' 을 다시 누르면 개요로(#1q — 해시가 같아 화면 바꿈이 없다).
            onclick: target === "requests" && current ? () => loadRequests(true) : null,
          },
          el("span", { class: "nav-label" }, ico(n.icon), el("span", { text: n.label })),
          n.count ? el("span", { class: "count", text: String(n.count) }) : "",
        ),
      );
    }),
  );
}

/// 상대 PC 의 에이전트 이름 — 원격 왕복이라 에이전트 화면 · 멘션이 없을 때만 묻는다.
async function loadPeerAgents(id) {
  state.peerAgents.set(id, undefined); // 묻는 중 — 같은 상대를 겹쳐 묻지 않는다
  try {
    state.peerAgents.set(id, (await api("GET", `/api/peers/${id}/agents`)).names);
  } catch {
    state.peerAgents.set(id, null);
  }
  if (route(location.hash).view === "agents") renderAgentsView();
}

/// 세션 생성 화면에 상대 · 에이전트를 채워서 연다(사이드바 · 검색 결과의 상대 에이전트).
async function newChat(peer, agent) {
  if (!state.peers.some((p) => p.id === peer && p.paired)) {
    $("status_msg").textContent = "페어링되지 않은 상대 · 그룹설정에서 승인하세요";
    go("#settings/peers");
    return;
  }
  state.focusAfter = "chat_title";
  go("#new");
  $("chat_peer").value = String(peer);
  await loadChatAgents(agent);
}

// ---- 타일 표시 이름 · 아이콘(이 PC 에만) ----

const ICON_PX = 80; // 타일 40px × 2(고해상도 화면)
/// 열린 타일 편집 — `kind` = `me` · `peer` · `group`(#1i), `id` = 상대 · 그룹 id, `icon` = `undefined`(그대로) · `""`(지움) · PNG data URL.
let tileEdit = null;

/// 지금 고른 타일(레일 범위).
function currentTile() {
  if (state.railGroup != null) return { kind: "group", id: state.railGroup };
  if (state.railPeer != null) return { kind: "peer", id: state.railPeer };
  return { kind: "me", id: null };
}

/// 타일의 지금 값 — 이름(편집 칸) · 빈 이름일 때 보일 이름 · 그림 · 색.
function tileNow(t) {
  if (t.kind === "group") {
    const g = state.groups.find((x) => x.id === t.id);
    return g && { value: g.name, fallback: g.name, icon: g.icon, color: groupColor(g) };
  }
  if (t.kind === "peer") {
    const p = state.peers.find((x) => x.id === t.id);
    return p && { value: p.alias ?? "", fallback: p.name, icon: p.icon, color: tileColor(p) };
  }
  return { value: state.settings.me_alias ?? "", fallback: "나", icon: state.settings.me_icon, color: "me" };
}

function openTileEdit() {
  const t = currentTile();
  const now = tileNow(t);
  if (!now) return;
  tileEdit = { ...t, icon: undefined };
  const group = t.kind === "group";
  $("tile_alias_label").textContent = group ? "그룹 이름" : "표시 이름";
  // 그룹 이름 길이는 `groupNameError`(코드 포인트 40자)로 — HTML 제한은 UTF-16 단위라 두지 않는다(라운드 1).
  if (group) $("tile_alias").removeAttribute("maxlength");
  else $("tile_alias").maxLength = 32;
  $("tile_alias").value = now.value;
  $("tile_alias").placeholder = now.fallback;
  $("tile_icon_file").value = "";
  $("tile_msg").textContent = "";
  renderTilePreview();
  $("tile_form").hidden = false;
  $("tile_alias").focus();
}

function closeTileEdit() {
  tileEdit = null;
  $("tile_form").hidden = true;
}

/// 미리보기 — 고친 이름 · 아이콘으로 그린 타일.
function renderTilePreview() {
  const now = tileNow(tileEdit);
  if (!now) return;
  const icon = tileEdit.icon !== undefined ? tileEdit.icon : now.icon;
  const name = $("tile_alias").value.trim() || now.fallback;
  const box = $("tile_preview");
  box.className = `rail-btn tile-preview ${now.color}`;
  box.replaceChildren(tileFace(icon, tileEdit.kind === "group"));
}

/// 고른 그림 → 가운데 정사각형을 잘라 `ICON_PX` 크기 PNG data URL(서버는 PNG data URL 만 받는다). 그림은 data URL 로 읽는다 — CSP
/// `img-src 'self' data:` 가 `blob:` 을 막는다(#1i 실측 — 객체 URL 이면 "cannot be decoded").
async function iconFromFile(file) {
  const url = await new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(r.result);
    r.onerror = () => reject(r.error ?? new Error("그림을 읽지 못했습니다"));
    r.readAsDataURL(file);
  });
  const img = new Image();
  img.src = url;
  await img.decode();
  const s = Math.min(img.naturalWidth, img.naturalHeight);
  if (!s) throw new Error("크기를 알 수 없는 그림");
  const canvas = el("canvas", { width: ICON_PX, height: ICON_PX });
  canvas.getContext("2d").drawImage(img, (img.naturalWidth - s) / 2, (img.naturalHeight - s) / 2, s, s, 0, 0, ICON_PX, ICON_PX);
  return canvas.toDataURL("image/png");
}

$("tile_edit_btn").addEventListener("click", () => ($("tile_form").hidden ? openTileEdit() : closeTileEdit()));
$("tile_more_btn").addEventListener("click", (ev) => openTileMenu(currentTile(), ev.currentTarget));
$("rail_me").addEventListener("contextmenu", (ev) => tileMenuEvent(ev, { kind: "me", id: null }));
// 레일 어디서든(타일 사이 빈칸 · 바탕 포함) 브라우저 메뉴는 띄우지 않는다(#1o) — 타일 위면 위 타일 메뉴만.
$("rail").addEventListener("contextmenu", (ev) => ev.preventDefault());

/// 레일 타일 우클릭 · 메뉴 키(Shift + F10 포함 — 브라우저가 같은 `contextmenu` 를 낸다) → 타일 메뉴(#1i).
function tileMenuEvent(ev, t) {
  ev.preventDefault();
  openTileMenu(t, ev.currentTarget);
}

/// 그 타일을 고른다 — 타일 편집 폼은 고른 타일의 사이드바 머리에 있다.
function selectTile(t) {
  if (t.kind === "group") {
    if (state.railGroup !== t.id) pickGroup(t.id);
  } else if (state.railPeer !== t.id || state.railGroup != null) pickTile(t.id);
}

/// 타일 메뉴 — "타일 편집"(이름 · 그림) · "이미지 지우기"(그림이 있을 때) · 상대면 그룹 체크(#1j) · 위로 · 아래로 옮기기 · 그룹이면 "그룹 삭제"(확인 한 번 —
/// 대화는 지우지 않는다).
function openTileMenu(t, anchor) {
  const now = tileNow(t);
  if (!now) return;
  const item = (icon, text, onclick, cls) => el("button", { type: "button", role: "menuitem", onclick, class: cls }, ico(icon), el("span", { text }));
  const items = [
    item("pencil", "타일 편집", () => {
      closeFloat(false);
      selectTile(t);
      openTileEdit();
    }),
  ];
  if (now.icon)
    items.push(
      item("image-off", "이미지 지우기", async () => {
        closeFloat(true);
        await act("이미지 지우기", async () => {
          if (t.kind === "me") fillPrefs(await api("PUT", "/api/settings", { me_icon: "" }));
          else if (t.kind === "group") {
            await api("PATCH", `/api/groups/${t.id}`, { icon: "" });
            await loadGroups();
          } else {
            await api("PATCH", `/api/peers/${t.id}`, { icon: "" });
            await loadPeers(true);
          }
          return "이미지를 지웠습니다";
        });
      }),
    );
  // 위로 · 아래로 옮기기(#1j) — Alt + ↑ / ↓ 와 같다. 맨 위 · 맨 아래면 그 항목은 꺼진다.
  if (t.kind !== "me") {
    items.push(el("hr", { class: "menu-sep" }));
    const order = [...$("rail").querySelectorAll(`[data-drag="${t.kind}"]`)].map((n) => Number(n.dataset.id));
    for (const [text, dir] of [
      ["위로 옮기기", -1],
      ["아래로 옮기기", 1],
    ]) {
      const b = item(dir < 0 ? "arrow-up" : "arrow-down", text, () => {
        closeFloat(false);
        stepMove({ kind: t.kind, id: t.id, section: null }, dir);
      });
      b.append(el("span", { class: "kbd", "aria-hidden": "true", text: dir < 0 ? "Alt + ↑" : "Alt + ↓" }));
      b.disabled = !stepTarget(order, t.id, dir);
      items.push(b);
    }
  }
  if (t.kind === "group")
    items.push(
      item(
        "trash",
        "그룹 삭제",
        () => {
          $("float").replaceChildren(groupDeleteForm(t.id));
          $("float").setAttribute("role", "dialog");
          $("float").querySelector("button")?.focus();
        },
        "danger-text",
      ),
    );
  openFloat(anchor, "menu", "타일 메뉴", items);
}

/// 그룹 삭제 확인(한 번) — 타일 메뉴 · 그룹설정 절 머리 공용. 그룹 · 소속만 지운다(대화 · 상대는 그대로).
function groupDeleteForm(id) {
  const g = state.groups.find((x) => x.id === id);
  return el(
    "div",
    { class: "float-form" },
    el("strong", { text: `그룹 ${g?.name ?? ""} 삭제` }),
    el("p", { class: "hint", text: "그룹만 지웁니다. 대화는 남습니다." }),
    el(
      "div",
      { class: "actions" },
      button("취소", () => closeFloat(true)),
      button(
        "삭제",
        async () => {
          closeFloat(false);
          await act("그룹 삭제", async () => {
            await api("DELETE", `/api/groups/${id}`);
            const was = state.railGroup === id;
            await loadGroups();
            if (was) pickTile(null);
            focusAfterFloat("#rail_group_add");
            return `그룹 ${g?.name ?? ""} 을 지웠습니다`;
          });
        },
        "danger",
      ),
    ),
  );
}

$("tile_cancel").addEventListener("click", closeTileEdit);
$("tile_alias").addEventListener("input", () => tileEdit && renderTilePreview());
$("tile_icon_clear").addEventListener("click", () => {
  if (!tileEdit) return;
  tileEdit.icon = "";
  $("tile_icon_file").value = "";
  renderTilePreview();
});
$("tile_icon_file").addEventListener("change", async () => {
  const file = $("tile_icon_file").files[0];
  const edit = tileEdit;
  if (!file || !edit) return;
  try {
    const icon = await iconFromFile(file);
    if (tileEdit !== edit) return; // 그 사이 닫았거나 다른 타일로 옮겼다
    edit.icon = icon;
    renderTilePreview();
    $("tile_msg").textContent = "저장하면 아이콘이 바뀝니다";
  } catch (e) {
    $("tile_msg").textContent = "아이콘을 만들지 못했습니다: " + e.message;
  }
});
$("tile_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const edit = tileEdit;
  if (!edit) return;
  const { kind, id, icon } = edit;
  const alias = $("tile_alias").value;
  if (kind === "group") {
    const bad = groupNameError(alias, state.groups, id);
    if (bad) {
      $("tile_msg").textContent = bad;
      return;
    }
  }
  act(
    "타일 저장",
    async () => {
      if (kind === "me") {
        fillPrefs(await api("PUT", "/api/settings", icon === undefined ? { me_alias: alias } : { me_alias: alias, me_icon: icon }));
      } else if (kind === "group") {
        await api("PATCH", `/api/groups/${id}`, icon === undefined ? { name: alias } : { name: alias, icon });
        await loadGroups();
      } else {
        await api("PATCH", `/api/peers/${id}`, icon === undefined ? { alias } : { alias, icon });
        await loadPeers(true);
        await refresh(); // 열린 대화 · 목록의 발화자 이름도 새 표시 이름으로
      }
      if (tileEdit === edit) closeTileEdit(); // 그 사이 다른 타일 편집을 열었으면 그대로
    },
    "tile_msg",
  );
});
/// 설정 화면의 한 절로(`#settings/{절}`) — 그린 뒤 `focus` 칸에 초점.
function goSettings(id, focus = null) {
  state.focusAfter = focus;
  go(`#settings/${id}`);
}

// 레일 + (#1k) — 새로 만들기 "초대 수락" 과 같게 초대 수락 칸을 펼쳐 바로 붙여 넣게(#1r — 칸은 라우터가 펼친다).
$("rail_add").addEventListener("click", () => goSettings("invite_accept", "invite_text"));

/// 페어링 카드 "초대 수락"(#1k) — 붙여 넣기 칸을 펼치고 접는다.
function showAccept(on) {
  $("invite_accept").hidden = !on;
  $("accept_btn").setAttribute("aria-expanded", String(on));
}
$("accept_btn").addEventListener("click", () => {
  const on = $("invite_accept").hidden;
  showAccept(on);
  if (on) $("invite_text").focus({ preventScroll: true });
});
$("rail_settings").addEventListener("click", () => go("#settings"));

/// 관리 줄(#1k — 디자인 캔버스 ④) — 머리(타일 · 이름 · 부제 · 단추) + 연필 단추("{이름} 관리", #1r)로 여닫는 상세. 펼침은 `state.open`(다시 그려도 유지).
function manageItem(key, label, head, ...body) {
  const open = state.open.has(key);
  const detail = el("div", { class: "item-body", id: `detail_${key}`, hidden: !open }, ...body);
  // 연필 단추(#1r — 바꾸기 · 고치기는 연필 하나로).
  const btn = el("button", {
    type: "button",
    id: `${key}_manage`,
    class: "icon-btn manage-btn",
    title: `${label} 관리`,
    "aria-expanded": String(open),
    "aria-controls": detail.id,
    "aria-label": `${label} 관리`,
    onclick: () => {
      const on = detail.hidden;
      detail.hidden = !on;
      btn.setAttribute("aria-expanded", String(on));
      if (on) state.open.add(key);
      else state.open.delete(key);
    },
  }, ico("pencil"));
  return el("div", { class: "manage-item" }, el("div", { class: "manage-head" }, ...head, btn), detail);
}

/// 줄 타일(#1k) — 상대 · 에이전트 사람 모양(그림이 있으면 그림), 그룹은 사람 + 그림자.
function rowTile(cls, icon, group = false) {
  return el("span", { class: `pa ${cls}`, "aria-hidden": "true" }, tileFace(icon, group));
}

function kvTable(rows) {
  return el("table", { class: "kv" }, el("tbody", {}, ...rows.map(([k, v, cls]) => el("tr", {}, el("th", { scope: "row", text: k }), el("td", {}, el("span", { class: cls, text: v }))))));
}

let uid = 0;
/// 라벨 + 컨트롤 한 쌍(id 자동) — `box` 는 컨트롤을 감싼 칸.
function field(label, control, box = control) {
  control.id ||= "f" + ++uid;
  return [el("label", { for: control.id, text: label }), box];
}

/// 같은 내용이면 다시 그리지 않는다 — 5초 새로 고침 · 이벤트가 레일 · 내비 · 대화 목록의 초점과 누르는 중인 클릭을 빼앗지 않게.
/// 서명에 그리는 데 쓰는 값을 모두 넣는다(다르면 서명을 새로 적고 그린다).
function unchanged(box, sig) {
  if (box.dataset.sig === sig) return true;
  box.dataset.sig = sig;
  return false;
}

/// 목록을 다시 그려도 되나 — 같은 값이면 건너뛰고, 사용자가 그 안 입력칸에서 입력 중이면(배경 새로 고침) 미룬다. 저장 · 필터는 `force`.
/// 입력칸만 본다 — 상대별 절 머리(`summary`)를 눌러 초점이 남아도 목록은 계속 새로 고친다(#1g).
function needsRender(box, sig, force) {
  const typing = box.contains(document.activeElement) && holdsRender(document.activeElement.tagName);
  if (!force && (box.dataset.sig === sig || typing)) return false;
  box.dataset.sig = sig;
  return true;
}

/// 연결 줄 부제(#1k) — `{이름} · {PC}` · 접속 상태(페어링됨이면 "방금 · n분 전 통신", 아니면 연결 단계).
function peerSub(p) {
  return [p.alias ? `${p.name} · ${p.host}` : p.host, p.paired ? presence(p).text : peerState(p)].join(" · ");
}

/// 그룹설정 대화 줄을 끄는 중(#1z-2) — 그동안 `renderPeerList` 는 건너뛴다(캡처한 줄이 DOM 에서 빠지지 않게, 레일 타일 끌기와 같은 이유).
let gsDragging = false;

function renderPeerList(force = false) {
  // 접속 상태(`online` · `focus`)는 이 목록에 그리지 않는다 — 30초 Ping 이 저장 전 입력을 지우지 않게 서명에서 뺀다.
  const shown = state.peers.map(({ online, focus, ...p }) => p);
  const groupSig = state.groups.map((g) => [g.id, g.name, g.icon, g.convs]);
  const convSig = state.convs.map((c) => [c.id, c.peer_id, c.pos, c.title, c.agent, c.dir, convClosed(c)]);
  const sig = JSON.stringify([shown, groupSig, convSig, $("peer_q").value, state.settings.claude_model, state.settings.claude_effort]);
  // 접속 상태 글자 · 점은 시각이 흐르면 바뀐다(#1k 라운드 2 · #1z R2) — 줄을 다시 만들지 않고(편집 중인 모델 · effort 를 지키게) 그것만 고친다.
  for (const n of $("peer_groups").querySelectorAll("[data-peer-sub]")) {
    const p = state.peers.find((x) => x.id === Number(n.dataset.peerSub));
    if (p) n.textContent = peerSub(p);
  }
  for (const n of $("peer_groups").querySelectorAll("[data-peer-dot]")) {
    const p = state.peers.find((x) => x.id === Number(n.dataset.peerDot));
    if (p) n.replaceWith(gsDot(p));
  }
  if (gsDragging || !needsRender($("peer_groups"), sig, force)) return;
  const q = $("peer_q").value;
  const convs = sortByPos(state.convs);
  // 그룹설정(#1z-2 — 시안 B): 위 = 그룹 카드 격자(누르면 카드 안 대화 전체), 아래 = 사용자 표(줄을 누르면 아래로 관리 · 대화). 대화 줄을 카드에 끌어 놓으면 넣기(복사).
  const list = groupSettingsList(state.groups, state.peers, convs, q, state.peerNames);
  const secs = settingsGroupSections(state.groups, convs, state.peerNames);
  $("peer_groups").replaceChildren(gsGroupsEl(list, secs, q), gsUsersEl(list, convs, q));
}

/// 그룹설정 사용자 접속 점 · 글자(#1z) — 서명에 접속 상태가 없어(입력 중 다시 그리지 않게) `renderPeerList` 가 매번 이것만 고친다(R2 — 시간이 흘러 끊김이 돼도 녹색이 남지 않게).
function gsDot(p, pr = presence(p)) {
  const text = p.paired ? pr.text : peerState(p);
  return el("span", { class: "gs-state", "data-peer-dot": String(p.id) }, el("span", { class: `gs-dot ${pr.key}`, "aria-hidden": "true" }), el("span", { text }));
}

/// 펼침 단추(#1z-2 — 그룹 카드 · 사용자 줄) — 누르면 `body` 를 보이고 숨기고 `state.open` 에 기억한다(5초 새로 고침으로 다시 그려도 그대로).
/// 같은 열쇠의 접힘 때만 보이는 요소(`data-gs-shut`)는 반대로.
function gsToggle(key, body, label, ...face) {
  const open = state.open.has(key);
  body.hidden = !open;
  const b = el(
    "button",
    {
      type: "button",
      class: "gs-toggle",
      "aria-expanded": String(open),
      "aria-controls": body.id,
      title: label,
      onclick: (ev) => {
        ev.stopPropagation();
        const on = body.hidden;
        body.hidden = !on;
        b.setAttribute("aria-expanded", String(on));
        if (on) state.open.add(key);
        else state.open.delete(key);
        for (const x of $("peer_groups").querySelectorAll(`[data-gs-shut="${key}"]`)) x.hidden = on;
      },
    },
    ...face,
  );
  return b;
}

/// 그룹 카드 격자(#1z-2 — 시안 B) — 카드 = 타일 · 이름 · 대화 수 · 이름 바꾸기 · 삭제 + 대화 3개 · "+n 더 보기"(접힘) / 대화 전체 · 그룹에서 빼기(펼침).
/// 카드가 대화 줄을 놓는 자리(`data-drag="group"` — `convGrab` 넣기).
function gsGroupsEl(list, secs, q) {
  const none = q ? "맞는 대화 없음" : "대화 없음 — 끌어 놓으세요";
  const cards = list.groups.map((g) => {
    const s = secs.find((x) => x.group === g.id);
    const rows = (s?.rows ?? []).filter((r) => g.convs.includes(r.id)); // 검색 중이면 맞는 대화(`groupSettingsList`)
    const key = `gsg${g.id}`;
    const full = el(
      "div",
      { class: "gs-card-full", id: `gs_card_${g.id}` },
      rows.length
        ? el("ol", { class: "rows settings-convs" }, ...rows.map((r) => settingsConvRow(state.convs.find((c) => c.id === r.id), r, g.id)))
        : el("p", { class: "hint", text: none }),
    );
    const toggle = gsToggle(
      key,
      full,
      `그룹 ${g.name} 대화 펼치기`,
      rowTile(`small ${colorOf(g.id)}`, g.icon, true),
      el("strong", { class: "gs-card-name", text: g.name }),
      el("span", { class: "hint", text: `대화 ${rows.length}` }),
    );
    const peek = el(
      "div",
      { class: "gs-card-peek", "data-gs-shut": key, hidden: state.open.has(key) },
      rows.length ? el("ul", { class: "gs-peek" }, ...rows.slice(0, 3).map((r) => el("li", { text: `· ${r.title}` }))) : el("p", { class: "hint", text: none }),
      rows.length > 3 ? el("button", { type: "button", class: "link-btn", text: `+${rows.length - 3} 더 보기`, onclick: () => toggle.click() }) : "",
    );
    return el(
      "article",
      { class: "gs-card", "data-drag": "group", "data-id": String(g.id), "aria-label": `그룹 ${g.name}` },
      el(
        "div",
        { class: "gs-card-head" },
        toggle,
        svgButton(PENCIL_SVG, `그룹 ${g.name} 이름 바꾸기`, (ev) => openGroupRename(g.id, ev.currentTarget)),
        svgButton(TRASH_SVG, `그룹 ${g.name} 삭제`, (ev) => openFloat(ev.currentTarget, "dialog", "그룹 삭제", [groupDeleteForm(g.id)]), "danger-text"),
      ),
      peek,
      full,
    );
  });
  return el(
    "section",
    { class: "gs-block", "aria-label": "그룹" },
    el("div", { class: "gs-block-head" }, el("strong", { text: "그룹" }), el("span", { class: "hint", text: `${list.groups.length} · 대화를 카드에 끌어 놓으면 넣기` })),
    cards.length ? el("div", { class: "gs-cards" }, ...cards) : el("p", { class: "hint", text: q ? "맞는 그룹 없음" : "그룹 없음 — + 그룹 추가" }),
  );
}

/// 사용자 표(#1z-2 — 시안 B) — 사용자 · PC · 상태 · 모델 · effort · 대화 · 관리, 줄(또는 이름 · 연필)을 누르면 바로 아래 줄에 관리 + 그 사용자 대화(끌어 그룹 카드에 넣기).
function gsUsersEl(list, convs, q) {
  const rows = list.users.flatMap((u) => {
    const p = state.peers.find((x) => x.id === u.id);
    const more = el("tr", { class: "gs-more", id: `gs_user_${p.id}` }, el("td", { colspan: "7" }, userDetail(p, convs)));
    const toggle = gsToggle(`gsu${p.id}`, more, `${displayName(p)} 설정 · 대화 펼치기`, rowTile(`small ${tileColor(p)}`, p.icon), el("span", { text: displayName(p) }));
    const tr = el(
      "tr",
      { class: "gs-user", onclick: (ev) => !ev.target.closest("button, a, select, input") && toggle.click() },
      el("td", {}, toggle),
      el("td", { class: "hint", text: p.host }),
      el("td", {}, gsDot(p)),
      el("td", { class: "hint", text: p.model || "기본값" }),
      el("td", { class: "hint", text: p.effort || "기본값" }),
      el("td", { text: String(u.count) }),
      el("td", {}, svgButton(PENCIL_SVG, `${displayName(p)} 관리`, () => toggle.click())),
    );
    return [tr, more];
  });
  return el(
    "section",
    { class: "gs-block", "aria-label": "사용자" },
    el("div", { class: "gs-block-head" }, el("strong", { text: "사용자" }), el("span", { class: "hint", text: `${list.users.length} · 줄을 누르면 아래로 펼쳐 설정 · 대화` })),
    rows.length
      ? el(
          "div",
          { class: "gs-table-box" },
          el(
            "table",
            { class: "gs-table" },
            el(
              "thead",
              {},
              el("tr", {}, ...["사용자", "PC", "상태", "모델", "effort", "대화"].map((h) => el("th", { scope: "col", text: h })), el("th", { scope: "col" }, el("span", { class: "sr-only", text: "관리" }))),
            ),
            el("tbody", {}, ...rows),
          ),
        )
      : el("p", { class: "hint", text: q ? "맞는 사용자 없음" : "연결 없음" }),
  );
}

/// 설정 대화 줄(#1w · #1z-2) — 끌기 손잡이(포인터 — 그룹 카드에 놓으면 넣기) · (그룹 카드 안이면) 상대 타일 · 제목 · 부제 · 닫힌 대화 태그 · (그룹 카드 안이면) "그룹에서 빼기".
function settingsConvRow(c, label, group) {
  const p = state.peers.find((x) => x.id === c.peer_id);
  const closed = convClosed(c);
  return el(
    "li",
    { class: "settings-conv", "data-drag": "conv", "data-id": String(c.id) },
    el(
      "button",
      {
        type: "button",
        class: "drag-handle",
        title: "끌어서 그룹 카드에 놓으면 그룹에",
        "aria-label": `${label.title} 끌어 그룹에 넣기`,
        onpointerdown: (ev) => convGrab(ev, ev.currentTarget.parentElement, { join: true }),
      },
      ico("grip"),
    ),
    group == null ? "" : rowTile(`small ${colorOf(c.peer_id)}`, p?.icon),
    el("span", { class: "manage-name" }, el("strong", { text: label.title }), el("span", { class: "hint", text: label.sub })),
    closed ? el("span", { class: "chip", text: closed }) : "",
    group == null
      ? ""
      : el("button", {
          type: "button",
          class: "small",
          text: "그룹에서 빼기",
          "aria-label": `${label.title} 그룹에서 빼기`,
          onclick: async () => {
            const g = state.groups.find((x) => x.id === group);
            if (await changeGroup(group, [], [c.id])) $("peer_msg").textContent = `그룹 ${g?.name ?? ""} 에서 뺐습니다`;
          },
        }),
  );
}

/// 그룹설정 사용자 펼침의 저장 전 모델 · effort(#1z-2 R1) — 여러 사용자를 펼친 채 한 사람을 저장하면 목록을 다시 그리므로, 다른 줄의 고친 값을 상대 id 별로 들고 있다가 되살린다.
const gsDrafts = new Map();

/// 그룹설정 사용자 펼침(#1z-2) — 부제 · 지문 · 주소 · 모델 · effort · 단추(옛 관리 줄의 상세) + "대화 n" 줄(끌어 그룹 카드에 넣기).
function userDetail(p, convs) {
  const peerAct = (label, fn) => act(label, fn, "peer_msg", () => loadPeers(true));
  const defaults = state.settings;
  const draft = gsDrafts.get(p.id);
  const model = el(
    "select",
    {},
    el("option", { value: "", text: `기본값(${defaults.claude_model ?? "sonnet"})` }),
    ...MODELS.map((m) => el("option", { value: m, text: m })),
  );
  model.value = draft?.model ?? p.model ?? "";
  const effort = el(
    "select",
    {},
    el("option", { value: "", text: `기본값(${defaults.claude_effort ?? "high"})` }),
    ...EFFORTS.map((e) => el("option", { value: e, text: e })),
  );
  effort.value = draft?.effort ?? p.effort ?? "";
  const keep = () => gsDrafts.set(p.id, { model: model.value, effort: effort.value });
  model.addEventListener("change", keep);
  effort.addEventListener("change", keep);
  const actions = el("div", { class: "actions" });
  actions.append(
    button(
      "저장",
      () =>
        peerAct("연결 설정 저장", async () => {
          const sent = gsDrafts.get(p.id); // 보낸 때의 고친 값 — 응답을 기다리는 사이 또 고치면 새 객체라 남는다(#1z-2 R2)
          await api("PATCH", `/api/peers/${p.id}`, { model: model.value, effort: effort.value });
          if (gsDrafts.get(p.id) === sent) gsDrafts.delete(p.id); // 저장한 값이 서버 값 — 실패면 고친 값을 그대로 둔다
        }),
      "primary",
    ),
  );
  if (!p.my_ok) actions.append(button("승인", () => peerAct("승인", () => api("POST", `/api/peers/${p.id}/approve`))));
  if (p.my_ok && !p.paired) actions.append(button("다시 알리기", () => peerAct("승인 재통지", () => api("POST", `/api/peers/${p.id}/approve`))));
  if (p.paired) actions.append(button("연결 확인", () => peerAct("연결 확인", () => api("POST", `/api/peers/${p.id}/ping`))));
  actions.append(
    button("내 에이전트 편집", () => goSettings("agents")),
    button(
      "삭제",
      () =>
        peerAct("삭제", async () => {
          const r = await api("DELETE", `/api/peers/${p.id}`);
          if (!r.notified) throw new Error("삭제했지만 상대에게 알리지 못했습니다. 상대 목록에는 남아 있습니다.");
        }),
      "danger",
    ),
  );
  const mine = convs.filter((c) => c.peer_id === p.id);
  return el(
    "div",
    { class: "gs-user-detail" },
    el("p", { class: "hint", "data-peer-sub": String(p.id), text: peerSub(p) }),
    kvTable([
      ["지문", p.fingerprint, "mono"],
      ["주소", p.addr, "mono"],
    ]),
    el("div", { class: "form-grid" }, ...field("모델", model), ...field("effort", effort)),
    actions,
    el("strong", { class: "gs-convs-head", text: `대화 ${mine.length}` }),
    mine.length
      ? el("ol", { class: "rows settings-convs" }, ...mine.map((c) => settingsConvRow(c, convLabel(c, state.peerNames), null)))
      : el("p", { class: "hint", text: "대화 없음" }),
  );
}

$("peer_q").addEventListener("input", () => renderPeerList(true));
$("peer_group_add").addEventListener("click", (ev) =>
  openGroupNew(ev.currentTarget, (g) => {
    state.open.add(`gsg${g.id}`); // 만든 그룹 카드를 펼친다
    $("peer_msg").textContent = `그룹 ${g.name} 을 만들었습니다`;
  }),
);

/// 설정 대화 줄을 그룹 카드에 넣기(#1w · #1z-2) — 복사(원래 자리 그대로), 이미 있으면 안내만.
async function gsJoin(to, id) {
  const op = convCopy(id, to, state.groups);
  const g = state.groups.find((x) => x.id === to);
  if (!op) {
    $("peer_msg").textContent = `이미 그룹 ${g?.name ?? ""} 에 있습니다`;
    return;
  }
  if (await changeGroup(to, op.add, [])) $("peer_msg").textContent = `그룹 ${g?.name ?? ""} 에 넣었습니다`;
}

// 초대 생성 — 한 번에 하나(단추 · 트리 · 새로 만들기 모두 이 단추를 누른다). 도는 동안 온 누름은 버려 겹친 요청이 서버 코드를 바꿔
// 화면 초대와 어긋나는 일이 없다(적대적 평가 R1). `disabled` 가 아니라 `aria-disabled` — 새로 만들기가 이 단추에 주는 초점을 잃지 않게(R2).
let inviteBusy = false;
$("window_btn").addEventListener("click", () => {
  if (inviteBusy) return;
  inviteBusy = true;
  $("window_btn").setAttribute("aria-disabled", "true");
  act(
    "초대 생성",
    async () => {
      try {
        const r = await api("POST", "/api/pairing-window");
        $("invite").textContent = r.invite;
        $("invite_box").hidden = false;
        return `초대를 만들었습니다. ${r.seconds / 60}분 안에 들어온 첫 요청 1건은 승인 없이 등록됩니다.`;
      } finally {
        inviteBusy = false;
        $("window_btn").removeAttribute("aria-disabled");
      }
    },
    "invite_msg",
    loadPeers,
  );
});

$("copy_btn").addEventListener("click", () =>
  act(
    "초대 복사",
    async () => {
      await navigator.clipboard.writeText($("invite").textContent);
      return "초대를 복사했습니다. 상대에게 보내세요.";
    },
    "invite_msg",
  ),
);

$("invite_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const invite = $("invite_text").value;
  act(
    "초대 수락",
    async () => {
      await api("POST", "/api/peers", { invite });
      $("invite_text").value = "";
      return "연결했습니다";
    },
    "invite_msg",
    loadPeers,
  );
});

// ---- 내 에이전트 ----

async function loadAgents() {
  try {
    state.agents = await api("GET", "/api/agents");
  } catch (e) {
    $("agent_msg").textContent = "에이전트 목록 오류: " + e.message;
    return;
  }
  // 세션 생성의 "내 응답 에이전트" 후보.
  const relaySel = $("chat_relay_agent");
  const keep = relaySel.value;
  relaySel.replaceChildren(
    el("option", { value: "", text: "안 함" }),
    ...state.agents.map((a) => el("option", { value: a.name, text: a.name })),
  );
  if (state.agents.some((a) => a.name === keep)) relaySel.value = keep;
  syncRelayCap();
  renderAgentList(true);
  if (route(location.hash).view === "agents") renderAgentsView();
}

function renderAgentList(force = false) {
  const sig = JSON.stringify(state.agents);
  if (!needsRender($("agent_groups"), sig, force)) return;
  // 평평한 목록(#1j — 에이전트 그룹 칸 · 묶기 없음, 옛 값 `agents.grp` 는 서버에 그대로).
  const shown = state.agents;
  $("agent_groups").replaceChildren(
    ...shown.map((a) => {
      const folder = el("input", { value: a.folder, class: "mono" });
      const parent = el("input", { type: "checkbox", checked: a.parent_read, title: PARENT_NOTE });
      const prompt = el("textarea", { rows: "3" });
      prompt.value = a.prompt;
      const promptBox = el("div", { class: "field" }, prompt);
      return manageItem(
        "agent" + a.id,
        a.name,
        [rowTile("agent-tile", null), el("span", { class: "manage-name" }, el("strong", { text: a.name }), el("span", { class: "hint mono", text: a.folder }))],
        kvTable([["이름", a.name]]),
        el(
          "div",
          { class: "form-grid" },
          ...field("폴더(절대 경로)", folder),
          ...field("상위 한단계 탐색 허용", parent),
          ...(a.parent_read
            ? [el("span", { class: "label hint", text: "읽는 위 폴더" }), el("span", { class: "hint mono", text: a.parent_dir ?? "열지 않음(드라이브 루트 · 사용자 홈 · 데이터 폴더를 품음)" })]
            : []),
          el("span", { class: "label hint", text: "도구" }),
          el("span", { class: "hint", text: TOOLS_NOTE }),
          ...field("추가 지시", prompt, promptBox),
        ),
        el(
          "div",
          { class: "actions" },
          button("저장", () => act("에이전트 저장", () => api("PATCH", `/api/agents/${a.id}`, { folder: folder.value, prompt: prompt.value, parent_read: parent.checked }), "agent_msg", loadAgents), "primary"),
          button("삭제", () => act("삭제", () => api("DELETE", `/api/agents/${a.id}`), "agent_msg", loadAgents), "danger"),
        ),
      );
    }),
  );
  if (!shown.length) $("agent_groups").append(el("p", { class: "hint", text: "등록된 에이전트 없음" }));
}

/// "에이전트 등록"(#1l) — 목록 끝 '새 에이전트' 줄(#1r)을 보이고 숨긴다(보이면 이름 칸에 초점, 등록 · 취소면 닫는다).
function showAgentNew(on) {
  $("agent_new").hidden = !on;
  $("agent_new_btn").setAttribute("aria-expanded", String(on));
}
$("agent_new_btn").addEventListener("click", () => {
  const on = $("agent_new").hidden;
  showAgentNew(on);
  if (on) $("agent_name").focus();
});

$("agent_new_cancel").addEventListener("click", () => {
  showAgentNew(false);
  $("agent_new_btn").focus();
});

$("agent_tools").textContent = TOOLS_NOTE;

$("agent_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = {
    name: $("agent_name").value,
    folder: $("agent_folder").value,
    prompt: $("agent_prompt").value,
    parent_read: $("agent_parent").checked,
  };
  act(
    "등록",
    async () => {
      await api("POST", "/api/agents", body);
      $("agent_form").reset();
      showAgentNew(false);
      $("agent_new_btn").focus(); // 숨긴 칸에 초점이 남지 않게(적대적 평가 R1)
    },
    "agent_msg",
    loadAgents,
  );
});

// ---- 대화 목록 ----

async function loadConvs() {
  const stamp = orderStamp("conv");
  try {
    state.convs = keepOrder("conv", await api("GET", "/api/conversations"), stamp);
  } catch (e) {
    $("status_msg").textContent = "대화 목록 오류: " + e.message;
    return;
  }
  if (state.current) state.current = state.convs.find((c) => c.id === state.current.id) ?? state.current;
  renderConvList();
  renderRail();
  refreshHome();
  renderSideNav(); // 즐겨찾기 수 · 에이전트 응답 안 읽음
  renderPeerList(); // 설정 그룹설정의 대화 줄(#1w)
  if (route(location.hash).view === "starred") renderStarred();
  if (route(location.hash).view === "agentConvs") renderAgentConvs();
}

const STAR_SVG =
  '<svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor" stroke="currentColor" stroke-width="1.8" stroke-linejoin="round" aria-hidden="true"><path d="M12 3l2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"></path></svg>';

/// 즐겨찾기 ☆ 단추(#1j) — 대화방 줄 · 대화 머리 같은 모양(빠짐 = 회색 테두리 · 넣음 = 노랑). 누르면 넣고 빼고 목록을 다시 읽는다.
function starButton(c, label) {
  const on = c.starred === true;
  const b = el("button", {
    type: "button",
    class: on ? "star-btn on" : "star-btn",
    "aria-pressed": String(on),
    title: on ? "즐겨찾기 빼기" : "즐겨찾기",
    "aria-label": `${on ? "즐겨찾기 빼기" : "즐겨찾기"} · ${label}`,
    onclick: () => toggleStar(c),
  });
  b.innerHTML = STAR_SVG;
  return b;
}

async function toggleStar(c) {
  const on = c.starred !== true;
  await act(on ? "즐겨찾기" : "즐겨찾기 빼기", async () => {
    await api("PATCH", `/api/conversations/${c.id}`, { starred: on });
    c.starred = on;
    if (state.current?.id === c.id) state.current.starred = on;
    await loadConvs();
    if (state.current?.id === c.id) renderConvHead(state.current);
    return on ? "즐겨찾기에 넣었습니다" : "즐겨찾기에서 뺐습니다";
  });
}

/// 대화방 줄(#1j) — ☆ · 아바타(에이전트 첫 글자, 상대 색) · 제목 · 마지막 글 미리보기 · 시각 · 안 읽음 배지 · 실행 중 · 자동 왕복 칩. 상대 이름은 툴팁.
/// 발화자 종류 아이콘(#1z — 사람 ↔ 에이전트) — 대화 줄 미리보기 · 메시지 줄 앞(`view.js` `speakerIcon`).
function speakerEl(author) {
  const s = speakerIcon(author);
  return el("span", { class: `speaker ${s.kind}`, title: s.label, "aria-label": s.label, role: "img" }, ico(s.icon));
}

function convItem({ c, label }, section) {
  const meta = convMeta(c);
  const cls = [state.current?.id === c.id ? "current" : "", meta.unread ? "unread" : ""].filter(Boolean).join(" ");
  return el(
    "li",
    { class: cls || null, "data-conv": String(c.id), "data-drag": "conv", "data-id": String(c.id), "data-section": section },
    // 손잡이 끌기(#1x) — 즐겨찾기 ☆ 는 대화 머리에서. 줄 링크는 HTML 끌기를 막는다(브라우저 끌기 확장).
    el("button", {
      type: "button",
      class: "drag-handle",
      title: "끌어서 옮기기 · 그룹 타일에 놓으면 그룹에",
      "aria-label": `${label.title} 옮기기`,
      onpointerdown: (ev) => convGrab(ev, ev.currentTarget.parentElement),
    }, ico("grip")),
    el(
      "a",
      { href: `#c/${c.id}`, title: label.sub, draggable: "false" },
      // 한 줄(#1z-2 — 사용자 "대화방 제목만 표시하고 2행은 제거"): 로봇 아이콘(모든 줄 같은 AI 에이전트 아이콘) · 제목 · 실행 중 점 · 안 읽음 수 · 시각.
      el("span", { class: "room-av agent-av", "aria-hidden": "true" }, ico("bot")),
      el(
        "span",
        { class: "room-text" },
        el(
          "span",
          { class: "title" },
          el("span", { class: "room-title", text: label.title }),
          meta.running ? el("span", { class: "spin", role: "img", "aria-label": meta.state, title: meta.state }) : "",
          meta.unread ? el("span", { class: "count unread-n", "aria-label": `안 읽음 ${c.unread}`, text: String(c.unread) }) : "",
          c.last_at != null ? el("span", { class: "room-time", text: formatTime(c.last_at) }) : "",
        ),
      ),
    ),
    // 행 ⋯ — 마우스를 올리거나 초점이 오면 보인다(#1i).
    el("button", {
      type: "button",
      class: "icon-btn row-more",
      title: "대화 메뉴",
      "aria-label": `대화 메뉴 · ${label.title}`,
      "aria-haspopup": "menu",
      onclick: (ev) => openConvMenu(c.id, ev.currentTarget),
    }, ico("more")),
  );
}

function renderConvList() {
  const q = $("conv_search").value.trim();
  const group = railGroupOf();
  const rows = visibleConvRows(convRows(sortByPos(scopedConvs()), state.peerNames, null, q), group);
  const sections = convSections(rows, state.railPeer == null ? pairedIds() : null, state.peerNames);
  const sig = [q, state.railPeer, group && [group.id, group.convs, group.peers], state.groups.length, state.current?.id, rows, sections.map((s) => s.title)];
  if (unchanged($("conv_sections"), JSON.stringify(sig))) return;
  const empty = el("li", { class: "hint", text: q ? "맞는 대화 없음" : "대화 없음" });
  const add = (peer) =>
    el("a", {
      href: "#new",
      class: "side-add",
      title: SESSION_NEW,
      "aria-label": SESSION_NEW,
      text: "+",
      onclick:
        peer == null
          ? null
          : (ev) => {
              ev.preventDefault();
              newChat(peer);
            },
    });
  // 그룹(#1i) — 목록 머리에 "대화 추가"(검색 + 체크로 여러 대화를 한 번에).
  const head = group
    ? [
        el(
          "div",
          { class: "side-title group-head" },
          el("span", { text: `그룹 대화 · ${inGroup(state.convs, group).length}` }),
          el("button", { type: "button", id: "group_pick_btn", class: "small", text: "대화 추가", onclick: (ev) => openGroupPicker(ev.currentTarget) }),
        ),
      ]
    : [];
  const groupEmpty = el("li", { class: "hint", text: q ? "맞는 대화 없음" : "대화 없음" });
  keepFocus($("conv_sections"), () =>
    $("conv_sections").replaceChildren(
      ...head,
      ...sections.map((s) => {
        // 묶음(#1o · #1v) — 나 · 상대 타일 = 내 대화(머리 없음) · 공유 대화(머리), 에이전트 대화는 '나' 의 사이드바 '에이전트 응답' 화면(#1z-2). 그룹 타일은 묶음마다 머리.
        const parts = convListParts(s.rows, group);
        const list = (g) => el("ul", { class: "side-list" }, ...g.rows.map((r) => convItem(r, splitKey(s.key, g.dir))));
        const lists = parts.length
          ? parts.flatMap((g) =>
              g.head ? [el("div", { class: "dir-head" }, el("span", { text: g.title }), el("b", { text: String(g.rows.length) })), list(g)] : [list(g)],
            )
          : [el("ul", { class: "side-list" }, ...(rows.length ? [] : [group ? groupEmpty : empty]))];
        // '나' 의 상대별 절(#1g) — 접을 수 있고 접힘은 이 PC 에 남는다.
        if (s.peer != null) return foldEl(`convs:${s.peer}`, [el("span", { text: s.title }), add(s.peer)], lists, "side-section", "side-title");
        return el("section", { class: "side-section" }, el("h2", { class: "side-title" }, el("span", { text: s.title }), s.key === "convs" ? add(null) : ""), ...lists);
      }),
    ),
  );
}

/// 즐겨찾기 화면(#1j) — 고른 타일 범위의 ★ 대화방, 대화방 줄 그대로('나' · 그룹은 상대별 절). ☆ 를 끄면 다음 그리기에서 빠진다.
function renderStarred() {
  const sections = starredSections(sortByPos(scopedConvs()), state.peerNames, state.railPeer == null ? pairedIds() : null);
  const sig = JSON.stringify([state.current?.id, sections.map((s) => [s.key, s.title, s.rows.map((r) => [r.c, r.label])])]);
  if (!needsRender($("starred_list"), sig, false)) return;
  $("starred_list").replaceChildren(
    ...(sections.length
      ? sections.map((s) =>
          el(
            "section",
            { class: "side-section" },
            el("h2", { class: "side-title" }, el("span", { text: s.title })),
            el("ul", { class: "side-list" }, ...s.rows.map((r) => convItem(r, `starred:${s.key}`))),
          ),
        )
      : [el("p", { class: "empty hint", text: "즐겨찾기 없음" })]),
  );
}

/// 에이전트 응답 화면(#1v · #1z-2 — 옛 이름 에이전트 대화, '나' 에만) — 고른 타일 범위의 상대가 연 대화(내 에이전트가 답)를 요청한 상대별 절로, 대화 줄 그대로(끌어 옮기기는 같은 절 안).
function renderAgentConvs() {
  const paired = state.railPeer == null ? pairedIds() : [state.railPeer];
  const sections = agentConvSections(sortByPos(scopedConvs()), paired, state.peerNames);
  const sig = JSON.stringify([state.current?.id, sections.map((s) => [s.key, s.title, s.rows.map((r) => [r.c, r.label])])]);
  if (!needsRender($("agent_convs_list"), sig, false)) return;
  $("agent_convs_list").replaceChildren(
    ...(sections.length
      ? sections.map((s) =>
          el(
            "section",
            { class: "side-section" },
            el("h2", { class: "side-title" }, el("span", { text: s.title }), el("b", { class: "count", text: String(s.rows.length) })),
            el("ul", { class: "side-list" }, ...s.rows.map((r) => convItem(r, s.key))),
          ),
        )
      : [el("p", { class: "empty hint", text: "에이전트 응답 없음" })]),
  );
}

// ---- 상대별 절(#1g) — '나' 의 목록을 상대별로 묶는다 ----

/// 페어링된 상대 id — 레일 순서.
function pairedIds() {
  return state.peers.filter((p) => p.paired).map((p) => p.id);
}

/// 접을 수 있는 절 — `<details>`(기본 펼침), 열쇠가 접힘 목록에 있으면 접혀 시작한다. 접거나 펴면 설정 `ui_folded` 에 남긴다.
function foldEl(key, head, body, cls = "peer-fold", headCls = null, shut = false) {
  // 처음 접힘(`shut`, #1q) — 펼친 것을 `open:{열쇠}` 로 기억한다(없으면 접힘).
  const mark = shut ? `open:${key}` : key;
  const d = el("details", { class: cls, open: shut ? state.folded.has(mark) : !state.folded.has(mark), "data-fold": key }, el("summary", { class: headCls }, ...head), ...body);
  d.addEventListener("toggle", () => setFolded(mark, shut ? d.open : !d.open));
  return d;
}

/// 접힘 저장 — 한 번에 하나, 보내는 동안 바뀌면 마지막 목록만 이어서(`SendLatest`). 실패는 상태 줄에, 화면 접힘은 그대로.
const foldSaver = new SendLatest((keys) =>
  api("PUT", "/api/settings", { ui_folded: keys }).catch((e) => {
    $("status_msg").textContent = "접힘 저장 실패: " + e.message;
  }),
);

/// 접힘 바꾸기 — 바뀐 것이 있을 때만 목록을 통째로 저장.
function setFolded(key, folded) {
  if (state.folded.has(key) === folded) return;
  if (folded) state.folded.add(key);
  else state.folded.delete(key);
  foldSaver.push([...state.folded]);
}

/// 목록 행 → 요소. '나' 면 상대별 절(`{화면}:{상대}`, 제목 = 상대 이름 · 건수), 상대 타일이면 그대로. `toEls` 는 빈 목록이면 안내 행을 준다.
function grouped(screen, rows, toEls) {
  if (state.railPeer != null || !rows.length) return toEls(rows);
  return peerGroups(rows, pairedIds(), state.peerNames).map((g) =>
    el("li", { class: "fold" }, foldEl(`${screen}:${g.peer}`, [`${g.name} · ${g.rows.length}`], [el("ol", { class: "rows" }, ...toEls(g.rows))])),
  );
}

/// '나' 의 '대화' 화면이면 요약을 다시 그린다.
function refreshHome() {
  if (route(location.hash).view === "home") renderHome();
}

/// '나' 의 '대화' = 상대별 요약(#1g) — 머리(이름 · 접속 상태 · 대화 수 · 안 읽음 · 대기 요청 · 최근 활동) + 그 상대 대화(누르면 그 대화).
/// 페어링된 상대가 없고 대화도 없으면 처음 안내.
function renderHome() {
  const group = railGroupOf();
  const convs = scopedConvs();
  // 그룹 요약(#1i)은 그 그룹에 대화가 있는 상대만 — '나' 는 대화 없는 페어링 상대도, 상대 타일은 그 상대 하나(#1j).
  const peer = state.peers.find((x) => x.id === state.railPeer);
  const ids = group ? groupPeers(convs, pairedIds()) : state.railPeer != null ? [state.railPeer] : pairedIds();
  const groups = peerSummary(convs, ids, state.peerNames, state.waitingBy);
  $("home_title").textContent = group ? `그룹 · ${group.name}` : peer ? `개요 · ${displayName(peer)}` : "개요";
  $("group_empty").hidden = !group || convs.length > 0;
  $("home_intro").hidden = groups.length > 0 || group != null; // 두 보기 공통(#1z-4 R2)
  // 보기(#1z-4) — 미리보기(기본, 시안 B) · 목록(아래 접는 절).
  const pv = state.homeView === "preview";
  $("home_view_list").setAttribute("aria-pressed", String(!pv));
  $("home_view_preview").setAttribute("aria-pressed", String(pv));
  $("home_summary").hidden = pv;
  $("home_preview").hidden = !pv;
  if (pv) return renderHomePreview(groups, !group && state.railPeer != null);
  const peerOf = (id) => state.peers.find((x) => x.id === id);
  const sig = JSON.stringify([groups.map(({ rows, ...g }) => [g, rows.map((c) => [c.id, c.title, c.last_state, c.unread, c.last_at, c.starred])]), groups.map((g) => presence(peerOf(g.peer)).text)]);
  if (!needsRender($("home_summary"), sig, false)) return;
  $("home_summary").replaceChildren(
    ...groups.map((g) => {
      const p = peerOf(g.peer);
      // 머리 = 사용자 아이콘(그 상대 타일 모양 · 색) · 줄 = 에이전트 로봇 아이콘(#1z-2) — 사람 · 에이전트를 아이콘으로 가른다.
      const head = [
        rowTile(`small ${colorOf(g.peer)}`, p?.icon),
        el("strong", { text: g.name }),
        p ? "" : el("span", { class: "hint", text: "페어링 해제" }),
        g.unread ? el("span", { class: "stat", text: `안 읽음 ${g.unread}` }) : "",
        g.waiting ? el("span", { class: "stat", text: `대기 요청 ${g.waiting}` }) : "",
      ];
      const rows = g.rows.length
        ? g.rows.map((c) => {
            const label = convLabel(c, state.peerNames);
            const meta = convMeta(c);
            return el(
              "li",
              { class: meta.unread ? "unseen" : null },
              el(
                "a",
                { href: `#c/${c.id}` },
                el(
                  "span",
                  { class: "meta" },
                  el("strong", { text: label.title }),
                  el("span", { class: "home-agent" }, speakerEl("agent"), el("span", { text: meta.state ? `${meta.agent} · ${meta.state}` : meta.agent })),
                  meta.unread ? el("span", { class: "stat", text: `안 읽음 ${c.unread}` }) : "",
                  c.last_at != null ? el("span", { text: formatTime(c.last_at) }) : "",
                ),
              ),
            );
          })
        : [el("li", { class: "hint" }, "대화 없음 ", p?.paired ? button(SESSION_NEW, () => newChat(g.peer), "small") : "")];
      return el("li", { class: "fold" }, foldEl(`home:${g.peer}`, head, [el("ol", { class: "rows" }, ...rows)]));
    }),
  );
}

/// 개요 미리보기(#1z-4 시안 B) — 창마다 고른 대화의 마지막 글(`PREVIEW_MSGS`, 서버가 SQL 에서 자름)을 말풍선으로. 다시 읽기는 필요한 대화만
/// (사용자 "Q3 반영" — 5초마다 전부 읽지 않는다): 받은 적 없음 · 요약이 바뀜(`previewKey`) · 변경 신호가 그 대화를 가리킴(`previewStale`) · 개요에 들어옴 ·
/// 보기 바꿈(`previewAll`). 실패한 대화는 다음에 다시. 서명 = 실제로 그릴 말풍선(실패하면 지난 글로 그리고 다음 성공 때 바뀐다). 늦게 온 응답은 차례(`seq`)가
/// 앞선 것으로 덮지 않고, 다른 화면이면 그리지 않는다. 읽음은 바꾸지 않는다(대화를 열어야 읽음).
// ponytail: 이 PC 에서 마지막이 아닌 글만 바뀌는 경우(신호 없음 · 요약 그대로)는 개요에 다시 들어올 때 보인다 — 이 PC 변경 신호가 생기면 그것으로(D39 ②).
async function renderHomePreview(groups, single) {
  const wins = previewWindows(groups, single, state.previewTab);
  const ticket = previewLatest.next();
  const all = state.previewAll;
  const stale = state.previewStale;
  state.previewAll = false;
  state.previewStale = new Set();
  const convs = wins
    .map((w) => w.conv)
    .filter((c) => c && (all || stale.has(c.id) || state.previewMsgs.get(c.id)?.key !== previewKey(c)));
  const seq = ticket;
  const got = await Promise.allSettled(convs.map((c) => api("GET", `/api/conversations/${c.id}/messages?limit=${PREVIEW_MSGS}`)));
  got.forEach((r, i) => {
    const c = convs[i];
    if (r.status !== "fulfilled") state.previewStale.add(c.id);
    else if ((state.previewMsgs.get(c.id)?.seq ?? -1) < seq) state.previewMsgs.set(c.id, { key: previewKey(c), ms: r.value, seq });
  });
  // 이번에 고르지 않은 표시(다른 창의 신호)는 남긴다 — 다음 그리기에서.
  for (const id of stale) if (!convs.some((c) => c.id === id)) state.previewStale.add(id);
  if (!previewLatest.is(ticket)) return;
  const failed = got.find((r) => r.status === "rejected");
  if (failed) $("status_msg").textContent = "미리보기 오류: " + failed.reason.message;
  const live = new Set(state.convs.map((c) => c.id));
  for (const id of state.previewMsgs.keys()) if (!live.has(id)) state.previewMsgs.delete(id);
  for (const id of state.previewStale) if (!live.has(id)) state.previewStale.delete(id);
  if (route(location.hash).view !== "home" || state.homeView !== "preview") return;
  const peerOf = (id) => state.peers.find((x) => x.id === id);
  const sig = JSON.stringify([
    single,
    wins.map((w) => [
      w.key,
      w.name,
      w.waiting,
      w.conv && [
        w.conv.id,
        w.conv.title,
        w.conv.last_state,
        w.conv.unread,
        convMeta(w.conv).agent,
        state.previewMsgs.has(w.conv.id) && previewBubbles(w.conv, state.previewMsgs.get(w.conv.id).ms, state.peerNames.get(w.conv.peer_id)),
      ],
      w.tabs.map((c) => [c.id, c.title, c.last_state, c.unread]),
      presence(peerOf(w.peer)).text,
      peerOf(w.peer)?.paired,
    ]),
  ]);
  const box = $("home_preview");
  if (!needsRender(box, sig, false)) return;
  box.replaceChildren(...wins.map((w) => previewWindowEl(w, single, peerOf(w.peer))));
}

/// 미리보기 창 하나 — 머리(상대: 아이콘 · 이름 · 접속 · 대기 요청 / 상대 타일: 대화 제목 · 상태 · 에이전트) · 대화방 탭 · 본문(누르면 그 대화).
/// 본문은 글자로만 — 상대가 쓴 글을 HTML · 마크다운으로 해석하지 않는다.
function previewWindowEl(w, single, p) {
  const c = w.conv;
  const title = c ? convLabel(c, state.peerNames).title : "";
  const pres = presence(p);
  const head = single
    ? [
        el("strong", { text: title }),
        c.last_state ? el("span", { class: `st-${c.last_state}`, text: stateText(c.last_state) }) : "",
        el("span", { class: "home-agent" }, speakerEl("agent"), el("span", { text: convMeta(c).agent })),
        c.unread ? el("span", { class: "stat", text: `안 읽음 ${c.unread}` }) : "",
      ]
    : [
        rowTile(`small ${colorOf(w.peer)}`, p?.icon),
        el("strong", { text: w.name }),
        el("span", { class: `st-dot st-${pres.key}`, "aria-hidden": "true" }),
        el("span", { text: p ? pres.text : "페어링 해제" }),
        w.waiting ? el("span", { class: "stat", text: `대기 요청 ${w.waiting}` }) : "",
      ];
  // 탭은 `PREVIEW_TABS` 개까지(고른 대화는 늘 보임), 넘치면 "+n개" — 누르면 그 상대 개요(대화마다 창, 사용자 "Q2 반영").
  const { shown, more } = previewTabs(w.tabs, c?.id);
  const tabs = shown.map((t) =>
    el(
      "button",
      {
        type: "button",
        role: "tab",
        class: "pv-tab",
        "aria-selected": String(t.id === c?.id),
        "data-tab": `${w.peer}:${t.id}`,
        onclick: () => pickPreviewTab(w.peer, t.id),
      },
      el("span", { class: "pv-dot", "data-state": t.last_state ?? "", "aria-hidden": "true" }),
      el("span", { text: convLabel(t, state.peerNames).title }),
      t.unread ? el("span", { class: "pv-unread", "aria-label": `안 읽음 ${t.unread}`, text: String(t.unread) }) : "",
    ),
  );
  const cached = c ? state.previewMsgs.get(c.id) : null;
  const bubbles = cached ? previewBubbles(c, cached.ms, state.peerNames.get(c.peer_id)) : [];
  const body = c
    ? el(
        "a",
        { class: "pv-body", href: `#c/${c.id}`, "aria-label": `${title} 대화 열기` },
        ...(!cached
          ? [el("span", { class: "hint", text: "글을 읽지 못했습니다" })]
          : bubbles.length
            ? bubbles.map((b) => el("div", { class: `pv-msg side-${b.side} ${b.tone}` }, b.who !== "나" ? el("b", { text: b.who }) : "", b.text))
            : [el("span", { class: "hint", text: "글 없음" })]),
      )
    : el("div", { class: "pv-empty" }, el("span", { text: "아직 주고받은 대화가 없습니다" }), p?.paired ? button(SESSION_NEW, () => newChat(w.peer), "small") : "");
  return el(
    "article",
    { class: "pv-win", "data-tone": colorOf(w.peer) },
    el("div", { class: "pv-head" }, ...head, c ? el("a", { class: "pv-open", href: `#c/${c.id}`, text: "대화 열기 →" }) : ""),
    tabs.length
      ? el(
          "div",
          { class: "pv-tabs" },
          el("div", { class: "pv-tablist", role: "tablist", "aria-label": `${w.name} 대화방` }, ...tabs),
          more
            ? el("button", { type: "button", class: "pv-more", title: `${w.name} 대화방 ${w.tabs.length}개 모두 보기`, onclick: () => pickTile(w.peer), text: `+${more}개` })
            : "",
        )
      : "",
    body,
  );
}

/// 미리보기 탭 고르기 — 화면 상태에만(저장하지 않는다), 다시 그린 뒤 그 탭에 초점.
async function pickPreviewTab(peer, conv) {
  state.previewTab.set(peer, conv);
  state.previewStale.add(conv); // 고른 대화는 새로 읽는다(받아 둔 글이 오래됐을 수 있다)
  await renderHome();
  $("home_preview").querySelector(`[data-tab="${peer}:${conv}"]`)?.focus();
}

/// 개요 보기 바꾸기(#1z-4) — 바로 그리고 설정 `ui_home_view` 에 남긴다(실패는 상태 줄, 화면은 바꾼 대로).
function setHomeView(v) {
  state.homeView = v;
  state.previewAll = true;
  renderHome();
  api("PUT", "/api/settings", { ui_home_view: v }).catch((e) => {
    $("status_msg").textContent = "보기 저장 실패: " + e.message;
  });
}
for (const v of ["list", "preview"]) $(`home_view_${v}`).addEventListener("click", () => setHomeView(v));

$("conv_search").addEventListener("input", renderConvList);

// ---- 화면 전환 ----

/// 화면 → 메인의 절 id.
const VIEW_EL = {
  home: "home",
  activity: "activity",
  requests: "requests",
  starred: "starred",
  saved: "saved",
  agents: "agents_view",
  agentConvs: "agent_convs",
  peer: "peer",
  conversation: "conversation",
  sent: "conversation",
  new: "new",
  settings: "settings",
};

function show(view) {
  const id = VIEW_EL[view];
  for (const v of new Set(Object.values(VIEW_EL))) $(v).hidden = v !== id;
  // 설정은 대화 목록 없이 레일 옆 전체 폭, 실행 상세는 대화 화면에서만.
  $("sidebar").hidden = $("side_resize").hidden = view === "settings";
  showDetail();
  renderSideNav();
}

/// 실행 상세 열고 닫기(`state.detailOpen`) — 대화 화면일 때만 보인다.
function showDetail() {
  $("detail").hidden = !(state.detailOpen && !$("conversation").hidden);
  $("detail_btn").setAttribute("aria-pressed", String(state.detailOpen));
  renderDetail();
  if (!$("detail").hidden) loadRuns();
}

/// 이 PC 실행 문(`GET /api/runs`) — 실행 상세가 열려 있을 때만 읽는다(새로 고침 주기).
async function loadRuns() {
  try {
    state.runs = await api("GET", "/api/runs");
  } catch (e) {
    state.runs = null;
    $("status_msg").textContent = "실행 목록 오류: " + e.message;
  }
  renderRuns();
}

function renderRuns() {
  const r = state.runs ? runsSummary(state.runs) : null;
  $("detail_runs_n").textContent = r ? r.title : "";
  if (unchanged($("detail_runs"), JSON.stringify(r))) return;
  $("detail_runs").replaceChildren(
    ...(!r
      ? [el("li", { class: "hint", text: "불러오는 중 …" })]
      : r.rows.length
        ? r.rows.map((x) =>
            el(
              "li",
              {},
              el(
                "a",
                {
                  href: x.href || "#",
                  onclick: (ev) => {
                    ev.preventDefault();
                    if (x.href) go(x.href);
                  },
                },
                el("span", { class: "sub", text: x.label }),
                el("span", { class: "text", text: preview(x.text) }),
              ),
            ),
          )
        : [el("li", { class: "hint", text: "실행 · 대기 없음" })]),
  );
}

/// 그린 뒤 초점 — 메뉴로 온 화면의 첫 칸(접힌 `<details>` 안이면 편다).
function applyFocus() {
  const id = state.focusAfter;
  state.focusAfter = null;
  const f = id ? $(id) : null;
  if (!f) return;
  f.closest("details")?.setAttribute("open", "");
  f.focus();
}

async function findConv(id) {
  let conv = state.convs.find((c) => c.id === id);
  if (!conv) {
    await loadConvs();
    conv = state.convs.find((c) => c.id === id);
  }
  return conv;
}

async function render() {
  const r = route(location.hash);
  closeMenus();
  $("rail_me").setAttribute("aria-pressed", String(state.railPeer == null && state.railGroup == null));
  // 옛 보낸 글 탭 주소(`#s/{id}`)는 그 대화로(#1j).
  if (r.view === "sent") {
    location.replace(`#c/${r.id}`);
    return;
  }
  if (r.view === "conversation") {
    const conv = await findConv(r.id);
    if (!conv) {
      $("status_msg").textContent = `대화 ${r.id} 를 찾지 못했습니다`;
      show("home");
      return;
    }
    state.current = conv;
    // 뒤로 가기 등으로 레일에서 고른 PC · 그룹과 다른 대화를 열면 레일을 '나'(전부)로 — 열린 대화로 보낼 수 있게.
    if (!keepsConversation(conv, state.railPeer) || (railGroupOf() && !inGroup([conv], railGroupOf()).length)) pickRail(null);
    renderConvList();
    showConversation(conv);
    state.focusMsg = r.msg ?? null;
    await openConversation(conv);
    applyFocus();
    return;
  }
  // 옛 상대 주고받은 글(`#p/{id}`)은 그 상대를 골라 그 상대 개요로(#1j — 상대 타일의 첫 화면).
  if (r.view === "peer" && r.id != null) {
    pickRail(r.id);
    location.replace("#");
    return;
  }
  state.current = null;
  // 내 활동 · 주고받은 글(`#p`)은 '나' 에만(#1g · #1j).
  if ((r.view === "activity" || r.view === "peer") && state.railPeer != null) pickRail(null);
  // 그룹에는 내 활동 · 주고받은 글 · 에이전트 메뉴가 없다 — '나' 로 바꿔 연다(#1i · #1j).
  if (groupLeaves(r.view) && state.railGroup != null) pickRail(null);
  if (peerLeaves(r.view) && state.railPeer != null) pickRail(null); // 에이전트 응답은 '나' 에만(#1z-2)
  renderConvList();
  show(r.view);
  if (r.view === "home") {
    state.previewAll = true; // 개요에 들어옴 — 미리보기 글을 전부 새로(#1z-4, 다른 화면에서 고친 글)
    renderHome();
  }
  if (r.view === "activity") await loadActivity(true);
  if (r.view === "requests") await loadRequests(true);
  if (r.view === "starred") renderStarred();
  if (r.view === "saved") await loadSaved();
  if (r.view === "agents") loadAgentsView();
  if (r.view === "agentConvs") renderAgentConvs();
  if (r.view === "new") openCompose();
  if (r.view === "peer") await loadPeerMessages(r.id);
  // 초대 수락 칸 — 트리 · 새로 만들기 "초대 수락" · 레일 + 로 오면 펼치고, 다른 설정 항목 · 다른 화면으로 가면 접는다(#1r).
  showAccept(r.view === "settings" && r.focus === "invite_accept");
  if (r.view === "settings" && r.focus) $(r.focus).scrollIntoView({ block: "start" });
  applyFocus();
}

/// 대화 머리글 — 방향 태그 · 제목 · ★ · 부제 `{상대} 의 {에이전트} · {model} · {effort}`(마지막 답의 값, 없으면 뺀다).
function renderConvHead(conv) {
  const label = convLabel(conv, state.peerNames);
  $("conv_title").textContent = label.title;
  const run = state.msgsOf === conv.id ? lastRun(state.msgs, conv.dir !== "out") : "";
  // 닫힌 대화(#1w) — 나감 · 상대가 나감(공유 사본은 `shareHead` 가 말한다).
  const closed = conv.dir === "shared" ? "" : convClosed(conv);
  $("conv_sub").textContent = [label.sub, shareHead(conv), closed, run].filter(Boolean).join(" · ");
  $("star_btn").replaceWith(Object.assign(starButton(conv, label.title), { id: "star_btn" }));
  // 머리 "북마크 n"(#1j) — 이 대화에서 북마크한 글 수, 누르면 세부정보 패널의 북마크 절.
  const saved = state.msgsOf === conv.id ? state.msgs.filter((m) => m.saved).length : 0;
  $("saved_n").textContent = saved ? String(saved) : "";
  const pr = presence(state.peers.find((p) => p.id === conv.peer_id));
  Object.assign($("conv_presence"), { textContent: pr.text, className: `presence ${pr.key}`, hidden: false });
  renderCtx(conv);
}

/// 머리 컨텍스트 표시 겸 단추(#1m) — "42k / 1M" · 60% 주황 · 80% 붉음 · 모름.
function renderCtx(conv) {
  const ms = state.msgsOf === conv.id ? state.msgs : [];
  const lab = contextLabel(contextSize(ms));
  const b = $("ctx_btn");
  b.dataset.level = lab.level;
  $("ctx_text").textContent = lab.text;
  b.title = `컨텍스트 ${lab.text}`;
}

/// 압축 · 비우기 메뉴(#1m) — 고르면 확인 한 번.
function openCtxMenu(conv, anchor) {
  const ms = state.msgsOf === conv.id ? state.msgs : [];
  const answered = ms.some((m) => m.author === "agent");
  const items = ctxMenu(contextBusy(ms), answered).map((it) =>
    el(
      "button",
      {
        type: "button",
        role: "menuitem",
        disabled: it.disabled,
        onclick: () => {
          $("float").replaceChildren(ctxConfirm(conv, it.key));
          $("float").setAttribute("role", "dialog");
          $("float").querySelector("button")?.focus();
        },
      },
      ico(it.icon),
      el("span", { text: it.label }),
    ),
  );
  const why = contextBusy(ms) ? "실행 · 압축이 끝난 뒤에 할 수 있습니다" : answered ? "" : "에이전트 답이 있어야 세션이 생깁니다";
  openFloat(anchor, "menu", "컨텍스트", [el("p", { class: "menu-head" }, ico("gauge"), el("span", { text: "컨텍스트 · 상한 1M" })), ...items, why ? el("p", { class: "hint", text: why }) : ""].filter(Boolean));
}

/// 압축 · 비우기 확인(#1m).
function ctxConfirm(conv, op) {
  const compact = op === "compact";
  return el(
    "div",
    { class: "float-form" },
    el("strong", { text: compact ? "컨텍스트 압축" : "컨텍스트 비우기" }),
    el("p", {
      class: "hint",
      text: compact
        ? "대화를 요약해 새 세션으로 이어 갑니다. 답하는 PC 의 Claude 사용량을 씁니다."
        : "새 세션으로 시작합니다. 에이전트는 앞 대화를 기억하지 않습니다.",
    }),
    el(
      "div",
      { class: "actions" },
      button("취소", () => closeFloat(true)),
      button(
        compact ? "압축" : "비우기",
        async () => {
          closeFloat(false);
          await act(
            compact ? "압축" : "비우기",
            async () => {
              await api("POST", `/api/conversations/${conv.id}/context`, { op });
              return compact ? "압축을 시작했습니다" : "컨텍스트를 비웠습니다";
            },
            "status_msg",
            () => (state.current?.id === conv.id ? openConversation(state.current) : null),
          );
          focusAfterFloat("#ctx_btn", "reply_text");
        },
        "primary",
      ),
    ),
  );
}

$("ctx_btn").addEventListener("click", (ev) => {
  if (state.current) openCtxMenu(state.current, ev.currentTarget);
});

/// 대화 머리글(#1j — 탭 없음, 대화 보기 · 원문 보기). 입력창 안내 = 대화 에이전트에게.
function showConversation(conv) {
  show("conversation");
  editTitle(false);
  renderConvHead(conv);
  // 멘션 — 내가 연 대화(자동 왕복 아님)만. 후보 = 상대 PC 에이전트(타일을 고를 때와 같은 목록 — 없거나 못 받았으면 이때 다시 묻는다).
  const names = state.peerAgents.get(conv.peer_id);
  if (conv.dir === "out" && !conv.relay_agent && (!state.peerAgents.has(conv.peer_id) || names === null)) loadPeerAgents(conv.peer_id);
  closeMention();
  if (state.msgsOf !== conv.id) $("conv_relay").hidden = $("conv_relay_btn").hidden = true;
  // 공유 대화(#1o) · 닫힌 대화(#1w — 나감 · 상대가 나감) — 보기만: 입력창 · 컨텍스트 단추 없음.
  const readOnly = conv.dir === "shared" || !!convClosed(conv);
  $("reply_form").hidden = $("ctx_btn").hidden = readOnly;
}

// ---- 대화 ----

/// 실행 중 경과 — 받은 때부터(대기 포함, 실행 시작 시각은 남지 않는다).
function elapsed(since) {
  return `실행 중 · ${Math.max(0, Math.round(Date.now() / 1000 - since))}초`;
}

function messageEl(conv, m) {
  const who = authorLabel(conv, m, state.peerNames.get(conv.peer_id));
  const side = bubbleSide(m);
  const running = m.state === "running";
  const meta = [formatTime(m.created_at)];
  const progress = outboxText(m) || progressText(m);
  if (progress) meta.push(progress);
  else if (m.state !== "done" && !running) meta.push(stateText(m.state));
  meta.push(refusedText(m));
  if (m.author === "agent" && (m.turns != null || m.duration_ms != null)) {
    meta.push([m.turns != null ? `${m.turns}턴` : "", formatDuration(m.duration_ms)].filter(Boolean).join(" · "));
  }
  meta.push(editedTag(m));
  const acts = messageActions(m, !!convClosed(conv));
  // 말풍선(#1j) — 이 PC 쪽 글은 오른쪽(이름은 "나" 가 아닐 때만), 상대 쪽은 왼쪽에 아바타 · 이름. 방향 라벨은 없다.
  // 에이전트 글(#1z)은 어느 쪽이든 로봇 아바타 + "에이전트" 칩(말풍선 모양은 `app.css` `.msg.agent`).
  const agent = m.author === "agent";
  const article = el(
    "article",
    { class: `msg ${m.author} side-${side}`, "data-msg": String(m.id), "data-tone": colorOf(conv.peer_id), tabindex: "0" },
    agent
      ? el("div", { class: "avatar agent-av", "aria-hidden": "true" }, ico("bot"))
      : side === "theirs"
        ? el("div", { class: `avatar ${colorOf(conv.peer_id)}`, "aria-hidden": "true", text: who.slice(who.lastIndexOf("/") + 1).slice(0, 1) })
        : "",
    el(
      "div",
      { class: "msg-body" },
      el(
        "div",
        { class: "msg-meta" },
        side === "theirs" || who !== "나" ? el("strong", { text: who }) : "",
        agent ? el("span", { class: "tag agent-tag", text: "에이전트" }) : "",
        askTag(m) ? el("span", { class: ["done", "rejected"].includes(m.req_state) ? "tag ask done" : "tag ask", text: askTag(m) }) : "",
        el("span", { text: meta.filter(Boolean).join(" · ") }),
        // 5초마다 바뀌는 경과는 읽어 주지 않는다(대화가 aria-live).
        running ? el("span", { class: "st-running", "data-since": String(m.created_at), "aria-live": "off", text: elapsed(m.created_at) }) : "",
      ),
      el(
        "div",
        { class: "bubble" },
        el(
          "span",
          { class: "msg-icons" },
          bookmarkButton(m),
          iconButton(COPY_SVG, "복사", (ev) => copyText(m, ev.currentTarget)),
        ),
        ...bodyPair(m.text),
      ),
      acts.length
        ? el(
            "div",
            { class: "msg-actions", role: "group", "aria-label": "메시지 작업" },
            ...acts.map((a) => el("button", { type: "button", "data-act": a.key, text: a.label, onclick: () => messageAct(m, a.key) })),
          )
        : "",
    ),
  );
  // 누르면(Enter) 그 글의 요청이 실행 상세의 '진행' 에 — 작업 단추 · 글자 고르기(끌어서 선택)는 빼고.
  const pick = () => {
    state.pickMsg = m.id;
    renderDetail();
  };
  article.addEventListener("click", (ev) => {
    if (ev.target.closest(".msg-actions, .msg-icons") || !window.getSelection().isCollapsed) return;
    pick();
  });
  article.addEventListener("keydown", (ev) => {
    if (ev.target === article && ev.key === "Enter") pick();
  });
  return article;
}

/// 메시지 작업 — 다시 알리기,
/// 요청 상태 바꾸기(`req:{상태}` — 받은 요청, 의견 칸을 열고 확인하면 쓴 쪽에 간다).
function messageAct(m, key) {
  if (key === "req-again") {
    resendState(m.id, m.req_state, m.req_note);
  } else if (key.startsWith("req:")) {
    const bar = $("messages").querySelector(`[data-msg="${m.id}"] .msg-actions`);
    if (bar) noteForm(bar, m.id, key.slice(4), messageActions(m, !!convClosed(state.current)).find((a) => a.key === key)?.label ?? "");
  }
}

/// 말풍선 북마크 아이콘(#1j) — 넣기 · 빼기(이 PC 에만). 북마크한 글은 노란 아이콘이 늘 보인다.
function bookmarkButton(m) {
  const on = m.saved === true;
  const b = iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크", () => toggleBookmark(m));
  b.classList.add("bookmark");
  if (on) b.classList.add("on");
  b.setAttribute("aria-pressed", String(on));
  b.dataset.act = "save";
  return b;
}

/// 북마크 넣기 · 빼기 — 마지막으로 읽어 온 대화 기준. 바꾼 뒤 그 아이콘 · 머리 수 · 실행 상세만(대화를 다시 그리면 HTML 본문이 다시 읽힌다).
function toggleBookmark(m) {
  const cur = state.msgs.find((x) => x.id === m.id) ?? m;
  const on = !cur.saved;
  act(on ? "북마크" : "북마크 해제", async () => {
    await api("PATCH", `/api/messages/${m.id}`, { saved: on });
    cur.saved = on;
    $("messages").querySelector(`[data-msg="${m.id}"] [data-act="save"]`)?.replaceWith(bookmarkButton(cur));
    if (state.current) renderConvHead(state.current);
    renderDetail();
    return on ? "북마크했습니다" : "북마크 해제";
  });
}

// ---- 복사 아이콘 · 편집 모드(#1f) ----

const COPY_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1"></path></svg>';
const BOOKMARK_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path></svg>';
const TRASH_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.9" aria-hidden="true"><path d="M3 6h18"></path><path d="M8 6V4h8v2"></path><path d="M6 6l1 14h10l1-14"></path></svg>';
const PENCIL_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"></path></svg>';

/// 그림 단추(#1k — 그룹설정 절 머리) — 그림은 고정 문자열(위 상수)만.
function svgButton(svg, label, onClick, cls = "") {
  const b = el("button", { type: "button", class: `icon-btn ${cls}`.trim(), title: label, "aria-label": label, onclick: onClick });
  b.innerHTML = svg;
  return b;
}

/// 말풍선 머리줄 아이콘 단추 — 그림은 고정 문자열(위 상수)만.
function iconButton(svg, label, onClick) {
  const b = el("button", { type: "button", class: "icon-btn msg-icon", title: label, "aria-label": label, onclick: onClick });
  b.innerHTML = svg;
  return b;
}

/// 복사 아이콘 — 본문 markdown 원문을 클립보드로, 아이콘 옆에 잠깐 "복사했습니다".
async function copyText(m, btn) {
  try {
    await navigator.clipboard.writeText(m.text);
    $("status_msg").textContent = "복사했습니다";
    btn.classList.add("copied");
    btn.title = "복사했습니다";
    setTimeout(() => {
      btn.classList.remove("copied");
      btn.title = "복사";
    }, 1500);
  } catch (e) {
    $("status_msg").textContent = "복사 실패: " + e.message;
  }
}

/// 열린 대화의 글을 지금 상태(편집 모드 · 고른 글)로 다시 그린다.
function redrawMessages() {
  const conv = state.current;
  const box = $("messages");
  if (!conv || state.msgsOf !== conv.id) return;
  box.dataset.sig = messagesSig(conv, state.msgs);
  box.replaceChildren(...messageEls(conv, state.msgs));
}

/// 말풍선 목록(#1j) — 날짜가 바뀌면 구분선. 컨텍스트 압축 · 비우기(#1m)는 말풍선이 아니라 구분선(압축은 요약 펼치기).
function messageEls(conv, ms) {
  const days = dayBreaks(ms);
  return ms.flatMap((m, i) =>
    [
      days[i] ? el("div", { class: "day-break", role: "separator", text: days[i] }) : "",
      m.author === "context" ? contextEl(m) : messageEl(conv, m),
      feedbackEl(conv, m),
    ].filter(Boolean),
  );
}

/// 요청 의견 말풍선(#1v · #1z 용어) — 대기가 아닌 요청 바로 아래, 받은 쪽 사람 쪽에 지금 상태(크게) · 의견(줄바꿈 그대로) · 바뀐 시각. 상태가 바뀌면 다시 그린다(서명에 `req_*`).
function feedbackEl(conv, m) {
  const f = reqFeedback(m);
  if (!f) return "";
  const who = f.side === "mine" ? "나" : shortName(state.peerNames.get(conv.peer_id));
  return el(
    "article",
    { class: `msg feedback side-${f.side}`, "data-feedback": String(m.id), "data-tone": colorOf(conv.peer_id) },
    f.side === "theirs" ? el("div", { class: `avatar ${colorOf(conv.peer_id)}`, "aria-hidden": "true", text: who.slice(0, 1) }) : "",
    el(
      "div",
      { class: "msg-body" },
      el("div", { class: "msg-meta" }, f.side === "theirs" ? el("strong", { text: who }) : "", el("span", { text: `요청 의견 · ${formatTime(f.at)}` })),
      el(
        "div",
        { class: "bubble" },
        el("span", { class: `tag ask feedback-state req-${f.key}`, text: f.state }),
        f.note ? el("p", { class: "feedback-note", text: f.note }) : "",
      ),
    ),
  );
}

function contextEl(m) {
  return el(
    "div",
    { class: "ctx-break", role: "separator", "data-msg": String(m.id), "data-state": m.state },
    el("span", { class: "ctx-line" }, ico(m.ctx_op === "clear" ? "reset" : "compress"), el("span", { text: `${contextNote(m)} · ${formatTime(m.created_at)}` })),
    m.text ? el("details", { class: "ctx-summary" }, el("summary", { text: "요약 보기" }), el("div", { class: "ctx-text", text: m.text })) : "",
  );
}


/// 요청 상태 바꾸기 — `box` 끝에 의견 칸(3줄 · 글자 수 — #1z) + 취소 · 확인을 연다(이미 열려 있으면 바꾼다). Enter = 확인 · Shift+Enter = 줄바꿈(입력창과 같다).
/// 확인하면 `PATCH` 뒤 새로 고침(대화 · 요청 목록 · 배지).
function noteForm(box, id, to, label, onCancel) {
  box.querySelector(".req-note")?.remove();
  const note = el("textarea", { rows: "3", maxlength: "200", placeholder: "의견(선택) · 요청자에게 보입니다", "aria-label": `${label} 의견` });
  const count = el("span", { class: "hint note-count", "aria-live": "off", text: noteCount("") });
  const form = el(
    "form",
    { class: "req-note" },
    note,
    el(
      "div",
      { class: "req-note-bar" },
      count,
      el("button", {
        type: "button",
        text: "취소",
        onclick: () => {
          form.remove();
          onCancel?.();
        },
      }),
      el("button", { type: "submit", class: "primary", text: label }),
    ),
  );
  note.addEventListener("input", () => (count.textContent = noteCount(note.value)));
  note.addEventListener("keydown", (ev) => {
    if (!enterSends(ev)) return;
    ev.preventDefault();
    form.requestSubmit();
  });
  form.addEventListener("submit", (ev) => {
    ev.preventDefault();
    act(
      `요청 ${label}`,
      async () => {
        const r = await api("PATCH", `/api/messages/${id}`, { req_state: to, note: note.value });
        form.remove();
        return r.warning ?? `요청 ${label} · 요청자에게 알렸습니다`;
      },
      "status_msg",
      refresh,
    );
  });
  box.append(form);
  note.focus();
}

/// 알리지 못한 요청 상태를 같은 상태 · 메모로 다시 보낸다.
function resendState(id, state, note) {
  act(
    "다시 알리기",
    async () => {
      const r = await api("PATCH", `/api/messages/${id}`, { req_state: state, note: note ?? "" });
      return r.warning ?? "요청자에게 다시 알렸습니다";
    },
    "status_msg",
    refresh,
  );
}

function messagesSig(conv, ms) {
  return messagesSigOf(state.peerNames.get(conv.peer_id), ms);
}

/// 그 글로 — 가운데로 스크롤해 잠깐 강조하고 실행 상세의 '진행' 을 그 요청으로(`#c/{대화}/m/{글}`).
function focusMessage(id) {
  const box = $("messages").querySelector(`[data-msg="${id}"]`);
  if (!box) {
    $("status_msg").textContent = `메시지 ${id} 를 이 대화에서 찾지 못했습니다`;
    return;
  }
  state.pickMsg = id;
  renderDetail();
  box.scrollIntoView({ block: "center" });
  box.classList.add("flash");
  setTimeout(() => box.classList.remove("flash"), 2000);
}

/// 열린 대화의 글 — 머리글(부제 · 자동 왕복) · 실행 상세가 쓴다.
function setConvMessages(conv, ms) {
  state.msgs = ms;
  state.msgsOf = conv.id;
  showRelay(conv, ms);
  renderConvHead(conv);
}

async function openConversation(conv) {
  const box = $("messages");
  // 다른 대화로 바꾸면 이전 메시지를 먼저 지운다 — 제목과 본문이 다른 대화를 가리키지 않게.
  if (box.dataset.conv !== String(conv.id)) {
    box.replaceChildren();
    box.dataset.sig = "";
    state.pickMsg = null;
  }
  box.dataset.conv = conv.id;
  const ticket = latest.next();
  let ms;
  try {
    ms = await api("GET", `/api/conversations/${conv.id}/messages`);
  } catch (e) {
    $("status_msg").textContent = "메시지 오류: " + e.message;
    return;
  }
  // 그 사이 다른 대화 · 화면을 열었으면 늦게 온 응답은 버린다.
  if (!latest.is(ticket)) return;
  setConvMessages(conv, ms);
  // 열어 본 대화 = 읽음(이 PC 에만) — 받은 마지막 글까지만(그 뒤 온 글은 안 읽음으로 남게), 탭이 보일 때만. 목록 · 레일 배지를 다시 읽는다.
  if (conv.unread > 0 && ms.length && document.visibilityState === "visible") {
    const upto = Math.max(...ms.map((m) => m.id));
    api("POST", `/api/conversations/${conv.id}/read`, { upto }).then(loadConvs, () => {});
  }
  const sig = messagesSig(conv, ms);
  // 실행 중 경과는 다시 그리지 않고 글자만.
  for (const s of box.querySelectorAll("[data-since]")) s.textContent = elapsed(Number(s.dataset.since));
  if (box.dataset.sig !== sig) {
    box.dataset.sig = sig;
    const atBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 40;
    box.replaceChildren(...messageEls(conv, ms));
    if (atBottom) box.scrollTop = box.scrollHeight;
  }
  renderDetail();
  // 갈 글 — 이 대화를 실제로 그린 뒤(늦게 온 응답 · 새로 고침이 그려도).
  if (state.focusMsg != null) {
    const id = state.focusMsg;
    state.focusMsg = null;
    focusMessage(id);
  }
}

// ---- 대화 제목(이 PC 에만) ----

/// 제목 고치기 켜고 끄기 — 켜면 머리글 제목 줄 대신 입력칸(처음 값 = 지금 제목, 비우면 첫 메시지 앞부분).
function editTitle(on) {
  $("title_form").hidden = !on;
  $("conv_title_line").hidden = on;
  if (!on) return;
  // 첫 글에서 따온 제목의 줄바꿈 · 탭은 공백으로 — 그대로 저장하면 제어 문자로 거절된다.
  $("title_input").value = (state.current?.title ?? "").replace(/\p{Cc}+/gu, " ").trim();
  $("title_input").focus();
  $("title_input").select();
}

$("title_edit_btn").addEventListener("click", () => editTitle(true));
$("title_cancel").addEventListener("click", () => editTitle(false));
$("title_input").addEventListener("keydown", (ev) => {
  if (ev.key === "Escape") editTitle(false);
});
$("title_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const conv = state.current;
  if (!conv) return;
  const title = $("title_input").value;
  act("제목 저장", async () => {
    await api("PATCH", `/api/conversations/${conv.id}`, { title });
    await loadConvs();
    if (state.current?.id === conv.id) {
      editTitle(false);
      renderConvHead(state.current);
    }
    return title.trim() ? "제목 저장" : "제목 지움 — 첫 메시지 앞부분으로";
  });
});

/// 머리글의 자동 왕복 표시 · 멈춤 · 재개 버튼.
function showRelay(conv, ms) {
  const r = relayText(conv, ms);
  const btn = $("conv_relay_btn");
  $("conv_relay").hidden = r == null;
  $("conv_relay").textContent = r?.text ?? "";
  btn.hidden = !r?.action || !!convClosed(conv);
  btn.textContent = r?.action === "resume" ? "재개" : "멈춤";
  btn.dataset.on = String(r?.action === "resume");
}

/// 자동 왕복 멈춤 · 재개 — 머리글 · 실행 상세 버튼이 같은 API.
function switchRelay(on) {
  const conv = state.current;
  if (!conv || state.msgsOf !== conv.id) return; // 지금 대화의 글을 받기 전 단추
  act(
    on ? "재개" : "멈춤",
    async () => {
      await api("POST", `/api/conversations/${conv.id}/relay`, { on });
      return on ? "자동 왕복 재개" : "자동 왕복 멈춤";
    },
    "status_msg",
    refresh,
  );
}

/// 세부정보 "관람자"(#1o) — 내 대화 · agent 대화: 관람자 목록(빼기) · 추가(페어링된 상대, 5명까지). 공유 대화(사본): 누가 공유했는지 · 나가기.
/// 원 상대 쪽(관람자 수만 안다)은 머리 "관람자 n" 만.
function renderShare(conv) {
  const shared = conv.dir === "shared";
  const mine = conv.observers ?? [];
  $("detail_share_sec").hidden = false;
  $("detail_share_n").textContent = shared ? "" : mine.length ? `${mine.length} / ${MAX_OBSERVERS}` : conv.watchers ? String(conv.watchers) : "";
  const note = shared
    ? `${josaName(shortName(state.peerNames.get(conv.peer_id)))} 공유한 대화 · ${conv.share?.ended != null ? "공유 해제됨" : "읽기 전용"}`
    : conv.watchers && !mine.length
      ? `상대가 관람자 ${conv.watchers}명에게 공유했습니다`
      : "";
  Object.assign($("detail_share_note"), { textContent: note, hidden: !note });
  if (!unchanged($("detail_share"), JSON.stringify([conv.id, mine, state.peers.map((p) => [p.id, displayName(p)])])))
    $("detail_share").replaceChildren(
      ...mine.map((id) => {
        const p = state.peers.find((x) => x.id === id);
        const name = p ? displayName(p) : "알 수 없는 상대";
        return el(
          "li",
          { class: "share-row" },
          el("span", { text: name }),
          el("button", { type: "button", class: "small", "aria-label": `${name} 관람자에서 빼기`, text: "빼기", onclick: () => removeObserver(conv, id, name) }),
        );
      }),
    );
  const cands = shareCandidates(conv, state.peers);
  $("share_tools").hidden = shared || !!convClosed(conv) || cands.length === 0;
  if (!shared)
    $("share_peer").replaceChildren(...cands.map((p) => el("option", { value: String(p.id), text: displayName(p) })));
  $("share_leave_btn").hidden = !shared || conv.share?.ended != null;
}

/// 이름 + 주격 조사(이 · 가).
function josaName(name) {
  return `${name}${josa(name, "이", "가")}`;
}

function addObserver() {
  const conv = state.current;
  const id = Number($("share_peer").value);
  if (!conv || !id) return;
  act("관람자 추가", async () => {
    await api("POST", `/api/conversations/${conv.id}/observers`, { peer_id: id });
    return "관람자를 넣었습니다";
  }, "status_msg", loadConvs);
}

function removeObserver(conv, id, name) {
  act("관람자 빼기", async () => {
    await api("DELETE", `/api/conversations/${conv.id}/observers/${id}`);
    return `${name} 을 관람자에서 뺐습니다`;
  }, "status_msg", loadConvs);
}

$("share_add_btn").addEventListener("click", addObserver);
// 공유 대화 나가기 — ⋯ "나가기" 와 같다(#1x — 확인 뒤 공유를 끝내고 사본을 이 PC 목록에서 삭제).
$("share_leave_btn").addEventListener("click", (ev) => {
  const conv = state.current;
  if (!conv) return;
  openFloat(ev.currentTarget, "dialog", "대화 나가기", [convEndForm(conv, "leave")]);
});

for (const id of ["conv_relay_btn", "detail_relay_btn"]) $(id).addEventListener("click", () => switchRelay($(id).dataset.on === "true"));

// ---- 실행 상세 — 진행 · 대화 정보 · 자동 왕복 · 북마크 항목(이 대화) ----

function stepEl(s) {
  return el(
    "li",
    { class: s.state },
    el("div", { class: "t", text: s.title }),
    s.at ? el("div", { class: "at", text: s.at }) : "",
    s.meta ? el("div", { class: "meta", text: s.meta }) : "",
  );
}

function renderDetail() {
  const conv = state.current;
  if ($("detail").hidden || !conv) return;
  // 이 대화의 글을 받기 전에는 이전 대화의 내용을 남기지 않는다.
  const ready = state.msgsOf === conv.id;
  const ms = ready ? state.msgs : [];
  // 진행 — 고른 글의 요청(그 답을 골라도 그 요청), 없으면 마지막 요청. 고른 요청은 대화에서 표시.
  const items = timeline(ms);
  const item = items.find((i) => i.request.id === state.pickMsg || i.reply?.id === state.pickMsg) ?? items.at(-1);
  const steps = !ready ? "불러오는 중 …" : item ? runSteps(item) : "요청 없음";
  if (!unchanged($("detail_steps"), JSON.stringify([conv.id, steps])))
    $("detail_steps").replaceChildren(...(typeof steps === "string" ? [el("li", { class: "hint", text: steps })] : steps.map(stepEl)));
  const picked = state.pickMsg != null && item ? item.request.id : null;
  for (const a of $("messages").querySelectorAll(".msg.picked")) if (a.dataset.msg !== String(picked)) a.classList.remove("picked");
  if (picked != null) $("messages").querySelector(`[data-msg="${picked}"]`)?.classList.add("picked");
  // 대화 정보 — 실행 기본값: 받은 대화는 이 PC 의 상대별 덮어쓰기 → 전역 설정, 보낸 대화는 상대 PC 가 정한다(마지막 실행 값을 보인다).
  const p = state.peers.find((x) => x.id === conv.peer_id);
  const out = conv.dir === "out";
  $("info_peer").textContent = p ? `${displayName(p)} (${p.host})` : "알 수 없는 상대";
  const shared = conv.dir === "shared";
  $("info_agent_label").textContent = shared ? "대화 에이전트" : out ? "상대 에이전트" : "내 에이전트";
  $("info_agent").textContent = conv.agent;
  const run = lastRun(ms, !out);
  // 공유 대화(#1o) — 이 PC 가 실행하지 않는다: 마지막 실행 값만.
  $("info_defaults").textContent = shared
    ? run || "—"
    : out
    ? run
      ? `${run} · 마지막 실행`
      : ready
        ? "상대 PC 가 정함"
        : "불러오는 중 …"
    : `${p?.model || state.settings.claude_model || "sonnet"} · ${p?.effort || state.settings.claude_effort || "high"}`;
  // 자동 왕복 — 진행 n/상한 · 멈춤(재개는 연 쪽). 글을 받기 전에는 숨긴다(단추가 이전 대화에 작동하지 않게).
  const relay = ready ? relayText(conv, ms) : null;
  $("detail_relay_sec").hidden = relay == null;
  if (relay) {
    const capped = out && conv.relay_cap > 0;
    $("detail_relay_text").textContent = relay.text;
    $("detail_relay_count").textContent = out && conv.relay_agent ? `내 ${conv.relay_agent} ⇄ ${conv.agent}` : "";
    $("detail_relay_bar").parentElement.hidden = !capped;
    if (capped) $("detail_relay_bar").style.width = `${Math.min(100, Math.round(((conv.relay_count ?? 0) / conv.relay_cap) * 100))}%`;
    const btn = $("detail_relay_btn");
    btn.hidden = !relay.action || !!convClosed(conv); // 닫힌 대화는 읽기 전용(#1w)
    btn.textContent = relay.action === "resume" ? "재개" : "멈춤";
    btn.dataset.on = String(relay.action === "resume");
  }
  renderShare(conv);
  // 요청 — 이 대화의 요청 글(최근 글부터, 상태 · 메모).
  const peerName = state.peerNames.get(conv.peer_id);
  const reqs = ms.filter((m) => m.author === "human" && m.request).reverse();
  $("detail_req_n").textContent = reqs.length ? String(reqs.length) : "";
  if (!unchanged($("detail_req"), JSON.stringify([conv.id, ready, peerName, reqs.map((m) => [m.id, m.req_state, m.req_note, m.text.length])])))
    $("detail_req").replaceChildren(
      ...(reqs.length
        ? reqs.map((m) => detailItem(conv, m, [askTag(m), m.req_note, authorLabel(conv, m, peerName), formatTime(m.created_at)].filter(Boolean).join(" · ")))
        : [el("li", { class: "hint", text: !ready ? "불러오는 중 …" : "요청 없음" })]),
    );
  // 북마크 항목 — 이 대화에서 북마크한 글(최근 글부터).
  const saved = ms.filter((m) => m.saved).reverse();
  $("detail_saved_n").textContent = saved.length ? String(saved.length) : "";
  if (unchanged($("detail_saved"), JSON.stringify([conv.id, ready, peerName, saved.map((m) => [m.id, m.text.length])]))) return;
  $("detail_saved").replaceChildren(
    ...(saved.length
      ? saved.map((m) => detailItem(conv, m, `${authorLabel(conv, m, peerName)} · ${formatTime(m.created_at)}`))
      : [el("li", { class: "hint", text: ready ? "북마크 없음" : "불러오는 중 …" })]),
  );
}

/// 실행 상세 목록의 글 한 줄 — 위 줄 `sub`(발화자 · 시각 …) · 본문 앞부분, 누르면 그 글로(`#c/{대화}/m/{글}`).
function detailItem(conv, m, sub) {
  const href = `#c/${conv.id}/m/${m.id}`;
  return el(
    "li",
    {},
    el(
      "a",
      {
        href,
        onclick: (ev) => {
          ev.preventDefault();
          go(href);
        },
      },
      el("span", { class: "sub", text: sub }),
      el("span", { class: "text", text: preview(m.text) }),
    ),
  );
}

// 머리글 "북마크 n" — 실행 상세를 열고 그 절로.
$("saved_btn").addEventListener("click", () => {
  state.detailOpen = true;
  showDetail();
  $("detail_saved_sec").scrollIntoView({ block: "start" });
});

/// 열린 대화에 새 요청 — 입력창. 보내는 대상은 지금 화면에 열린 대화(해시)이고, 레일에서 고른 상대와 맞아야 한다.
/// 글은 대화 에이전트에게 — 멘션이면 부른 에이전트에게. `opts` = `{request, agent}`(글 종류 · 멘션). `sent` = 보낸 뒤 할 일(입력창 비우기).
function sendText(text, opts, sent) {
  const r = route(location.hash);
  const conv = state.current;
  if (r.view !== "conversation" || conv?.id !== r.id || !keepsConversation(conv, state.railPeer)) {
    $("status_msg").textContent = "보내지 않았습니다. 대화를 다시 여세요.";
    return;
  }
  if (!text.trim()) {
    $("status_msg").textContent = "내용이 비어 보내지 않았습니다";
    return;
  }
  if (state.sending) {
    $("status_msg").textContent = "앞 메시지를 보내는 중입니다";
    return;
  }
  state.sending = true;
  const id = conv.id;
  act(
    "보내기",
    async () => {
      const res = await api("POST", `/api/conversations/${id}/messages`, { text, request: opts.request === true, ...(opts.agent ? { agent: opts.agent } : {}) });
      sent?.();
      state.pickMsg = null; // 실행 상세 '진행' = 방금 보낸 요청
      return res.warning ?? "보냈습니다";
    },
    "status_msg",
    async () => {
      state.sending = false;
      await refresh();
    },
  );
}

$("reply_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  // 글 종류는 보낸 뒤 "질의" 로 돌아간다. 멘션 = 글의 첫 `@이름`(상대 PC 에이전트 목록에 있는 것).
  const ask = $("reply_form").elements.ask;
  const text = $("reply_text").value;
  const reset = () => {
    $("reply_text").value = "";
    fitComposer();
    ask.value = "query";
  };
  // 요청(#1d) — 실행이 없어 부를 에이전트(멘션)도 없다.
  if (ask.value === "request") {
    sendText(text, { request: true }, reset);
    return;
  }
  const conv = state.current;
  const names = mentionNames();
  const canMention = conv?.dir === "out" && !conv.relay_agent;
  // 멘션을 쓰는 대화인데 후보 목록이 없으면(받는 중 · 못 받음) 멘션처럼 보이는 글은 보내지 않는다 — 대화 에이전트로 조용히 가지 않게. 목록을 다시 묻는다.
  if (names == null && canMention && hasMention(text)) {
    $("status_msg").textContent = "에이전트 목록을 받지 못해 보내지 않았습니다. 잠시 뒤 다시 보내세요.";
    if (state.peerAgents.get(conv.peer_id) !== undefined || !state.peerAgents.has(conv.peer_id)) loadPeerAgents(conv.peer_id);
    return;
  }
  const agent = mentionTarget(text, names ?? []);
  // 멘션 꼴 `@이름` 이 후보와 하나도 맞지 않으면(오타 · 낡은 목록) 한 번 막고 알린다 — 목록을 다시 받고, 같은 글을 다시 보내면 멘션 없이 보낸다
  // (코드의 `@Override` 같은 글도 보낼 수 있게).
  if (canMention && names && !agent && hasMention(text) && state.mentionWarned !== text) {
    state.mentionWarned = text;
    $("status_msg").textContent = `${mentionToken(text)}: 없는 에이전트입니다. 멘션이 아니면 한 번 더 보내세요.`;
    loadPeerAgents(conv.peer_id);
    return;
  }
  state.mentionWarned = null;
  sendText(text, { request: false, agent }, reset);
});

/// 입력창 서식 버튼 — 고른 글자에 markdown 표기(`view.js` `insertMarkdown`).
for (const b of document.querySelectorAll("[data-md]"))
  b.addEventListener("click", () => {
    const t = $("reply_text");
    const r = insertMarkdown(t.value, t.selectionStart, t.selectionEnd, b.dataset.md);
    t.value = r.text;
    t.focus();
    t.setSelectionRange(r.start, r.end);
  });
// Enter = 보내기 · Shift+Enter = 줄바꿈, 한글 조합 중 Enter 는 보내지 않는다(`view.js` `enterSends`). 멘션 후보가 열려 있으면 ↑ ↓ 고르기 ·
// Enter · Tab 넣기 · Esc 닫기(그동안 Enter 는 보내지 않는다).
$("reply_text").addEventListener("keydown", (ev) => {
  const m = state.mention;
  if (m && !ev.isComposing && ev.keyCode !== 229) {
    if (ev.key === "ArrowDown" || ev.key === "ArrowUp") {
      ev.preventDefault();
      m.active = (m.active + (ev.key === "ArrowDown" ? 1 : m.items.length - 1)) % m.items.length;
      renderMention();
      return;
    }
    if ((ev.key === "Enter" || ev.key === "Tab") && !ev.shiftKey) {
      ev.preventDefault();
      pickMention(m.active);
      return;
    }
    if (ev.key === "Escape") {
      ev.preventDefault();
      closeMention();
      return;
    }
  }
  if (!enterSends(ev)) return;
  ev.preventDefault();
  $("reply_form").requestSubmit();
});
$("reply_text").addEventListener("input", updateMention);
// 입력창 글 칸(#1z) — 한 줄(44px)에서 시작해 글만큼 늘고 200px 에서 멈춘다(그 뒤는 칸 안 스크롤).
function fitComposer() {
  const t = $("reply_text");
  t.style.height = "auto";
  t.style.height = Math.min(t.scrollHeight, 200) + "px";
}
$("reply_text").addEventListener("input", fitComposer);
// 커서만 옮기면 열린 후보만 다시 거른다 — 닫힌 창은 열지 않는다(이미 쓴 멘션 안으로 옮겨 Enter 로 보낼 때 막지 않게).
const moveMention = () => {
  if (state.mention) updateMention();
};
$("reply_text").addEventListener("click", moveMention);
$("reply_text").addEventListener("keyup", (ev) => {
  if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(ev.key)) moveMention();
});
$("reply_text").addEventListener("blur", closeMention);
// 목록을 눌러도(스크롤바 포함) 입력창 초점을 뺏지 않는다 — 항목은 click 으로 넣는다.
$("mention_box").addEventListener("mousedown", (ev) => ev.preventDefault());

/// 멘션 후보 — 열린 대화가 내가 연 대화(자동 왕복 아님)이고 상대 에이전트 목록을 받았으면 그 이름들, 아니면 `null`.
function mentionNames() {
  const conv = state.current;
  if (!conv || conv.dir !== "out" || conv.relay_agent) return null;
  const names = state.peerAgents.get(conv.peer_id);
  return Array.isArray(names) ? names : null;
}

/// 커서 앞의 `@…` 로 후보를 거른다(앞부분 일치, 대소문자 무시) — 없으면 닫는다.
function updateMention() {
  const t = $("reply_text");
  const names = mentionNames();
  const q = names ? mentionQuery(t.value, t.selectionStart) : null;
  const items = q ? names.filter((n) => n.toLowerCase().startsWith(q.query.toLowerCase())) : [];
  state.mention = items.length ? { start: q.start, items, active: 0 } : null;
  renderMention();
}

function closeMention() {
  state.mention = null;
  renderMention();
}

/// 후보 목록(listbox) — 고른 항목은 `aria-activedescendant` 로 입력창에 알린다. 누르면 넣는다 — 입력창 초점은 그대로(목록 mousedown 막음).
function renderMention() {
  const m = state.mention;
  const t = $("reply_text");
  $("mention_box").hidden = !m;
  if (!m) {
    t.removeAttribute("aria-activedescendant");
    return;
  }
  $("mention_box").replaceChildren(
    ...m.items.map((n, i) =>
      el("li", {
        id: `mention_${i}`,
        role: "option",
        "aria-selected": String(i === m.active),
        text: n,
        onclick: () => pickMention(i),
      }),
    ),
  );
  t.setAttribute("aria-activedescendant", `mention_${m.active}`);
}

/// 입력 중인 `@…`(커서 뒤 그 낱말 끝까지)를 `@이름` 으로 바꾸고 커서를 그 뒤로(뒤가 공백이 아니면 공백 하나). 그사이 커서 · 글 · 선택이 바뀌어
/// 같은 `@` 가 아니면 닫기만 한다.
function pickMention(i) {
  const t = $("reply_text");
  const m = state.mention;
  const q = mentionQuery(t.value, t.selectionStart);
  if (!q || q.start !== m.start || t.selectionStart !== t.selectionEnd) {
    closeMention();
    return;
  }
  const tail = t.value.slice(t.selectionStart);
  const after = tail.slice(tail.match(/^\S*/)[0].length);
  const insert = `@${m.items[i]}${/^\s/.test(after) ? "" : " "}`;
  const before = t.value.slice(0, m.start);
  t.value = before + insert + after;
  t.setSelectionRange(before.length + insert.length, before.length + insert.length);
  closeMention();
}

// ---- 내 활동 · 상대와 주고받은 글 ----

function rowEls(rows, empty) {
  if (!rows.length) return [el("li", { class: "hint", text: empty })];
  return rows.map((r) =>
    el(
      "li",
      {},
      el(
        "a",
        { href: r.href },
        el(
          "span",
          { class: "meta" },
          r.author ? speakerEl(r.author) : "",
          el("strong", { text: r.who }),
          el("span", { text: formatTime(r.at) }),
          r.room ? el("span", { class: "chip", text: r.room }) : "",
          r.kind ? el("span", { class: "tag ask", text: r.kind }) : "",
        ),
        el("span", { text: r.preview }),
      ),
    ),
  );
}

/// 내 활동 행 — 종류 라벨 · 발화자 · 시각 · 대화 제목 · 본문 앞부분, 안 본 항목은 강조.
function activityEls(rows) {
  if (!rows.length) return [el("li", { class: "hint", text: "내 활동 없음" })];
  return rows.map((r) =>
    el(
      "li",
      { class: r.unseen ? "unseen" : null },
      el(
        "a",
        { href: r.href },
        el(
          "span",
          { class: "meta" },
          el("span", { class: `tag act-${r.tone}`, text: r.label }),
          r.author ? speakerEl(r.author) : "",
          el("strong", { text: r.who }),
          el("span", { text: formatTime(r.at) }),
          el("span", { class: "hint", text: r.title }),
        ),
        r.preview ? el("span", { text: r.preview }) : "",
        r.note ? el("span", { class: "hint keep-lines", text: `의견: ${r.note}` }) : "",
      ),
    ),
  );
}

/// 내 활동 — '나' 에만(#1g, 상대별 절). 열 때(`open`) 거르기를 처음으로 되돌리고(본 것으로 표시하는 범위 = 보이는 범위) 전부를 본 것으로(탭이 보일 때만).
/// 그때 안 본 항목은 다시 그려도 강조를 남긴다. 여는 동안의 새로 고침은 건너뛴다(늦게 온 것으로 버려져 표시를 놓치지 않게).
async function loadActivity(open = false) {
  if (!open && state.actOpening) return;
  if (open) {
    state.actOpening = true;
    state.actKind = "all";
    $("act_q").value = "";
  }
  renderActTabs();
  try {
    const params = new URLSearchParams({ kind: state.actKind });
    if ($("act_q").value.trim()) params.set("q", $("act_q").value.trim());
    const ticket = latest.next();
    let r;
    try {
      r = await api("GET", "/api/activity?" + params);
    } catch (e) {
      $("status_msg").textContent = "내 활동 오류: " + e.message;
      return;
    }
    if (!latest.is(ticket)) return;
    const rows = activityRows(r.items, state.peerNames);
    if (open) state.actFresh = new Set(rows.filter((x) => x.unseen).map((x) => x.key));
    for (const x of rows) x.unseen ||= state.actFresh.has(x.key);
    if (needsRender($("act_list"), JSON.stringify([rows, state.peerNames.size]), open)) $("act_list").replaceChildren(...grouped("activity", rows, activityEls));
    if (open && document.visibilityState === "visible") {
      // 받은 범위(`upto`)까지만 — 받은 뒤 온 것은 안 본 것으로 남게. `peer` 가 없으니 전부.
      await api("POST", "/api/activity/seen", r.upto).catch(() => {});
      await loadCounts();
    }
  } finally {
    if (open) state.actOpening = false;
  }
}

let actTimer;
$("act_q").addEventListener("input", () => {
  clearTimeout(actTimer);
  actTimer = setTimeout(() => loadActivity(), 250);
});

/// 내 활동 거르기 탭(#1q) — 전체 · 받은 요청 · 내 요청 · 받은 답 · 실패, 아이콘 + 이름(#1r).
function renderActTabs() {
  $("act_tabs").replaceChildren(
    ...ACT_KINDS.map((k) =>
      el("a", {
        href: "#activity",
        "data-kind": k.kind,
        "aria-current": k.kind === state.actKind ? "page" : "false",
        onclick: (ev) => {
          ev.preventDefault();
          state.actKind = k.kind;
          loadActivity();
        },
      }, ico(k.icon), el("span", { text: k.label })),
    ),
  );
}

/// 안 본 내 활동 수 — 사이드바 내비(고른 타일 범위) · 종 점(전부).
async function loadCounts() {
  const peer = state.railPeer;
  const group = state.railGroup;
  const scope = scopeQuery(peer, group);
  let r;
  try {
    r = await api("GET", "/api/activity?limit=0" + (peer != null ? `&peer=${peer}` : ""));
  } catch {
    return; // 다음 새로 고침에 다시
  }
  let waiting = state.reqWaiting;
  let by = null;
  try {
    const w = await api("GET", `/api/messages?side=theirs&ask=request&req=wait&limit=500${scope ? `&${scope}` : ""}`);
    waiting = w.length;
    by = waitingByPeer(w);
  } catch {
    // 다음 새로 고침에 다시
  }
  if (state.railPeer !== peer || state.railGroup !== group) return; // 그 사이 다른 타일을 골랐다
  state.reqWaiting = waiting;
  if (by) {
    state.waitingBy = by;
    refreshHome();
  }
  state.unseen = peer != null ? r.unseen : r.unseen_all;
  state.unseenAll = r.unseen_all;
  $("bell_dot").hidden = state.unseenAll === 0;
  renderSideNav();
}

/// '요청' 화면(#1q) — 개요(`state.reqBox` = `null`: 받은 요청 · 보낸 요청 두 절, 머리에 상태별 건수) 또는 탭으로 고른 쪽 목록만. 고른 타일 범위,
/// 최신부터. 머리 건수는 서버 집계(범위 전체), 상태 거르기는 서버에서(`req=` — 오래된 대기 요청도 나온다). 메뉴로 열면 개요 · 거르기 처음으로.
/// ponytail: 행은 쪽(· 상태)마다 최근 500건(서버 최대)까지 — 넘으면 커서 페이지.
async function loadRequests(open = false) {
  if (open) {
    $("req_state").value = "";
    $("req_archive").checked = false; // archive 항목은 들어올 때마다 숨김(#1y)
    state.reqBox = reqBoard() ? "theirs" : null; // 보드는 탭 한 쪽만(#1v) — 개요는 목록
  }
  const box = state.reqBox;
  const board = reqBoard() && box != null;
  for (const b of ["theirs", "mine"]) $(`req_tab_${b}`).setAttribute("aria-current", b === box ? "page" : "false");
  $("req_view_list").setAttribute("aria-pressed", String(!board));
  $("req_view_board").setAttribute("aria-pressed", String(board));
  // 보드의 열이 곧 상태(#1v) — 상태 거르기는 목록에서만.
  $("req_state").hidden = $("req_state_label").hidden = board;
  const boxes = box ? [box] : ["theirs", "mine"];
  const scope = scopeQuery(state.railPeer, state.railGroup);
  const st = board ? "" : $("req_state").value;
  const side = (b) => `side=${b}${scope ? `&${scope}` : ""}`;
  const archived = $("req_archive").checked;
  const ticket = latest.next();
  let lists, counts, olds;
  try {
    [lists, counts, olds] = await Promise.all([
      Promise.all(boxes.map((b) => api("GET", `/api/messages?${side(b)}&ask=request&limit=500${st ? `&req=${st}` : ""}`))),
      Promise.all(boxes.map((b) => api("GET", `/api/messages/req_counts?${side(b)}`))),
      archived ? Promise.all(boxes.map((b) => api("GET", `/api/messages?${side(b)}&ask=request&archived=true&limit=500`))) : null,
    ]);
  } catch (e) {
    $("status_msg").textContent = "요청 목록 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const data = reqBoxes(boxes, lists, counts, state.peerNames);
  renderReqArchive(boxes, olds);
  $("req_list").hidden = board;
  $("req_board").hidden = !board;
  if (board) {
    if (needsRender($("req_board"), JSON.stringify([box, state.railPeer, state.railGroup, data]), open))
      $("req_board").replaceChildren(...boardEls(boardColumns(data[0].rows, data[0].sum)));
    return;
  }
  if (!needsRender($("req_list"), JSON.stringify([box, st, state.railPeer, state.railGroup, data]), open)) return;
  $("req_list").replaceChildren(...(box ? grouped(`requests:${box}`, data[0].rows, (rs) => requestEls(rs, box)) : data.map(reqBoxEl)));
}

/// archive 된 요청(#1y) — "archive 항목 보기" 일 때만, 목록 · 보드 아래 따로 된 절(쪽마다, 처음 펼침).
function renderReqArchive(boxes, olds) {
  const list = $("req_archive_list");
  list.hidden = !olds;
  if (!olds) return list.replaceChildren();
  list.replaceChildren(
    ...boxes.map((box, i) => {
      const rows = requestRows(olds[i], state.peerNames);
      const head = [el("strong", { text: `archive — ${box === "theirs" ? "받은 요청" : "보낸 요청"}` }), el("span", { class: "stat", text: String(rows.length) })];
      const body = [el("ol", { class: "rows" }, ...(rows.length ? rows.map(requestItem) : [el("li", { class: "hint", text: "없음" })]))];
      return el("li", { class: "fold" }, foldEl(`reqarchive:${box}`, head, body, "peer-fold req-box"));
    }),
  );
}

/// 요청 archive · 해제(#1y — 이 PC 에만).
function archiveReq(id, on) {
  act("archive", async () => {
    await api("POST", `/api/messages/${id}/archive`, { archived: on });
    await loadRequests();
    return on ? "archive 로 옮김" : "archive 해제";
  });
}

/// 요청 보기(#1v — D39 ①) — 보드면 참. 이 PC 에 남긴다.
/// ponytail: 접힘 목록(`ui_folded`)의 열쇠 하나를 빌려 쓴다 — 보기 설정(그룹 · 정렬 · 거르기)이 늘면 전용 설정 키.
const REQ_BOARD = "reqboard:me";
function reqBoard() {
  return state.folded.has(REQ_BOARD);
}

/// 요청 보드(#1v) — 열 = 상태(머리 = 이름 · 서버 집계 건수), 카드 = 요청 행 그대로(글자 단추 · 의견 칸 포함). 받은 요청 카드는 손잡이로 끌어
/// 다른 열에 놓으면 카드가 그 열로 옮겨진 채 의견 칸이 열리고, 확인해야 요청자에게 간다(취소면 제자리로).
/// 끌기는 포인터 이벤트 — HTML 끌어 놓기(`draggable`)는 브라우저 끌기 확장(링크 · 글 끌어 검색 등)이 가로채 반응한다(사용자 2026-10-19).
function boardEls(cols) {
  return cols.map((c) => {
    const cards = c.rows.map((r) => {
      const li = requestItem(r);
      li.classList.add("board-card", `req-${r.stateKey}`);
      if (r.moves.length) {
        const grip = el("button", { type: "button", class: "drag-handle", title: "끌어서 상태 옮기기", "aria-label": "끌어서 상태 옮기기" }, ico("grip"));
        grip.addEventListener("pointerdown", (ev) => boardGrab(ev, li, r));
        li.prepend(grip);
      }
      return li;
    });
    return el(
      "section",
      { class: `board-col req-${c.key}`, "data-col": c.key },
      el("h3", { class: "board-head" }, el("span", { class: `tag ask req-${c.key}`, text: c.name }), el("b", { class: "count", text: String(c.count) })),
      el("ol", { class: "rows" }, ...(cards.length ? cards : [el("li", { class: "hint", text: "없음" })])),
    );
  });
}

/// 보드 카드 끌기(포인터) — 손잡이를 누른 채 움직이면 카드 사본이 따라오고, 놓을 수 있는 열은 강조 · 없는 열은 흐리게. 놓을 수 있는 열 위에서 떼면 `boardDrop`.
function boardGrab(ev, li, r) {
  if (ev.button !== 0) return;
  ev.preventDefault();
  const grip = ev.currentTarget;
  grip.setPointerCapture(ev.pointerId);
  const box = li.getBoundingClientRect();
  const dx = ev.clientX - box.left;
  const dy = ev.clientY - box.top;
  const ghost = li.cloneNode(true);
  ghost.classList.add("board-ghost");
  ghost.style.width = `${box.width}px`;
  document.body.append(ghost);
  li.classList.add("dragging");
  const cols = [...$("req_board").querySelectorAll(".board-col")];
  for (const c of cols) c.classList.add(canDrop(r, c.dataset.col) ? "drop-can" : "drop-no");
  let over = null;
  const move = (e) => {
    ghost.style.transform = `translate(${e.clientX - dx}px, ${e.clientY - dy}px)`;
    const col = document.elementFromPoint(e.clientX, e.clientY)?.closest(".board-col");
    over = col && canDrop(r, col.dataset.col) ? col : null;
    for (const c of cols) c.classList.toggle("drop-ok", c === over);
  };
  const end = (e) => {
    grip.removeEventListener("pointermove", move);
    grip.removeEventListener("pointerup", end);
    grip.removeEventListener("pointercancel", end);
    ghost.remove();
    li.classList.remove("dragging");
    for (const c of cols) c.classList.remove("drop-ok", "drop-can", "drop-no");
    if (e.type === "pointerup" && over) boardDrop(li, r, over);
  };
  grip.addEventListener("pointermove", move);
  grip.addEventListener("pointerup", end);
  grip.addEventListener("pointercancel", end);
  move(ev);
}

/// 카드를 놓은 열로 옮기고 그 상태의 의견 칸을 연다 — 확인하면 바뀌고(새로 고침), 취소면 원래 열 · 자리로 돌아간다.
function boardDrop(li, r, col) {
  const mv = r.moves.find((m) => m.to === col.dataset.col);
  const bar = li.querySelector(".req-acts");
  if (!mv || !bar) return;
  const home = li.parentNode;
  const next = li.nextSibling;
  const list = col.querySelector(".rows");
  list.querySelector(":scope > li.hint")?.remove();
  list.prepend(li);
  if (!home.querySelector(":scope > li[data-req]")) home.append(el("li", { class: "hint", text: "없음" }));
  li.classList.add("pending");
  noteForm(bar, r.id, mv.to, mv.label, () => {
    li.classList.remove("pending");
    home.querySelector(":scope > li.hint")?.remove();
    home.insertBefore(li, next);
    if (!list.querySelector(":scope > li[data-req]")) list.append(el("li", { class: "hint", text: "없음" }));
  });
}

// 보기 바꾸기(#1v) — 보드를 고를 때 개요면 받은 요청 탭으로.
for (const [id, on] of [
  ["req_view_list", false],
  ["req_view_board", true],
])
  $(id).addEventListener("click", () => {
    setFolded(REQ_BOARD, on);
    if (on && state.reqBox == null) state.reqBox = "theirs";
    loadRequests();
  });

/// 요청 개요 한 절(#1q) — 머리 = 이름 · 건수 · 상태별 건수, 안 = 그 쪽 목록('나' 면 상대별 절). 받은 요청은 처음 펼침, 보낸 요청은 처음 접힘.
function reqBoxEl({ box, sum, rows }) {
  const h = reqHead(box === "theirs" ? "받은 요청" : "보낸 요청", sum);
  const head = [el("strong", { text: h.title }), el("span", { class: "stat", text: h.count }), ...h.states.map((x) => el("span", { class: `stat req-${x.key}`, text: x.text }))];
  const body = [el("ol", { class: "rows" }, ...grouped(`requests:${box}`, rows, (rs) => requestEls(rs, box)))];
  return el("li", { class: "fold" }, foldEl(`requests:${box}`, head, body, "peer-fold req-box", null, box === "mine"));
}

/// 요청 행 — 위 링크(이름 · 상태 · 시각 · 대화 / 본문), 바로 아래 상태 바꾸기 글자 단추(#1q — 의견 칸도 이 줄에서), 그 아래 의견 · 알리지 못함.
function requestEls(rows, box) {
  if (!rows.length) return [el("li", { class: "hint", text: box === "theirs" ? "받은 요청 없음" : "보낸 요청 없음" })];
  return rows.map(requestItem);
}

/// 요청 한 줄(목록 행 · 보드 카드 같은 모양, #1v).
function requestItem(r) {
  const li = el(
    "li",
    { "data-req": String(r.id) },
    el(
      "a",
      { href: r.href, draggable: "false" },
      el(
        "span",
        { class: "meta" },
        el("strong", { text: r.who }),
        el("span", { class: r.done ? "tag ask done" : "tag ask", text: r.state }),
        el("span", { text: formatTime(r.at) }),
        el("span", { class: "hint", text: r.title }),
      ),
      el("span", { text: r.preview }),
    ),
  );
  if (r.moves.length || r.resend || r.archive) {
    const bar = el("div", { class: "req-acts" });
    for (const mv of r.moves) bar.append(textButton(mv.label, REQ_BTN[mv.to], () => noteForm(bar, r.id, mv.to, mv.label)));
    if (r.resend) bar.append(textButton("다시 알리기", null, () => resendState(r.id, r.stateKey, r.note)));
    if (r.archive) bar.append(textButton(r.archive.label, null, () => archiveReq(r.id, r.archive.key === "archive")));
    li.append(bar);
  }
  // 의견(받은 쪽이 상태와 함께 남긴 글, 여러 줄 — #1z) — 회색 힌트가 아니라 상태 색 띠의 상자로 읽히게.
  if (r.note) li.append(el("div", { class: `req-feedback req-${r.stateKey}` }, el("span", { class: "req-feedback-label", text: "의견" }), el("span", { text: r.note })));
  if (r.resend) li.append(el("div", { class: "req-notes" }, el("span", { class: "hint", text: "요청자에게 알리지 못함" })));
  return li;
}

/// 글자 단추(#1q) — 테두리 없이 아이콘 + 이름, `look` = `REQ_BTN` 값(없으면 강조색 글자만).
function textButton(label, look, onClick) {
  return el("button", { type: "button", class: look ? `text-btn ${look.tone}` : "text-btn", onclick: onClick }, look ? ico(look.icon) : "", el("span", { text: label }));
}

// 탭(#1q) — 누르면 그 쪽 목록만, 눌린 탭을 다시 누르면 개요.
for (const b of ["theirs", "mine"])
  $(`req_tab_${b}`).addEventListener("click", (ev) => {
    ev.preventDefault();
    state.reqBox = state.reqBox === b && !reqBoard() ? null : b; // 보드에는 개요가 없다(#1v)
    loadRequests();
  });
$("req_state").addEventListener("input", () => loadRequests());
$("req_archive").addEventListener("change", () => loadRequests());

/// 북마크 항목 — 고른 타일 범위, 북마크한 순 최신부터, 누르면 그 글로.
async function loadSaved() {
  const ticket = latest.next();
  let ms;
  try {
    const scope = scopeQuery(state.railPeer, state.railGroup);
    ms = await api("GET", "/api/saved" + (scope ? `?${scope}` : ""));
  } catch (e) {
    $("status_msg").textContent = "북마크 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const rows = inboxRows(ms, state.peerNames).map((r) => ({ ...r, href: `${r.href}/m/${r.id}` }));
  if (needsRender($("saved_list"), JSON.stringify([state.railPeer, state.railGroup, rows]), false))
    $("saved_list").replaceChildren(...grouped("saved", rows, (rs) => rowEls(rs, "북마크 없음")));
}

/// 에이전트 화면 — 고른 타일 범위('나' = 내 에이전트 + 페어링된 상대 전부의 에이전트). 상대 목록은 원격 왕복이라 이 화면을 열 때 없는 것만 받는다.
function loadAgentsView() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const peers = p ? [p] : state.peers.filter((x) => x.paired);
  for (const x of peers) if (!state.peerAgents.has(x.id)) loadPeerAgents(x.id);
  renderAgentsView();
}

function renderAgentsView() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  const peers = p ? [p] : state.peers.filter((x) => x.paired);
  $("agents_title").textContent = p ? `${displayName(p)}의 에이전트` : "등록된 에이전트";
  const row = (name, sub, props) =>
    el(
      "li",
      {},
      el(
        "a",
        props,
        el("span", { class: "meta" }, el("span", { class: "agent-dot", "aria-hidden": "true" }), el("strong", { class: "agent-name", text: name })),
        sub ? el("span", { class: "hint", text: sub }) : "",
      ),
    );
  const peerRows = (x) => {
    const names = state.peerAgents.get(x.id);
    if (names === undefined) return [el("li", { class: "hint", text: `${displayName(x)} · 불러오는 중 …` })];
    if (names === null) return [el("li", { class: "hint", text: `${displayName(x)} · 목록을 받지 못했습니다` })];
    // 줄 오른쪽 "세션 생성" 단추(#1o) — 줄을 눌러도 같다.
    return names.map((n) => {
      const li = row(n, null, {
        href: "#new",
        onclick: (ev) => {
          ev.preventDefault();
          newChat(x.id, n);
        },
      });
      li.classList.add("has-edit");
      li.append(el("button", { type: "button", class: "small agent-new", "aria-label": `${SESSION_NEW} · ${n}`, text: SESSION_NEW, onclick: () => newChat(x.id, n) }));
      return li;
    });
  };
  const none = [el("li", { class: "hint", text: "에이전트 없음" })];
  if (p) {
    const rows = peerRows(p);
    $("agents_list").replaceChildren(...(rows.length ? rows : none));
    return;
  }
  // '나' = 절: 내 에이전트(맨 앞) · 상대마다 '{이름}의 에이전트'(#1g) — 에이전트가 없는 상대는 숨긴다(불러오는 중 · 실패는 보인다).
  // 내 에이전트(#1k) — 줄 오른쪽 연필 = 설정 '내 에이전트' 의 그 에이전트를 펼쳐서(줄을 눌러도 같다).
  const mine = state.agents.map((a) => {
    const li = row(a.name, `내 에이전트 · ${a.folder}`, {
      href: "#settings/agents",
      onclick: (ev) => {
        ev.preventDefault();
        editAgent(a.id);
      },
    });
    const pen = el("button", { type: "button", class: "icon-btn agent-edit", title: "편집", "aria-label": `${a.name} 편집`, onclick: () => editAgent(a.id) });
    pen.innerHTML = PENCIL_SVG;
    li.classList.add("has-edit");
    li.append(pen);
    return li;
  });
  const sections = [
    ...(mine.length ? [["agents:me", `내 에이전트 · ${mine.length}`, mine]] : []),
    ...peers.map((x) => {
      const names = state.peerAgents.get(x.id);
      return [`agents:${x.id}`, `${displayName(x)}의 에이전트${Array.isArray(names) ? ` · ${names.length}` : ""}`, peerRows(x)];
    }),
  ].filter(([, , rows]) => rows.length);
  $("agents_list").replaceChildren(
    ...(sections.length ? sections.map(([key, title, rows]) => el("li", { class: "fold" }, foldEl(key, [title], [el("ol", { class: "rows" }, ...rows)]))) : none),
  );
}

/// 내 에이전트 편집(#1k) — 설정 '내 에이전트' 로 가서 그 에이전트 상세를 펼치고 "관리" 단추에 초점.
function editAgent(id) {
  state.open.add("agent" + id);
  renderAgentList(true);
  goSettings("agents", `agent${id}_manage`);
}

/// 주고받은 메시지(#1q, '나' 에만) — 상대별 절(레일 순서) 처음 접힘, 머리 = 타일 · 이름 · 건수 · 마지막 메시지 한 줄 · 시각. 펼치면 그 상대 글을
/// 20건씩(서버 페이지), 끝 줄 "더 보기" 로 이어 받는다. 다시 그릴 때(5초 새로 고침)는 요약을 받고, 펼친 절은 보인 수만큼 다시 받는다.
async function loadPeerMessages() {
  $("peer_title").textContent = "주고받은 메시지";
  const ticket = latest.next();
  let sums;
  try {
    // 기간은 다른 창에서 바뀌었을 수 있다 — 서버가 거른 기준(설정)을 함께 다시 읽어 받아 둔 꼬리에도 같은 기간을(#1y 적대적 평가 R2).
    let s;
    [sums, s] = await Promise.all([api("GET", "/api/messages/peers"), api("GET", "/api/settings")]);
    if (s.ui_peer_weeks !== undefined && s.ui_peer_weeks !== state.settings.ui_peer_weeks) {
      state.settings.ui_peer_weeks = s.ui_peer_weeks;
      $("ui_peer_weeks").value = String(s.ui_peer_weeks);
    }
  } catch (e) {
    $("status_msg").textContent = "주고받은 메시지 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  state.peerHeads = peerSummaryRows(sums, pairedIds(), state.peerNames);
  for (const h of state.peerHeads) if (state.folded.has(`open:peer:${h.peer}`)) await loadPeerPage(h.peer);
  renderPeerMessages();
}

/// 한 상대의 글 — `more` 면 받은 마지막 글(`(created_at, id)` 커서) 앞의 20건을 이어 붙이고, 아니면 첫 페이지를 다시 받아 받은 꼬리와 합친다(`mergePage`).
/// 상대마다 한 줄로 차례를 지킨다(R2 — 새로 고침과 "더 보기" 가 겹쳐 늦게 끝난 쪽이 다른 쪽 결과를 지우지 않게).
function loadPeerPage(peer, more = false) {
  const run = async () => {
    const have = state.peerPages.get(peer) ?? [];
    const end = have.at(-1);
    const q = more && end ? `&limit=${PEER_PAGE}&before=${end.id}&before_at=${end.created_at}` : `&limit=${PEER_PAGE}`;
    try {
      const ms = await api("GET", `/api/messages?side=both&peer=${peer}${q}`);
      const now = state.peerPages.get(peer) ?? [];
      state.peerPages.set(peer, more ? [...now, ...ms.filter((m) => !now.some((x) => x.id === m.id))] : mergePage(ms, now, PEER_PAGE, peerSince()));
    } catch (e) {
      $("status_msg").textContent = "주고받은 메시지 오류: " + e.message;
    }
  };
  const next = (state.peerLoads.get(peer) ?? Promise.resolve()).then(run);
  state.peerLoads.set(peer, next);
  return next;
}

/// 주고받은 메시지 보기 기간의 시작(#1y — 서버 `peer_since` 와 같은 셈, 유닉스 초).
function peerSince() {
  return Math.floor(Date.now() / 1000) - (Number(state.settings.ui_peer_weeks) || 4) * 7 * 86400;
}

function renderPeerMessages() {
  const heads = state.peerHeads;
  if (!heads.length) {
    $("peer_list").replaceChildren(el("li", { class: "hint", text: "주고받은 메시지 없음" }));
    return;
  }
  if (!needsRender($("peer_list"), JSON.stringify([peerMsgsSig(heads, state.peerPages), state.peerNames.size]), false)) return;
  $("peer_list").replaceChildren(
    ...heads.map((h) => {
      const p = state.peers.find((x) => x.id === h.peer);
      const head = [
        el("span", { class: `row-tile ${colorOf(h.peer)}`, "aria-hidden": "true" }, tileFace(p?.icon)),
        el("strong", { text: h.name }),
        el("span", { class: "stat", text: h.count }),
        el("span", { class: "peer-last", text: h.last }),
        el("span", { class: "hint", text: formatTime(h.at) }),
      ];
      const page = state.peerPages.get(h.peer);
      const body = page ? [el("ol", { class: "rows" }, ...peerRowEls(inboxRows(page, state.peerNames))), moreRow(h, page.length)] : [el("p", { class: "hint more-row", text: "불러오는 중 …" })];
      const d = foldEl(`peer:${h.peer}`, head, body, "peer-fold peer-msgs", null, true);
      d.addEventListener("toggle", async () => {
        if (!d.open || state.peerPages.has(h.peer)) return;
        await loadPeerPage(h.peer);
        renderPeerMessages();
      });
      return el("li", { class: "fold" }, d);
    }),
  );
}

/// 펼친 절 끝 줄 — "{보인 수} / {전체}" + 남았으면 "더 보기"(20건).
function moreRow(h, shown) {
  const m = pageMore(shown, h.total);
  const more = async () => {
    await loadPeerPage(h.peer, true);
    renderPeerMessages();
  };
  return el("div", { class: "more-row" }, el("span", { text: m.text }), m.more ? button("더 보기", more, "small") : "");
}

/// 주고받은 메시지 행(#1z-2 — 시안 `Peer`) — 원형 아바타(상대 사람 · 나 · 에이전트, `avatarKind`) · 메타(이름 · "사람" / 에이전트 이름 + "에이전트" 칩 · 대화 제목 ·
/// 요청 태그) · 본문 · 오른쪽 시각, 에이전트 줄은 옅은 에이전트 색 바탕. 오른쪽 끝 북마크 아이콘(#1q — 말풍선과 같은 그림 · 동작).
function peerRowEls(rows) {
  if (!rows.length) return [el("li", { class: "hint", text: "주고받은 메시지 없음" })];
  return rows.map((r) => {
    const kind = avatarKind(r.author, r.mine);
    const s = speakerIcon(r.author);
    const li = el(
      "li",
      { class: `has-mark pm-row pm-${kind}` },
      el(
        "a",
        { href: r.href, class: "pm-link" },
        el("span", { class: `pm-av pm-av-${kind}`, role: "img", "aria-label": s.label, title: s.label }, ico(s.icon)),
        el(
          "span",
          { class: "pm-text" },
          el(
            "span",
            { class: "meta" },
            el("strong", { class: kind === "agent" ? "pm-agent-name" : null, text: r.who }),
            kind === "agent" ? el("span", { class: "tag agent-tag", text: "에이전트" }) : el("span", { text: "사람" }),
            r.room ? el("span", { class: "hint", text: r.room }) : "",
            r.kind ? el("span", { class: "tag ask", text: r.kind }) : "",
          ),
          el("span", { class: "pm-body", text: r.preview }),
        ),
        el("span", { class: "pm-time", text: formatTime(r.at) }),
      ),
    );
    const b = iconButton(BOOKMARK_SVG, r.saved ? "북마크 해제" : "북마크", () => markRow(r));
    b.classList.add("bookmark", "row-mark");
    if (r.saved) b.classList.add("on");
    b.setAttribute("aria-pressed", String(r.saved));
    li.append(b);
    return li;
  });
}

/// 행 북마크 넣기 · 빼기(#1q) — 말풍선 북마크와 같은 `PATCH saved`, 받은 글 사본 · 열린 대화 글도 같이 바꾼다.
function markRow(r) {
  const on = !r.saved;
  act(on ? "북마크" : "북마크 해제", async () => {
    await api("PATCH", `/api/messages/${r.id}`, { saved: on });
    for (const ms of state.peerPages.values()) for (const m of ms) if (m.id === r.id) m.saved = on;
    const open = state.msgs.find((m) => m.id === r.id);
    if (open) open.saved = on;
    renderPeerMessages();
    return on ? "북마크했습니다" : "북마크 해제";
  });
}

// ---- 세션 생성 ----

function fillPeerSelects() {
  // 고르는 중인 칸을 5초마다 다시 만들지 않게 — 그리는 값(상대 · 페어링 · 이름)이 같으면 둔다.
  if (unchanged($("chat_peer"), JSON.stringify(state.peers.map((p) => [p.id, p.paired, displayName(p), p.host])))) return;
  const paired = state.peers.filter((p) => p.paired);
  const sel = $("chat_peer");
  const keep = sel.value;
  sel.replaceChildren(
    el("option", { value: "", text: paired.length ? "상대 선택" : "페어링된 상대 없음" }),
    ...paired.map((p) => el("option", { value: p.id, text: `${displayName(p)} · ${p.host}` })),
  );
  if (paired.some((p) => String(p.id) === keep)) sel.value = keep;
}

const agentsAsk = new Latest();
/// 세션 생성의 상대 에이전트 — 고른 상대 PC 에 묻는다. `want` 가 그 목록에 있으면 그것을 고른다.
async function loadChatAgents(want) {
  const id = $("chat_peer").value;
  const sel = $("chat_agent");
  sel.replaceChildren();
  const ticket = agentsAsk.next();
  if (!id) return;
  $("chat_msg").textContent = "상대 에이전트 불러오는 중 …";
  try {
    const r = await api("GET", `/api/peers/${id}/agents`);
    if (!agentsAsk.is(ticket)) return; // 그 사이 다른 상대를 골랐다
    sel.replaceChildren(...r.names.map((n) => el("option", { value: n, text: n })));
    if (want && r.names.includes(want)) sel.value = want;
    $("chat_msg").textContent = r.names.length ? "" : "상대 PC 에 등록된 에이전트가 없습니다";
  } catch (e) {
    $("chat_msg").textContent = "상대 에이전트 목록 실패: " + e.message;
  }
}
$("chat_peer").addEventListener("change", () => loadChatAgents());

/// 세션 생성 '최대 응답 횟수'(#1r) — '내 응답 에이전트' 를 고를 때만 보이는 하위 줄. 숨긴 동안은 `disabled` 라 폼 검사에서 빠진다
/// (숨긴 칸의 범위 밖 값이 보내기를 소리 없이 막지 않게 — 적대적 평가 R1), 보일 때는 `required` · 1 ~ 50 정수를 브라우저가 검사한다.
function syncRelayCap() {
  const off = !$("chat_relay_agent").value;
  $("chat_relay_cap").hidden = $("chat_relay_cap_label").hidden = $("chat_relay_cap").disabled = off;
}
$("chat_relay_agent").addEventListener("change", syncRelayCap);

/// 세션 생성 화면을 열 때(#1x) — 상대 칸이 비었으면 레일에서 고른 상대로 채우고 그 에이전트 목록을 읽는다.
function openCompose() {
  if ($("chat_peer").value) return;
  const want = composeDefaults(state.railPeer, state.peers);
  if (!want) return;
  $("chat_peer").value = want;
  loadChatAgents();
}

// 세션 생성(#1x) — 글 없이 대화방만 만들고 그 대화방으로(첫 글은 대화방 입력창에서, 질의 · 요청도 거기서).
$("chat_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = {
    peer_id: Number($("chat_peer").value),
    agent: $("chat_agent").value,
    title: $("chat_title").value,
  };
  if (!body.peer_id || !body.agent) {
    $("chat_msg").textContent = "상대와 상대 에이전트를 고르세요";
    return;
  }
  if ($("chat_relay_agent").value) {
    body.relay_agent = $("chat_relay_agent").value;
    body.relay_cap = $("chat_relay_cap").valueAsNumber;
  }
  // 그룹을 고른 채 만든 대화는 그 그룹에(#1i).
  if (state.railGroup != null) body.group = state.railGroup;
  act(
    "생성",
    async () => {
      const r = await api("POST", "/api/conversations", body);
      $("chat_title").value = "";
      if (body.group != null) await loadGroups();
      await loadConvs();
      state.focusAfter = "reply_text";
      location.hash = `#c/${r.conversation.id}`;
      return "대화를 만들었습니다 — 메시지를 입력하세요";
    },
    "chat_msg",
  );
});

// ---- 레일 그룹(#1i) — 떠 있는 상자(메뉴 · 이름 · 대화 추가) ----

let floatFrom = null; // 떠 있는 상자를 연 요소 — 닫으면 초점을 돌려준다

/// 떠 있는 상자 — `anchor` 옆(레일이면 오른쪽, 그 밖은 아래)에 연다. 레일은 넘치면 잘리므로 화면 기준(`position: fixed`)으로 둔다.
function openFloat(anchor, role, label, children) {
  const box = $("float");
  box.replaceChildren(...children);
  Object.assign(box, { hidden: false });
  box.setAttribute("role", role);
  box.setAttribute("aria-label", label);
  floatFrom = anchor;
  const r = anchor.getBoundingClientRect();
  const side = anchor.closest("#rail") != null;
  const w = box.offsetWidth;
  const h = box.offsetHeight;
  const left = side ? r.right + 8 : Math.min(r.left, window.innerWidth - w - 8);
  const top = side ? r.top : r.bottom + 4;
  box.style.left = Math.max(8, left) + "px";
  box.style.top = Math.max(8, Math.min(top, window.innerHeight - h - 8)) + "px";
  box.querySelector("input, button")?.focus();
}

/// 상자 닫기 — `refocus` 면 연 요소로(다시 그려져 없으면 '나' 타일로) 초점을 돌린다. 바깥을 눌러 닫을 때는 누른 곳에 초점이 간다(돌리지 않음).
function closeFloat(refocus) {
  $("float").hidden = true;
  $("float").replaceChildren();
  if (refocus) (floatFrom?.isConnected ? floatFrom : $("rail_me")).focus();
  floatFrom = null;
}

/// 작업 뒤 초점 — 다시 그린 새 요소(`sel`)가 있으면 거기, 없으면 `fallback`(라운드 1 — 키보드 사용자가 위치를 잃지 않게).
function focusAfterFloat(sel, fallback = "rail_me") {
  (document.querySelector(sel) ?? $(fallback))?.focus();
}

/// "그룹 +" — 이름 한 칸 → 만들고 그 그룹을 고른다. `after` 가 있으면(#1j — 그룹설정 "+ 그룹 추가" · 상대 "+ 새 그룹에 넣기…") 고르지 않고 그것을
/// 부른다. 거절 문구(빈 이름 · 40자 넘음 · 같은 이름)는 상자 안에.
function openGroupNew(anchor, after = null) {
  // 길이는 `groupNameError`(글자 = 코드 포인트, 서버와 같음)로 — HTML `maxlength` 는 UTF-16 단위라 그림 문자 이름을 20자 안팎에서 막는다(라운드 1).
  const input = el("input", { id: "group_new_name", autocomplete: "off", placeholder: "예: 결제 모듈", "aria-label": "그룹 이름" });
  const msg = el("p", { class: "hint", role: "status" });
  const form = el(
    "form",
    { class: "float-form" },
    el("strong", { text: "새 그룹" }),
    input,
    el("p", { class: "hint", text: "상대에게 보이지 않는 이 PC 의 대화 묶음입니다." }),
    el("div", { class: "actions" }, button("취소", () => closeFloat(true)), el("button", { type: "submit", class: "primary", text: "만들기" })),
    msg,
  );
  form.addEventListener("submit", async (ev) => {
    ev.preventDefault();
    const bad = groupNameError(input.value, state.groups);
    if (bad) {
      msg.textContent = bad;
      return;
    }
    try {
      const g = await api("POST", "/api/groups", { name: input.value });
      closeFloat(false);
      await loadGroups();
      if (after) {
        await after(g);
        anchor.isConnected ? anchor.focus() : focusAfterFloat(`#rail_groups [data-id="${g.id}"]`);
        return;
      }
      pickGroup(g.id);
      focusAfterFloat(`#rail_groups [data-id="${g.id}"]`);
      $("status_msg").textContent = `그룹 ${g.name} 을 만들었습니다`;
    } catch (e) {
      msg.textContent = e.message;
    }
  });
  openFloat(anchor, "dialog", "새 그룹", [form]);
}

$("rail_group_add").addEventListener("click", (ev) => openGroupNew(ev.currentTarget));

/// 그룹 이름 바꾸기(#1j — 그룹설정 절 머리) — 이름 한 칸, 거절 문구는 상자 안에. 바꾸면 레일 · 그룹설정이 함께.
function openGroupRename(id, anchor) {
  const g = state.groups.find((x) => x.id === id);
  if (!g) return;
  const input = el("input", { value: g.name, autocomplete: "off", "aria-label": "그룹 이름" });
  const msg = el("p", { class: "hint", role: "status" });
  const form = el(
    "form",
    { class: "float-form" },
    el("strong", { text: `그룹 ${g.name} 이름 바꾸기` }),
    input,
    el("div", { class: "actions" }, button("취소", () => closeFloat(true)), el("button", { type: "submit", class: "primary", text: "저장" })),
    msg,
  );
  form.addEventListener("submit", async (ev) => {
    ev.preventDefault();
    const bad = groupNameError(input.value, state.groups, id);
    if (bad) {
      msg.textContent = bad;
      return;
    }
    try {
      await api("PATCH", `/api/groups/${id}`, { name: input.value });
      closeFloat(false);
      await loadGroups();
      $("peer_msg").textContent = `그룹 이름을 바꿨습니다`;
      focusAfterFloat(`.gs-card[data-id="${id}"] .gs-card-head .icon-btn`, "peer_q");
    } catch (e) {
      msg.textContent = e.message;
    }
  });
  openFloat(anchor, "dialog", "그룹 이름 바꾸기", [form]);
}

/// 소속 바꾸기 — 성공이면 참. 어느 쪽이든 그룹을 다시 읽는다, 실패는 상태 줄에(부르는 쪽은 참일 때만 성공 문구 — 라운드 1).
async function changeGroup(id, add, remove) {
  let ok = true;
  try {
    await api("PUT", `/api/groups/${id}/conversations`, { add, remove });
  } catch (e) {
    ok = false;
    $("status_msg").textContent = "그룹 바꾸기 실패: " + e.message;
  }
  await loadGroups();
  return ok;
}

/// 대화 ⋯ 메뉴 — "그룹" ▸ 그룹마다 체크(누를 때마다 넣고 뺌, 메뉴는 열린 채), 그룹을 고른 채면 "그룹에서 빼기".
function openConvMenu(convId, anchor) {
  const items = () => {
    const out = [el("p", { class: "menu-head" }, ico("users"), el("span", { text: "그룹" }))];
    if (!state.groups.length) out.push(el("p", { class: "hint", text: "그룹 없음" }));
    for (const g of state.groups) {
      const on = g.convs.includes(convId);
      out.push(
        el("button", {
          type: "button",
          role: "menuitemcheckbox",
          "aria-checked": String(on),
          onclick: async (ev) => {
            const b = ev.currentTarget;
            await changeGroup(g.id, on ? [] : [convId], on ? [convId] : []);
            if (!$("float").hidden && floatFrom === anchor) {
              $("float").replaceChildren(...items());
              ($("float").querySelector(`[data-g="${g.id}"]`) ?? b).focus?.();
            }
          },
          "data-g": String(g.id),
        }, ico("users"), el("span", { text: g.name })),
      );
    }
    const g = railGroupOf();
    if (g?.convs.includes(convId))
      out.push(
        el("button", {
          type: "button",
          role: "menuitem",
          class: "danger-text",
          onclick: async () => {
            closeFloat(false);
            if (await changeGroup(g.id, [], [convId])) $("status_msg").textContent = `그룹 ${g.name} 에서 뺐습니다`;
            focusAfterFloat("#conv_sections li[data-conv] a", "group_pick_btn");
          },
        }, ico("minus"), el("span", { text: "그룹에서 빼기" })),
      );
    // 기록 삭제 ▸ · 나가기(#1x — D40 개정) — 확인 한 번. 하위 메뉴는 그 자리에서 펼친다(→ 펼침 · ← 접음).
    const conv = state.convs.find((c) => c.id === convId) ?? (state.current?.id === convId ? state.current : null);
    if (conv) {
      out.push(el("hr", { class: "menu-sep" }));
      const confirm = (key) => {
        $("float").replaceChildren(convEndForm(conv, key));
        $("float").setAttribute("role", "dialog");
        $("float").querySelector("button")?.focus();
      };
      for (const it of convMenuItems(conv)) {
        if (!it.sub) {
          out.push(
            el("button", { type: "button", role: "menuitem", class: it.danger ? "danger-text" : null, onclick: () => confirm(it.key) }, ico(it.icon), el("span", { text: it.label })),
          );
          continue;
        }
        const subs = it.sub.map((x) => el("button", { type: "button", role: "menuitem", class: "menu-sub danger-text", hidden: true, onclick: () => confirm(x.key) }, el("span", { text: x.label })));
        const toggle = (open) => {
          head.setAttribute("aria-expanded", String(open));
          for (const b of subs) b.hidden = !open;
          // 펼쳐 길어진 상자가 화면 아래로 넘치면 위로 올린다(`openFloat` 와 같은 여백).
          const box = $("float").getBoundingClientRect();
          if (box.bottom > window.innerHeight - 8) $("float").style.top = Math.max(8, window.innerHeight - box.height - 8) + "px";
          if (open) subs[0].focus();
        };
        const head = el(
          "button",
          {
            type: "button",
            role: "menuitem",
            "aria-haspopup": "menu",
            "aria-expanded": "false",
            onclick: () => toggle(head.getAttribute("aria-expanded") !== "true"),
            onkeydown: (ev) => {
              if (ev.key === "ArrowRight") toggle(true);
            },
          },
          ico(it.icon),
          el("span", { text: it.label }),
          el("span", { class: "kbd", "aria-hidden": "true", text: "▸" }),
        );
        for (const b of subs)
          b.addEventListener("keydown", (ev) => {
            if (ev.key !== "ArrowLeft") return;
            toggle(false);
            head.focus();
          });
        out.push(head, ...subs);
      }
    }
    return out;
  };
  openFloat(anchor, "menu", "대화 메뉴", items());
}

/// 확인 문구(#1x) — 나가기 · 기록 삭제 둘(내 글만 · 모든 사용자). `[머리, 안내, 단추]`.
const END_TEXT = {
  leave: ["대화 나가기", LEAVE_CONFIRM, "나가기"],
  clear_mine: ["기록 삭제 · 내 글만", "이 PC 의 대화 기록을 지웁니다. 대화는 남습니다.", "삭제"],
  clear_all: ["기록 삭제 · 모든 사용자", "나와 상대 PC 의 대화 기록을 모두 지웁니다. 대화는 남습니다.", "삭제"],
};

/// 나가기 · 기록 삭제 확인(#1x). 나가기 = 나가며 이 PC 목록 · 글 삭제(`DELETE` — 지금 삭제), 기록 삭제 = 글만(`POST …/clear`). 지운 대화가 열려 있었으면 '개요'로.
function convEndForm(conv, key) {
  const [head, hint, label] = END_TEXT[key];
  const title = convLabel(conv, state.peerNames).title;
  return el(
    "div",
    { class: "float-form" },
    el("strong", { text: `${head} · ${title}` }),
    el("p", { class: "hint", text: hint }),
    el(
      "div",
      { class: "actions" },
      button("취소", () => closeFloat(true)),
      button(
        label,
        () => {
          closeFloat(false);
          act(
            label,
            async () => {
              if (key === "leave") {
                await api("DELETE", `/api/conversations/${conv.id}`);
                if (state.current?.id === conv.id) {
                  state.current = null;
                  go("#");
                }
                await loadGroups();
                return "대화에서 나갔습니다";
              }
              await api("POST", `/api/conversations/${conv.id}/clear`, { all: key === "clear_all" });
              return key === "clear_all" ? "모든 사용자의 대화 기록을 지웠습니다" : "이 PC 의 대화 기록을 지웠습니다";
            },
            "status_msg",
            refresh,
          );
        },
        "danger",
      ),
    ),
  );
}

$("conv_more_btn").addEventListener("click", (ev) => {
  if (state.current) openConvMenu(state.current.id, ev.currentTarget);
});

/// 그룹 "대화 추가" — 제목 · 상대 검색 + 모든 대화 체크 목록(지금 소속은 체크된 채) → "적용" 으로 넣고 빼기 한 번에.
function openGroupPicker(anchor) {
  const g = railGroupOf();
  if (!g) return;
  const picked = new Set(g.convs);
  const q = el("input", { type: "search", placeholder: "제목 · 상대 검색", "aria-label": "대화 검색", autocomplete: "off" });
  const list = el("ul", { class: "pick-list" });
  const draw = () => {
    const needle = q.value.trim().toLowerCase();
    const rows = sortByPos(state.convs)
      .map((c) => ({ c, label: convLabel(c, state.peerNames) }))
      .filter(({ label }) => !needle || `${label.title} ${label.sub}`.toLowerCase().includes(needle));
    list.replaceChildren(
      ...(rows.length
        ? rows.map(({ c, label }) => {
            const box = el("input", { type: "checkbox", checked: picked.has(c.id) });
            box.addEventListener("change", () => (box.checked ? picked.add(c.id) : picked.delete(c.id)));
            return el("li", {}, el("label", {}, box, el("span", { text: label.title }), el("span", { class: "hint", text: label.sub })));
          })
        : [el("li", { class: "hint", text: "맞는 대화 없음" })]),
    );
  };
  q.addEventListener("input", draw);
  draw();
  const apply = async () => {
    const d = membershipDiff(g.convs, [...picked]);
    closeFloat(true);
    if (!d.add.length && !d.remove.length) return;
    if (await changeGroup(g.id, d.add, d.remove)) $("status_msg").textContent = `그룹 ${g.name} · 넣음 ${d.add.length} · 뺌 ${d.remove.length}`;
    focusAfterFloat("#group_pick_btn");
  };
  openFloat(anchor, "dialog", `그룹 ${g.name} 에 대화 추가`, [
    el("div", { class: "float-form picker" }, el("strong", { text: `대화 추가 · ${g.name}` }), q, list, el("div", { class: "actions" }, button("취소", () => closeFloat(true)), button("적용", apply, "primary"))),
  ]);
}

// ---- 손으로 놓는 순서(#1i) — 끌어 놓기 · Alt + ↑ / ↓. 같은 묶음(상대 타일 · 그룹 타일 · 같은 절의 대화) 안에서만 ----

/// 요소 → 끌기 정보(`data-drag` · `data-id` · `data-section`).
function dragInfo(node) {
  const n = node?.closest?.("[data-drag]");
  return n ? { node: n, kind: n.dataset.drag, id: Number(n.dataset.id), section: n.dataset.section ?? null } : null;
}

/// 전체 순서(id) — 상대 = 그룹설정 순서(서버 자리 순), 그룹 = 레일 순서, 대화 = 자리 순.
function orderOf(kind) {
  if (kind === "peer") return state.peers.map((p) => p.id);
  if (kind === "group") return state.groups.map((g) => g.id);
  return sortByPos(state.convs).map((c) => c.id);
}

/// 종류별 순서 저장 — 한 번에 하나, 보내는 동안 바뀌면 마지막 순서만 이어서(`SendLatest`). `last` = 마지막으로 놓은 순서(저장이 끝나기 전 ·
/// 그 사이 시작한 목록 읽기에는 이것을 덮어 씌운다 — 늦게 온 옛 응답이 방금 옮긴 자리를 되돌리지 않게, 라운드 1). `stamp` = 옮김 · 저장 끝마다 하나씩.
const orderSaves = {};

function orderSave(kind) {
  if (orderSaves[kind]) return orderSaves[kind];
  const o = { last: null, stamp: 0 };
  o.sender = new SendLatest(async (ids) => {
    try {
      await api("PUT", "/api/order", { kind: { peer: "peers", group: "groups", conv: "conversations" }[kind], ids });
    } catch (e) {
      $("status_msg").textContent = "자리 저장 실패: " + e.message;
      if (o.last === ids) o.last = null; // 서버 값으로 되돌린다
    }
    o.stamp++;
    if (kind === "peer") await loadPeers();
    else if (kind === "group") await loadGroups();
    else await loadConvs();
  });
  orderSaves[kind] = o;
  return o;
}

/// 목록 읽기를 시작할 때의 표 — 응답 때 `keepOrder` 에 준다.
function orderStamp(kind) {
  return orderSaves[kind]?.stamp ?? 0;
}

/// 받은 목록 — 순서 저장이 진행 중이거나 읽는 사이 옮김 · 저장이 있었으면 마지막으로 놓은 순서를 덮어 씌운다.
function keepOrder(kind, items, stamp) {
  const o = orderSaves[kind];
  const last = orderToApply(o && { last: o.last, busy: o.sender.busy, stamp: o.stamp }, stamp);
  if (!last) return items;
  const fixed = withOrder(items, last);
  return kind === "conv" ? fixed : sortByPos(fixed);
}

/// 옮기기 — 화면을 먼저 바꾸고(새 순서를 자리로) 저장한다. 실패하면 서버 값으로 되돌린다.
function moveItem(kind, id, target, after) {
  const order = moveInOrder(orderOf(kind), id, target, after);
  if (kind === "peer") state.peers = sortByPos(withOrder(state.peers, order));
  else if (kind === "group") state.groups = sortByPos(withOrder(state.groups, order));
  else state.convs = withOrder(state.convs, order);
  renderRail();
  renderConvList();
  if (route(location.hash).view === "agentConvs") renderAgentConvs();
  refreshHome();
  const o = orderSave(kind);
  o.last = order;
  o.stamp++;
  o.sender.push(order);
}

/// 끌기 표시 지우기.
function clearDrop() {
  for (const n of document.querySelectorAll(".drop-before, .drop-after, .drop-into")) n.classList.remove("drop-before", "drop-after", "drop-into");
}

/// 대화 줄을 그룹 타일에 넣는다(복사 — 원래 자리 그대로) · 안내.
function joinGroup(groupId, convId) {
  const g = state.groups.find((x) => x.id === groupId);
  const already = g?.convs.includes(convId);
  changeGroup(groupId, [convId], []).then((ok) => {
    if (ok) $("status_msg").textContent = already ? `이미 그룹 ${g.name} 에 있습니다` : `그룹 ${g?.name ?? ""} 에 넣었습니다`;
  });
}

/// 대화 줄 끌기(포인터, #1x) — 손잡이를 누른 채 움직이면 줄 사본이 따라오고, 같은 절 줄 위 = 앞 · 뒤 표시, 그룹 타일 위 = 넣기 표시. 떼면 `moveItem` · `joinGroup`.
/// HTML 끌어 놓기(`draggable`)는 브라우저 끌기 확장이 가로챈다(요청 보드와 같다 — 사용자 2026-10-19).
/// `join` = 넣기만(#1z-2 — 그룹설정 대화 줄): 놓는 곳 = 포인터 아래 가장 가까운 그룹(카드 안 대화 줄 위여도 그 카드), 대화 순서는 바꾸지 않는다, 끄는 동안 그룹설정을 다시 그리지 않는다.
function convGrab(ev, li, { join = false } = {}) {
  if (ev.button !== 0) return;
  ev.preventDefault();
  const grip = ev.currentTarget;
  grip.setPointerCapture(ev.pointerId);
  if (join) gsDragging = true;
  const d = dragInfo(li);
  const box = li.getBoundingClientRect();
  const dx = ev.clientX - box.left;
  const dy = ev.clientY - box.top;
  const ghost = li.cloneNode(true);
  ghost.classList.add("board-ghost");
  ghost.style.width = `${box.width}px`;
  document.body.append(ghost);
  li.classList.add("dragging");
  let drop = null;
  const move = (e) => {
    ghost.style.transform = `translate(${e.clientX - dx}px, ${e.clientY - dy}px)`;
    const under = document.elementFromPoint(e.clientX, e.clientY);
    const t = dragInfo(join ? under?.closest?.('[data-drag="group"]') : under);
    const how = t && t.node !== li ? dropKind(d, t) : null;
    clearDrop();
    drop = null;
    if (!how) return;
    const r = t.node.getBoundingClientRect();
    const after = e.clientY > r.top + r.height / 2;
    t.node.classList.add(how === "join" ? "drop-into" : after ? "drop-after" : "drop-before");
    drop = { how, t, after };
  };
  let done = false;
  const end = (e) => {
    if (done) return; // pointerup 뒤 lostpointercapture 가 또 온다 — 한 번만(#1z R1 과 같은 꼴)
    done = true;
    grip.removeEventListener("pointermove", move);
    grip.removeEventListener("pointerup", end);
    grip.removeEventListener("pointercancel", end);
    grip.removeEventListener("lostpointercapture", end);
    ghost.remove();
    li.classList.remove("dragging");
    clearDrop();
    if (join) {
      gsDragging = false;
      if (e.type === "pointerup" && drop?.how === "join") gsJoin(drop.t.id, d.id);
      else renderPeerList(); // 끄는 동안 미룬 다시 그리기
      return;
    }
    if (e.type !== "pointerup" || !drop) return;
    if (drop.how === "join") joinGroup(drop.t.id, d.id);
    else moveItem(d.kind, d.id, drop.t.id, drop.after);
  };
  grip.addEventListener("pointermove", move);
  grip.addEventListener("pointerup", end);
  grip.addEventListener("pointercancel", end);
  grip.addEventListener("lostpointercapture", end);
  move(ev);
}

/// 레일 타일 끌기(#1z — 사용자 · 그룹 순서) — 포인터로(HTML 끌어 놓기는 브라우저 끌기 확장이 가로챈다, 대화 줄 · 요청 보드와 같다).
/// 누른 채 `DRAG_SLOP` 넘게 움직이면 끌기 — 사본이 따라오고 같은 묶음 타일 위 = 앞 · 뒤 표시, 떼면 `moveItem`. 끈 뒤 이어지는 click(타일 고르기)은 먹는다.
function tileGrab(ev) {
  if (ev.button !== 0) return;
  const tile = ev.currentTarget;
  const d = dragInfo(tile);
  if (!d) return;
  tile.setPointerCapture(ev.pointerId);
  const sx = ev.clientX;
  const sy = ev.clientY;
  const box = tile.getBoundingClientRect();
  const across = getComputedStyle($("rail")).flexDirection === "row"; // 좁은 화면 = 가로 레일
  const same = [...$("rail").querySelectorAll(`[data-drag="${d.kind}"]`)];
  let ghost = null;
  let drop = null;
  let done = false;
  const move = (e) => {
    if (!ghost) {
      if (!pastSlop(e.clientX - sx, e.clientY - sy)) return;
      railDragging = true; // 끄는 동안 레일을 다시 그리지 않는다 — 캡처한 타일이 DOM 에서 빠지지 않게(#1z R1)
      ghost = tile.cloneNode(true);
      ghost.classList.add("rail-ghost");
      ghost.style.width = `${box.width}px`;
      document.body.append(ghost);
      tile.classList.add("dragging");
    }
    ghost.style.transform = `translate(${e.clientX - (sx - box.left)}px, ${e.clientY - (sy - box.top)}px)`;
    clearDrop();
    drop = null;
    // 놓을 자리 = 레일 안이면 가장 가까운 같은 묶음 타일(#1z-2 — 타일 사이 틈 · 아래 이름 · 양 끝 밖에 놓아도 된다, `railSlot`).
    const rail = $("rail").getBoundingClientRect();
    if (across ? e.clientY < rail.top || e.clientY > rail.bottom : e.clientX < rail.left || e.clientX > rail.right) return;
    const mid = (n) => {
      const r = n.getBoundingClientRect();
      return across ? r.left + r.width / 2 : r.top + r.height / 2;
    };
    const slot = railSlot(same.map(mid), across ? e.clientX : e.clientY);
    const t = slot && dragInfo(same[slot.index]);
    if (!t || t.id === d.id) return;
    t.node.classList.add(slot.after ? "drop-after" : "drop-before");
    drop = { t, after: slot.after };
  };
  const end = (e) => {
    if (done) return;
    done = true;
    tile.removeEventListener("pointermove", move);
    tile.removeEventListener("pointerup", end);
    tile.removeEventListener("pointercancel", end);
    tile.removeEventListener("lostpointercapture", end);
    if (!ghost) return; // 끌지 않았다 — 그냥 누름(click 이 타일을 고른다)
    railDragging = false;
    ghost.remove();
    tile.classList.remove("dragging");
    clearDrop();
    const eat = (c) => {
      c.stopImmediatePropagation();
      c.preventDefault();
    };
    tile.addEventListener("click", eat, { capture: true, once: true });
    setTimeout(() => tile.removeEventListener("click", eat, { capture: true }), 0);
    if (e.type === "pointerup" && drop) moveItem(d.kind, d.id, drop.t.id, drop.after);
    else renderRail(); // 끄는 동안 미룬 다시 그리기
  };
  tile.addEventListener("pointermove", move);
  tile.addEventListener("pointerup", end);
  tile.addEventListener("pointercancel", end);
  tile.addEventListener("lostpointercapture", end);
}

// 레일 안 HTML 끌기(그림 · 링크 기본 끌기)는 막는다 — 타일 순서 · 대화 넣기 모두 포인터 끌기(#1z R2).
$("rail").addEventListener("dragstart", (ev) => ev.preventDefault());
for (const box of ["rail", "conv_sections", "agent_convs_list"]) {
  // Alt + ↑ / ↓ — 초점이 타일 · 대화 행(손잡이 · 링크 · ⋯)에 있으면 같은 묶음에서 한 칸.
  $(box).addEventListener("keydown", (ev) => {
    if (!ev.altKey || (ev.key !== "ArrowUp" && ev.key !== "ArrowDown")) return;
    const d = dragInfo(ev.target);
    if (!d) return;
    ev.preventDefault();
    stepMove(d, ev.key === "ArrowUp" ? -1 : 1);
  });
}

/// 그 항목을 찾는 선택자 — 대화 행은 같은 절(`data-section`) 안의 것.
function dragSel(d) {
  return `[data-drag="${d.kind}"][data-id="${d.id}"]${d.section != null ? `[data-section="${CSS.escape(d.section)}"]` : ""}`;
}

/// 같은 묶음(보이는 순서)에서 한 칸 옮기기(`dir` -1 = 위) — Alt + ↑ / ↓ · 타일 메뉴(#1j) 공용. 초점은 다시 그린 그 항목에(`keepFocus`), 바뀐 자리를 읽어 준다.
function stepMove(d, dir) {
  const scope = d.kind === "conv" ? $("conv_sections") : $("rail");
  const same = `[data-drag="${d.kind}"]${d.section != null ? `[data-section="${CSS.escape(d.section)}"]` : ""}`;
  const ids = () => [...scope.querySelectorAll(same)].map((n) => Number(n.dataset.id));
  const step = stepTarget(ids(), d.id, dir);
  if (!step) return;
  moveItem(d.kind, d.id, step.target, step.after);
  const moved = scope.querySelector(dragSel(d));
  if (!scope.contains(document.activeElement)) moved?.focus(); // 타일 메뉴에서 옮겼으면 그 타일에
  const name =
    d.kind === "peer"
      ? displayName(state.peers.find((p) => p.id === d.id) ?? {})
      : d.kind === "group"
        ? (state.groups.find((g) => g.id === d.id)?.name ?? "")
        : (moved?.querySelector(".room-title")?.textContent ?? "대화");
  $("order_live").textContent = movedText(name, ids().indexOf(d.id));
}

// ---- 설정(트리 · 실행 · 화면 · 알림) ----

function renderTree() {
  const item = (n) =>
    el(
      "li",
      {},
      el(
        "a",
        {
          href: `#settings/${n.id}`,
          onclick: (ev) => {
            ev.preventDefault();
            goSettings(n.id);
            if (n.press) $(n.press).click();
          },
        },
        ico(n.icon),
        el("span", { text: n.label }),
      ),
      n.children ? el("ul", {}, ...n.children.map(item)) : "",
    );
  $("settings_tree").replaceChildren(el("ul", {}, ...settingsTree().map(item)));
}

/// 설정 입력 — 입력 id = 설정 이름.
const PREFS = [
  "notify_pair",
  "notify_request",
  "notify_reply",
  "notify_relay",
  "notify_preview",
  "relay_cap_default",
  "claude_model",
  "claude_effort",
  "ui_theme",
  "ui_rail_order",
  "ui_tile_size",
  "ui_peer_weeks",
  "notify_sound",
  "autostart",
  "auto_compact",
];

function fillPrefs(s) {
  state.settings = { ...state.settings, ...s };
  for (const key of PREFS) {
    if (s[key] === undefined) continue;
    const input = $(key);
    if (input.type === "checkbox") input.checked = s[key] === true;
    else input.value = String(s[key]);
  }
  if (s.ui_theme !== undefined) applyTheme(s.ui_theme);
  if (s.ui_rail_order !== undefined) applyRailOrder(s.ui_rail_order);
  if (s.ui_tile_size !== undefined) applyTileSize(s.ui_tile_size);
  if (s.notify_paused !== undefined || s.focus !== undefined) $("bell").classList.toggle("paused", s.notify_paused === true || s.focus === true);
  if (s.me_alias !== undefined || s.me_icon !== undefined) {
    renderRail();
    renderSideHead();
  }
  if (s.me_alias !== undefined || s.me_icon !== undefined || s.focus !== undefined) renderAvatar();
  // 세션 생성의 상한 칸 처음 값.
  if (s.relay_cap_default !== undefined) $("chat_relay_cap").value = String(s.relay_cap_default);
}

$("claude_effort").replaceChildren(...EFFORTS.map((e) => el("option", { value: e, text: e })));
$("ui_peer_weeks").replaceChildren(...peerWeeksOptions().map((o) => el("option", { value: o.value, text: o.label })));
$("claude_model").replaceChildren(...MODELS.map((m) => el("option", { value: m, text: m })));

for (const key of PREFS) {
  const input = $(key);
  input.addEventListener("change", () => {
    const value = input.type === "checkbox" ? input.checked : input.type === "number" || "num" in input.dataset ? Number(input.value) : input.value.trim();
    input.disabled = true;
    act("설정", async () => {
      try {
        fillPrefs(await api("PUT", "/api/settings", { [key]: value }));
        if (key === "claude_model" || key === "claude_effort") renderPeerList(); // 상대별 칸의 "기본값(…)" 문구
        if (key === "ui_peer_weeks") {
          state.peerPages.clear(); // 받은 꼬리에 기간 밖 글이 남지 않게(#1y)
          loadPeerMessages();
        }
        return "설정 저장";
      } catch (e) {
        // 저장이 실패하면 서버 값으로 되돌린다 — 화면과 실제 설정이 어긋나지 않게.
        fillPrefs(await api("GET", "/api/settings").catch(() => ({})));
        throw e;
      } finally {
        input.disabled = false;
      }
    });
  });
}

// 동시 실행 상한 — 설정 저장소가 아니라 실행 문(`PUT /api/runs`)에. 실패하면 서버 값으로 되돌린다.
$("max_runs").addEventListener("change", () => {
  const input = $("max_runs");
  input.disabled = true;
  act("동시 실행 상한", async () => {
    try {
      const r = await api("PUT", "/api/runs", { cap: Number(input.value) });
      input.value = String(r.cap);
      state.runs = r;
      renderRuns();
      return `동시 실행 상한 ${r.cap}`;
    } catch (e) {
      input.value = String((await api("GET", "/api/runs").catch(() => ({ cap: "" }))).cap);
      throw e;
    } finally {
      input.disabled = false;
    }
  });
});

async function showSettings() {
  const box = $("notify_inbound");
  try {
    const s = await api("GET", "/api/settings");
    box.checked = s.notify_inbound === true;
    fillPrefs(s);
    // 폭은 시작할 때만 — 다른 설정 저장 응답(fillPrefs)이 막 끈 폭을 이전 값으로 되돌리지 않게.
    setSideWidth(s.ui_side_width ?? 272);
    // 접힘도 시작할 때만(#1g) — 그 뒤는 화면이 바꾼 값이 정본.
    state.folded = new Set(s.ui_folded ?? []);
    state.homeView = s.ui_home_view ?? "preview"; // 개요 보기(#1z-4)도 시작할 때만
    $("max_runs").value = String((await api("GET", "/api/runs")).cap);
  } catch (e) {
    $("status_msg").textContent = "설정 오류: " + e.message;
  }
  box.addEventListener("change", () => {
    const wanted = box.checked;
    // 브라우저 알림 권한은 사용자 동작(이 클릭) 안에서만 물을 수 있다. 거부 · 미지원이어도 설정은 저장하고 트레이 표시만 쓴다고 알린다.
    const permission =
      wanted && "Notification" in window
        ? Notification.permission === "default"
          ? Notification.requestPermission()
          : Promise.resolve(Notification.permission)
        : Promise.resolve(wanted ? "unsupported" : "off");
    box.disabled = true;
    act("알림 설정", async () => {
      try {
        const r = await api("PUT", "/api/settings", { notify_inbound: wanted });
        box.checked = r.notify_inbound === true;
        if (!box.checked) return "알림을 껐습니다";
        const p = await permission;
        return p === "granted"
          ? "알림을 켰습니다"
          : "알림을 켰습니다. 브라우저 알림을 쓸 수 없어 트레이에만 표시합니다.";
      } catch (e) {
        // 저장이 실패하면 서버 값으로 되돌린다 — 화면과 실제 알림 상태가 어긋나지 않게.
        box.checked = await api("GET", "/api/settings")
          .then((s) => s.notify_inbound === true)
          .catch(() => !wanted);
        throw e;
      } finally {
        box.disabled = false;
      }
    });
  });
}

// ---- 새로 고침 · 알림 스트림 ----

/// `only` = 다시 읽을 로더 이름 집합(`loadersFor`), 없으면 전부. 한 번에 하나만 돈다 — 도는 중에 온 요청은 합쳐 끝난 뒤 한 번 더
/// (먼저 시작한 오래된 조회가 나중 조회를 덮지 않게). 기다리는 쪽은 자기 요청이 반영된 뒤 풀린다.
const refresher = new OneAtATime((only) => refreshOnce(only));
function refresh(only) {
  return refresher.call(only ?? null);
}

async function refreshOnce(only) {
  const want = (k) => !only || only.has(k);
  if (want("peers")) await loadPeers();
  if (want("groups")) await loadGroups();
  if (want("convs")) await loadConvs();
  const r = route(location.hash);
  if (state.current && r.view === "conversation" && want("current")) await openConversation(state.current);
  if (r.view === "activity" && want("activity")) await loadActivity();
  if (r.view === "requests" && want("requests")) await loadRequests();
  if (r.view === "saved" && want("saved")) await loadSaved();
  if (r.view === "peer" && want("peer")) await loadPeerMessages(r.id);
  if (!$("detail").hidden && want("runs")) await loadRuns();
  if (r.view === "settings" && want("settings")) await showMe(); // 열린 피어 연결 수
  if (want("counts")) await loadCounts();
}

/// 변경 신호를 잠깐 모아 한 번만 다시 읽는다(연달아 온 글 · 진행).
let changeQ;
function onChange(res) {
  const first = changeQ === undefined;
  changeQ = mergeChanges(changeQ, res);
  if (!first) return;
  setTimeout(() => {
    const q = changeQ;
    changeQ = undefined;
    refresh(loadersFor(q, state.current?.id)).catch(refreshFailed);
  }, 50);
}

/// 변경 신호 다시 읽기의 실패 — 기다리는 쪽이 없어 상태 줄에 보인다. 다음 신호 · 5초가 다시 읽는다.
function refreshFailed(e) {
  $("status_msg").textContent = "새로 고침 실패: " + e.message;
}

/// 페어링 종류 알림 — 누르면 설정의 그룹설정, 그 밖(메시지 · 자동 왕복)은 내 활동.
const PAIRING_KINDS = new Set(["pair_request", "auto_paired", "approved"]);

/// 받은 요청 효과음(#1x) — 짧은 두 음(WebAudio, 파일 없음 — 880 Hz → 1320 Hz 각 0.12 초). 브라우저는 사용자 입력 전에는 소리를 막는다(autoplay) — 첫 입력에서 만든다.
let audio = null;
for (const type of ["pointerdown", "keydown"])
  addEventListener(type, () => (audio ??= new AudioContext()), { once: true, capture: true });

function chime() {
  if (!audio) return;
  audio.resume?.();
  const t = audio.currentTime;
  [880, 1320].forEach((hz, i) => {
    const at = t + i * 0.12;
    const o = audio.createOscillator();
    const g = audio.createGain();
    o.frequency.value = hz;
    g.gain.setValueAtTime(0.15, at);
    g.gain.exponentialRampToValueAtTime(0.001, at + 0.12);
    o.connect(g).connect(audio.destination);
    o.start(at);
    o.stop(at + 0.12);
  });
}

/// 서버 → 웹UI 알림 스트림. 붙는 순간 트레이 "새 요청" 이 지워진다. 원격 이름은 텍스트로만(브라우저 알림 본문도 텍스트).
function listen() {
  const es = new EventSource(`/api/events?t=${encodeURIComponent(token)}`);
  // 붙은 뒤 "봤다" 를 알린다(트레이 새 요청 지움) — 상태 변경은 POST 로만.
  es.onopen = () => {
    api("POST", "/api/seen").catch(() => {});
    // 끊긴 동안의 변경 신호는 다시 오지 않는다 — 붙을 때마다 미리보기 글을 전부 새로(#1z-4 R2).
    state.previewAll = true;
    refreshHome();
  };
  es.onmessage = (ev) => {
    let n;
    try {
      n = JSON.parse(ev.data);
    } catch {
      return;
    }
    $("status_msg").textContent = `${n.title} — ${n.body.split("\n")[0]}`;
    // 다시 읽기는 변경 신호(`change`)가 맡는다 — 알림은 설정에 따라 오지 않을 수 있다.
    // 받은 요청(#1x — B안) = 같은 tag 한 장(앞 알림을 바꾼다) · 누르면 요청 보드 · 효과음은 웹UI(OS 기본음 대신 — 알림은 무음).
    const nn = requestNotice(n);
    if ($("notify_inbound").checked && "Notification" in window && Notification.permission === "granted") {
      const shown = new Notification(nn.title, { body: nn.body, tag: nn.tag, silent: nn.sound });
      shown.onclick = () => {
        window.focus();
        if (PAIRING_KINDS.has(n.kind)) {
          go("#settings/peers");
          return;
        }
        pickRail(null); // 내 활동 · 요청 전부
        if (nn.href === "#requests") {
          setFolded(REQ_BOARD, true);
          state.reqBox ??= "theirs";
        }
        go(nn.href);
      };
    }
    if (nn.sound && state.settings.notify_sound !== false && !state.settings.focus) chime();
  };
  // 변경 신호 — 알림 설정 · 집중과 무관하게 온다(D39 ②). 읽지 못하면 전체.
  es.addEventListener("change", (ev) => {
    let c;
    try {
      c = JSON.parse(ev.data);
    } catch {
      c = {};
    }
    // 개요 미리보기(#1z-4) — 신호가 가리킨 대화(전체면 전부)는 다음 그리기에서 다시 읽는다.
    const stale = previewStaleOf(c.res);
    if (stale == null) state.previewAll = true;
    else for (const id of stale) state.previewStale.add(id);
    onChange(c.res);
  });
}

// ---- 대화 목록 폭(끌기 · ← → 키) — 이 PC 설정에 저장한다. 웹UI 포트가 실행마다 바뀌어(출처가 달라져) 브라우저 저장소로는
// 다음 실행에 이어지지 않는다.

const SIDE_W = { min: 200, max: 560, step: 16 };
let sideSave;

function setSideWidth(w) {
  const v = Math.min(SIDE_W.max, Math.max(SIDE_W.min, Math.round(w)));
  $("sidebar").style.setProperty("--side-w", v + "px");
  $("side_resize").setAttribute("aria-valuenow", String(v));
  return v;
}

/// 폭을 바꾸고 잠시 뒤 저장 — 키를 누르고 있어도 마지막 값 한 번.
function saveSideWidth(w) {
  const v = setSideWidth(w);
  clearTimeout(sideSave);
  sideSave = setTimeout(() => {
    api("PUT", "/api/settings", { ui_side_width: v }).catch((e) => {
      $("status_msg").textContent = "목록 폭 저장 실패: " + e.message;
    });
  }, 300);
}

$("side_resize").addEventListener("pointerdown", (ev) => {
  ev.preventDefault(); // 끄는 동안 글자 선택 안 함
  const start = ev.clientX;
  const from = $("sidebar").getBoundingClientRect().width;
  const move = (e) => setSideWidth(from + e.clientX - start);
  window.addEventListener("pointermove", move);
  window.addEventListener(
    "pointerup",
    () => {
      window.removeEventListener("pointermove", move);
      saveSideWidth($("sidebar").getBoundingClientRect().width);
    },
    { once: true },
  );
});
$("side_resize").addEventListener("keydown", (ev) => {
  const d = { ArrowLeft: -SIDE_W.step, ArrowRight: SIDE_W.step }[ev.key];
  if (!d) return;
  ev.preventDefault();
  saveSideWidth($("sidebar").getBoundingClientRect().width + d);
});

// ---- 레일 폭(#1z — 손잡이 끌기 · ← → 키) — 끄는 동안 폭이 따라가고 120px 기준으로 아이콘이 바뀐다. 놓으면 크게(72px) · 작게(168px) 중
// 가까운 쪽으로 맞추고 설정 "타일 크기" 에 저장한다(설정 화면 선택도 같이).

function saveTileSize(size) {
  applyTileSize(size);
  $("ui_tile_size").value = size;
  api("PUT", "/api/settings", { ui_tile_size: size }).catch((e) => {
    $("status_msg").textContent = "타일 크기 저장 실패: " + e.message;
  });
}

// 손잡이에 포인터 캡처 — 끝은 pointerup(저장) · pointercancel · lostpointercapture(되돌림) 어느 쪽이든 한 번만 정리한다(#1z R1 — 취소 뒤 폭이 계속 바뀌지 않게).
$("rail_resize").addEventListener("pointerdown", (ev) => {
  if (ev.button !== 0) return;
  ev.preventDefault(); // 끄는 동안 글자 선택 안 함
  const h = ev.currentTarget;
  h.setPointerCapture(ev.pointerId);
  const start = ev.clientX;
  const from = $("rail").getBoundingClientRect().width;
  const before = $("app").classList.contains("tile-small") ? "small" : "normal";
  let w = from;
  let done = false;
  const move = (e) => {
    w = Math.min(RAIL_W.max, Math.max(RAIL_W.min, from + e.clientX - start));
    $("app").style.setProperty("--rail-w", w + "px");
    applyTileSize(railSize(w));
  };
  const end = (e) => {
    if (done) return;
    done = true;
    for (const [type, fn] of handlers) h.removeEventListener(type, fn);
    $("app").style.removeProperty("--rail-w");
    if (e.type === "pointerup") saveTileSize(railSize(w));
    else applyTileSize(before);
  };
  const handlers = [
    ["pointermove", move],
    ["pointerup", end],
    ["pointercancel", end],
    ["lostpointercapture", end],
  ];
  for (const [type, fn] of handlers) h.addEventListener(type, fn);
});
$("rail_resize").addEventListener("keydown", (ev) => {
  const size = { ArrowLeft: "normal", ArrowRight: "small" }[ev.key];
  if (!size) return;
  ev.preventDefault();
  saveTileSize(size);
});

// ---- 상단 바 — + 새로 만들기 · 프로필 메뉴 · 알림(종) · 통합 검색 초점, 실행 상세 열고 닫기 ----

const MENUS = [
  ["new_btn", "new_menu"],
  ["avatar_btn", "profile_menu"],
];

function closeMenus() {
  for (const [b, m] of MENUS) {
    $(m).hidden = true;
    $(b).setAttribute("aria-expanded", "false");
  }
}

function toggleMenu(b, m) {
  const open = $(m).hidden;
  closeMenus();
  if (!open) return;
  $(m).hidden = false;
  $(b).setAttribute("aria-expanded", "true");
  $(m).querySelector("button")?.focus();
}

$("new_menu").replaceChildren(
  ...newMenu().map((i) =>
    el(
      "button",
      {
        type: "button",
        role: "menuitem",
        onclick: () => {
          closeMenus();
          if (i.focus === "agent_name") showAgentNew(true);
          state.focusAfter = i.focus;
          go(i.href);
          if (i.press) $(i.press).click();
        },
      },
      ico(i.icon),
      el("span", { text: i.label }),
    ),
  ),
);
$("new_btn").addEventListener("click", () => toggleMenu("new_btn", "new_menu"));

/// 집중 켜고 끄기 — 켜 있는 동안 상대 PC 의 새 요청을 받지 않고 알림을 멈춘다(알림 일시 중지 값은 그대로).
function setFocus(on) {
  act("집중", async () => {
    fillPrefs(await api("PUT", "/api/settings", { focus: on }));
    return on ? "집중으로 바꿨습니다" : "받기 가능으로 바꿨습니다";
  });
}

/// 프로필 메뉴(#1j) — 머리 "상태" · 받기 가능 / 집중(하나만) · 앱과 연결 끊김(자동 표시 — 단추 아님) | 그 밖. 켜짐은 설정 값으로(열 때마다).
function renderProfileMenu() {
  $("profile_menu").replaceChildren(
    el("div", { class: "menu-head", text: "상태" }),
    ...profileMenu(state.settings.notify_paused, state.settings.focus, appOffline()).map((i) => {
      if (i.sep) return el("hr", { class: "menu-sep" });
      const parts = [i.state ? el("span", { class: `st-dot st-${i.state}`, "aria-hidden": "true" }) : i.icon ? ico(i.icon) : "", el("span", { text: i.label }), i.hint ? el("span", { class: "menu-hint", text: i.hint }) : ""];
      if (i.info) return el("div", { class: i.current ? "menu-info on" : "menu-info", role: "none" }, ...parts);
      return el(
        "button",
        {
          type: "button",
          role: i.radio ? "menuitemradio" : i.checked === undefined ? "menuitem" : "menuitemcheckbox",
          "aria-checked": i.checked === undefined ? null : String(i.checked),
          onclick: () => profileAct(i),
        },
        ...parts,
      );
    }),
  );
}

function profileAct(i) {
  closeMenus();
  if (i.key === "settings") {
    go(i.href);
  } else if (i.key === "me") {
    $("me_dialog").showModal();
  } else if (i.key === "tile") {
    pickTile(null); // '나' 타일 — 사이드바 머리의 편집 폼
    openTileEdit();
    // 설정 화면에서 왔으면 사이드바가 아직 숨어 있다 — 안내 화면을 그린 뒤 초점.
    if ($("sidebar").hidden) state.focusAfter = "tile_alias";
  } else if (i.key === "avail" || i.key === "focus") {
    if (!i.checked) setFocus(i.key === "focus"); // 이미 고른 상태면 그대로
  } else {
    // 알림 일시 중지(이 PC 에만) — 켜 있는 동안 브라우저 알림 · 트레이 표시를 내지 않는다, 풀면 알림 설정대로.
    const on = !i.checked;
    act("알림 일시 중지", async () => {
      fillPrefs(await api("PUT", "/api/settings", { notify_paused: on }));
      return on ? "알림을 일시 중지했습니다" : "알림을 다시 켰습니다";
    });
  }
}

$("avatar_btn").addEventListener("click", () => {
  renderProfileMenu();
  toggleMenu("avatar_btn", "profile_menu");
});
// 메뉴 · 검색 결과 밖을 누르면 닫는다.
document.addEventListener("click", (ev) => {
  for (const [b, m] of MENUS) {
    if ($(m).hidden || $(b).contains(ev.target) || $(m).contains(ev.target)) continue;
    $(m).hidden = true;
    $(b).setAttribute("aria-expanded", "false");
  }
  if (!ev.target.closest?.(".search")) closeSearch();
  // 상자 안에서 눌러 바뀐 항목(그 사이 떨어져 나간 요소)은 바깥 누름이 아니다.
  if (!$("float").hidden && ev.target.isConnected && !$("float").contains(ev.target) && !floatFrom?.contains(ev.target)) {
    // 초점을 받지 못하는 바탕을 누르면 초점이 body 로 떨어진다 — 그때만 연 요소로 돌린다(라운드 2). 초점을 받는 곳을 눌렀으면 그대로.
    const lost = document.activeElement === document.body || $("float").contains(document.activeElement);
    closeFloat(lost && !ev.target.closest("a, button, input, select, textarea, summary, [tabindex]"));
  }
});
// Ctrl+K = 통합 검색 칸, Esc = 메뉴 · 검색 결과 닫기.
document.addEventListener("keydown", (ev) => {
  if ((ev.ctrlKey || ev.metaKey) && ev.key.toLowerCase() === "k") {
    ev.preventDefault();
    $("search_q").focus();
    $("search_q").select();
  } else if (ev.key === "Escape") {
    const from = MENUS.find(([, m]) => $(m).contains(document.activeElement));
    const inSearch = $("search_box").contains(document.activeElement);
    if (!$("float").hidden) {
      closeFloat(true);
      return;
    }
    closeMenus();
    closeSearch();
    if (from) $(from[0]).focus();
    else if (inSearch) $("search_q").focus();
  }
});

// 통합 검색 — 250 ms 뒤 조회, 묶음(메시지 · 대화 · 사람 · 에이전트)별 결과 상자, 누르면 그곳으로(`view.js` `searchGroups`).
const searchAsk = new Latest();
let searchTimer;

function closeSearch() {
  clearTimeout(searchTimer);
  searchAsk.next(); // 오고 있는 결과가 닫힌 상자를 다시 열지 않게
  $("search_box").hidden = true;
}

function showSearch(children) {
  $("search_box").replaceChildren(...children);
  $("search_box").hidden = false;
}

async function runSearch() {
  const q = $("search_q").value.trim();
  const ticket = searchAsk.next();
  if (!q) {
    closeSearch();
    return;
  }
  let r;
  try {
    r = await api("GET", `/api/search?q=${encodeURIComponent(q)}`);
  } catch (e) {
    if (searchAsk.is(ticket)) showSearch([el("p", { class: "hint", text: "검색 실패: " + e.message })]);
    return;
  }
  if (!searchAsk.is(ticket)) return; // 그 사이 더 쳤다
  const groups = searchGroups(r, state.peerNames);
  if (!groups.length) {
    showSearch([el("p", { class: "hint", text: "맞는 것 없음" })]);
    return;
  }
  showSearch(
    groups.map((g) =>
      el(
        "section",
        {},
        el("h3", { text: g.title }),
        el(
          "ul",
          {},
          ...g.rows.map((row) =>
            el("li", {}, el("button", { type: "button", onclick: () => searchGo(row) }, el("span", { text: row.label }), el("span", { class: "sub", text: row.sub }))),
          ),
        ),
      ),
    ),
  );
}

/// 검색 결과로 — 메시지 = 그 글 위치, 대화 = 그 대화, 사람 = 그 타일, 상대 에이전트 = 세션 생성 채움, 내 에이전트 = 설정.
function searchGo(row) {
  closeSearch();
  if (row.tile != null) pickTile(row.tile);
  else if (row.chat) newChat(row.chat.peer, row.chat.agent);
  else go(row.href);
}

$("search_q").addEventListener("input", () => {
  clearTimeout(searchTimer);
  searchTimer = setTimeout(runSearch, 250);
});
$("search_q").addEventListener("focus", () => {
  if ($("search_q").value.trim() && $("search_box").hidden) runSearch();
});
$("search_q").addEventListener("keydown", (ev) => {
  if (ev.key === "ArrowDown") {
    ev.preventDefault();
    $("search_box").querySelector("button")?.focus();
  }
});

// 종 = 내 활동 전부.
$("bell").addEventListener("click", (ev) => {
  ev.preventDefault();
  pickRail(null);
  go("#activity");
});

$("detail_btn").addEventListener("click", () => {
  state.detailOpen = !state.detailOpen;
  showDetail();
});
$("detail_close").addEventListener("click", () => {
  state.detailOpen = false;
  showDetail();
});

renderTree();
window.addEventListener("hashchange", render);
await showSettings(); // 테마 · 기본값을 먼저 — 그룹설정의 "기본값(…)" 문구가 쓴다
await Promise.all([showStatus(), showMe(), loadPeers(), loadAgents(), loadConvs(), loadGroups()]);
// 웹UI 를 열 때 페어링된 상대 연결 확인 한 번(#1c).
pingPeers(state.peers.filter((p) => p.paired).map((p) => p.id));
await render();
await loadCounts();
listen();
// ponytail: 이 PC 에서 생긴 변경 · 시간이 지나 꺼지는 접속 표시는 신호가 없어 5초마다 전부 다시 읽는다 — 그 신호가 생기면 줄인다(D39 ②).
setInterval(() => refresh(), 5000);
