// 화면 조립 — 시안 v4(Nocturne) 5영역: 상단 바(통합 검색 · + 새로 만들기 · 알림 · 프로필) · 레일 · 사이드바(내비 · ★ 즐겨찾기 · 대화 ·
// 에이전트) · 메인(대화 / 보낸 메시지 · 내 활동 · 저장된 항목 · 에이전트 · 설정 트리) · 실행 상세. 값은 textContent 로만 넣는다(HTML 주입
// 없음) — 원격에서 온 이름 · 본문도 같다. HTML 문서 본문만 sandbox iframe(srcdoc · CSP)으로. 판단 로직은 view.js(node 테스트).
import {
  stateText,
  peerState,
  convLabel,
  authorLabel,
  kindText,
  relayText,
  keepsConversation,
  convRows,
  tileKeeps,
  formatDuration,
  formatTime,
  route,
  Latest,
  settingsTree,
  groupFilter,
  groupNames,
  inboxRows,
  isHtmlDoc,
  htmlSrcdoc,
  markdownBlocks,
  peerMessages,
  tileText,
  displayName,
  sideNav,
  newMenu,
  profileMenu,
  convSections,
  convMeta,
  unreadByPeer,
  activityRows,
  insertMarkdown,
  enterSends,
  timeline,
  runSteps,
  messageActions,
  searchGroups,
  lastRun,
  presence,
  refusedText,
  askTag,
  mentionQuery,
  mentionTarget,
  hasMention,
  mentionToken,
  preview,
  READ_ONLY_TOOLS,
  EFFORTS,
} from "/view.js";

const $ = (id) => document.getElementById(id);

/// JSON 을 받거나, 실패 사유를 담은 Error 를 던진다.
async function api(method, path, body) {
  const init = { method, headers: {} };
  if (body !== undefined) {
    init.headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  const res = await fetch(path, init);
  const text = await res.text();
  let data = null;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    // JSON 이 아닌 오류 본문(요청 형식 거부 등)은 그대로 사유로.
    if (res.ok) throw new Error("응답이 JSON 이 아니다");
    throw new Error(`HTTP ${res.status} ${text.slice(0, 200)}`);
  }
  if (!res.ok) throw new Error(data?.error ?? "HTTP " + res.status);
  return data;
}

/// 요소 만들기 — `text` 는 textContent, `class` · 속성, `on*` 는 이벤트.
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
  current: null, // 열린 대화(요약)
  open: new Set(), // 펼쳐 둔 연결 · 에이전트(다시 그려도 유지)
  unseen: 0, // 안 본 내 활동 — 고른 타일 범위(사이드바 내비)
  unseenAll: 0, // 안 본 내 활동 — 전부(종 점)
  peerAgents: new Map(), // 상대 id → 그 PC 의 에이전트 이름(`null` = 받지 못함) — 타일을 고를 때 받는다(원격 왕복이라 새로 고침마다 묻지 않는다)
  detailOpen: window.innerWidth >= 1280, // 실행 상세 — 넓은 창이면 열고 시작(저장하지 않는다)
  focusAfter: null, // 다음 화면을 그린 뒤 초점을 줄 칸 id
  actFresh: new Set(), // 내 활동을 열 때 안 본 항목 — 본 것으로 표시한 뒤 다시 그려도 강조를 남긴다
  msgs: [], // 열린 대화의 글(`msgsOf` = 그 대화 id) — 머리글 부제 · 실행 상세가 쓴다
  msgsOf: null,
  pickMsg: null, // 실행 상세에서 고른 글(요청 또는 그 답) — 없으면 마지막 요청
  focusMsg: null, // 대화를 그린 뒤 갈 글(`#c/{대화}/m/{글}`) — 늦게 그려져도 그때 간다
  mention: null, // 입력창 멘션 후보 — `{start, items, active}`(열려 있을 때만)
  mentionWarned: null, // 후보와 맞지 않는 멘션이라 한 번 막은 글 — 같은 글을 다시 보내면 멘션 없이 보낸다
};
const latest = new Latest();

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

$("raw_btn").addEventListener("click", () => {
  const on = $("conversation").classList.toggle("show-raw");
  const label = on ? "뷰어로 보기" : "원문 보기";
  $("raw_btn").setAttribute("aria-pressed", String(on));
  $("raw_btn").title = label;
  $("raw_btn").setAttribute("aria-label", label);
});

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
    renderSideHead();
  } catch (e) {
    for (const id of ["me_name", "me_host", "me_fingerprint", "me_addrs", "side_name"]) $(id).textContent = "오류: " + e.message;
  }
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme === "light" ? "light" : "dark";
}

// ---- 피어 · 레일 ----

async function loadPeers(force = false) {
  try {
    state.peers = await api("GET", "/api/peers");
  } catch (e) {
    $("peer_msg").textContent = "연결 목록 오류: " + e.message;
    return;
  }
  state.peerNames = new Map(state.peers.map((p) => [p.id, `${displayName(p)} · ${p.host}`]));
  renderRail();
  renderSideHead();
  renderSideAgents();
  renderPeerList(force);
  fillPeerSelects();
  // 표시 이름이 바뀌었을 수 있다 — 목록 · 열린 대화 머리글도 새 이름으로.
  renderConvList();
  if (state.current && route(location.hash).view in PANELS) renderConvHead(state.current);
}

/// 타일 얼굴 — 아이콘이 있으면 그림, 없으면 이름 앞 두 글자.
function tileFace(name, icon) {
  return icon ? el("img", { src: icon, alt: "" }) : tileText(name);
}

/// 타일 색 — 페어링된 상대 순서로 다섯 색을 돈다.
function tileColor(p) {
  const i = state.peers.filter((x) => x.paired).findIndex((x) => x.id === p.id);
  return `peer-${(Math.max(i, 0) % 5) + 1}`;
}

function renderRail() {
  const paired = state.peers.filter((p) => p.paired);
  if (state.railPeer != null && !paired.some((p) => p.id === state.railPeer)) {
    state.railPeer = null;
    closeTileEdit(); // 그 상대에 묶인 편집
  }
  const unread = unreadByPeer(state.convs);
  const sig = [state.railPeer, state.settings.me_alias, state.settings.me_icon, paired.map((p) => [p.id, displayName(p), p.host, p.icon, unread.get(p.id) ?? 0, p.online, p.focus])];
  if (unchanged($("rail"), JSON.stringify(sig))) return;
  const me = $("rail_me");
  me.setAttribute("aria-pressed", String(state.railPeer == null));
  me.setAttribute("aria-label", `${state.settings.me_alias || "나"} — 전체 대화`);
  me.replaceChildren(tileFace(state.settings.me_alias || "나", state.settings.me_icon));
  $("rail_pcs").replaceChildren(
    ...paired.map((p) => {
      const n = unread.get(p.id) ?? 0;
      const pr = presence(p);
      return el(
        "button",
        {
          type: "button",
          class: `rail-btn ${tileColor(p)}`,
          "aria-label": `${displayName(p)} · ${p.host} 의 대화만${n ? ` · 안 읽음 ${n}` : ""} — ${pr.text}`,
          "aria-pressed": String(state.railPeer === p.id),
          onclick: () => pickTile(p.id),
        },
        tileFace(displayName(p), p.icon),
        n ? el("span", { class: "rail-badge", "aria-hidden": "true", text: n > 99 ? "99+" : String(n) }) : "",
        el("span", { class: `rail-dot ${pr.key}`, "aria-hidden": "true" }),
      );
    }),
  );
}

/// 레일 선택 — 다른 PC 를 고르면 열린 대화를 놓는다(입력창이 이전 상대를 가리킨 채 남지 않게). 화면 이동은 부른 쪽이.
function pickRail(id) {
  const changed = state.railPeer !== id;
  if (changed) closeTileEdit();
  state.railPeer = id;
  renderRail();
  renderSideHead();
  if (state.current && !keepsConversation(state.current, id)) state.current = null;
  renderConvList();
  if (changed) state.unseen = 0; // 새 범위의 수는 곧 다시 읽는다
  renderSideNav();
  if (id != null && !state.peerAgents.has(id) && state.peers.some((x) => x.id === id && x.paired)) loadPeerAgents(id);
  renderSideAgents();
  if (changed) loadCounts();
}

/// 타일 누름 — 사이드바 = 그 타일의 대화 목록('나' = 전부). 열린 대화가 그 타일 것이면 그대로, 아니면 메인은 안내
/// (목록에서 대화를 고르면 메인에 메시지). 상대 타일을 누르면 그 PC 의 에이전트 목록을 다시 받는다.
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
  if (p) {
    $("side_name").textContent = displayName(p);
    $("side_sub").textContent = [p.alias ? `${p.name} · ${p.host}` : p.host, peerState(p), p.paired ? presence(p).text : ""].filter(Boolean).join(" · ");
  } else if (state.me) {
    const alias = state.settings.me_alias;
    const fp = "지문 " + state.me.fingerprint.split(" ").slice(0, 2).join("·");
    $("side_name").textContent = alias || "나 — " + state.me.host;
    $("side_sub").textContent = [state.me.name, alias ? state.me.host : "", fp].filter(Boolean).join(" · ");
  }
}

$("rail_me").addEventListener("click", () => pickTile(null));

/// 상단 바 아바타 = '나' 타일 얼굴(표시 이름 · 아이콘).
function renderAvatar() {
  $("avatar_btn").replaceChildren(tileFace(state.settings.me_alias || "나", state.settings.me_icon));
}

/// 사이드바 내비 — 고른 타일 범위(`view.js` `sideNav`), 지금 화면은 강조. '대화' 는 대화 · 보낸 메시지 화면에서도.
function renderSideNav() {
  const view = route(location.hash).view;
  if (unchanged($("side_nav"), JSON.stringify([view, state.railPeer, state.unseen]))) return;
  $("side_nav").replaceChildren(
    ...sideNav(state.railPeer, state.unseen).map((n) => {
      const target = route(n.href).view;
      const current = target === view || (target === "home" && (view === "conversation" || view === "sent"));
      return el(
        "li",
        {},
        el(
          "a",
          { href: n.href, class: current ? "current" : null, "aria-current": current ? "page" : null },
          el("span", { text: n.label }),
          n.count ? el("span", { class: "count", text: String(n.count) }) : "",
        ),
      );
    }),
  );
}

/// 사이드바 "{이름}의 에이전트" — '나' = 내 에이전트(누르면 설정), 상대 = 그 PC 의 에이전트(누르면 새 대화에 채움).
function renderSideAgents() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  $("side_agents_title").textContent = p ? `${displayName(p)}의 에이전트` : "내 에이전트";
  // 5초 새로 고침이 링크를 다시 만들어 초점 · 누르는 중인 클릭을 빼앗지 않게 — 그리는 값이 같으면 둔다.
  const names = p ? state.peerAgents.get(p.id) : state.agents.map((a) => a.name);
  if (unchanged($("side_agents"), JSON.stringify([p?.id ?? null, names === undefined ? "…" : names]))) return;
  const item = (name, props) =>
    el("li", {}, el("a", props, el("span", { class: "agent-dot", "aria-hidden": "true" }), el("span", { class: "agent-name", text: name })));
  let rows;
  if (!p) {
    rows = state.agents.map((a) => item(a.name, { href: "#settings/agents" }));
  } else {
    if (names === undefined) rows = [el("li", { class: "hint", text: "불러오는 중 …" })];
    else if (names === null) rows = [el("li", { class: "hint", text: "목록을 받지 못했습니다" })];
    else
      rows = names.map((n) =>
        item(n, {
          href: "#new",
          onclick: (ev) => {
            ev.preventDefault();
            newChat(p.id, n);
          },
        }),
      );
  }
  $("side_agents").replaceChildren(...(rows.length ? rows : [el("li", { class: "hint", text: "에이전트 없음" })]));
}

/// 상대 PC 의 에이전트 이름 — 원격 왕복이라 타일을 고를 때만 묻는다.
async function loadPeerAgents(id) {
  state.peerAgents.set(id, undefined); // 묻는 중 — 같은 상대를 겹쳐 묻지 않는다
  try {
    state.peerAgents.set(id, (await api("GET", `/api/peers/${id}/agents`)).names);
  } catch {
    state.peerAgents.set(id, null);
  }
  if (state.railPeer === id) renderSideAgents();
  if (route(location.hash).view === "agents") renderAgentsView();
  if (state.current?.peer_id === id) renderMentionHint();
}

/// 새 대화 화면에 상대 · 에이전트를 채워서 연다(사이드바 · 검색 결과의 상대 에이전트).
async function newChat(peer, agent) {
  if (!state.peers.some((p) => p.id === peer && p.paired)) {
    $("status_msg").textContent = "페어링되지 않은 상대 — 연결 목록에서 승인 · 다시 연결하세요";
    go("#settings/peers");
    return;
  }
  state.focusAfter = "chat_text";
  go("#new");
  $("chat_peer").value = String(peer);
  await loadChatAgents(agent);
}

// ---- 타일 표시 이름 · 아이콘(이 PC 에만) ----

const ICON_PX = 80; // 타일 40px × 2(고해상도 화면)
/// 열린 타일 편집 — `peer` = 상대 id(`null` = 나), `icon` = `undefined`(그대로) · `""`(지움) · PNG data URL.
let tileEdit = null;

function openTileEdit() {
  const p = state.peers.find((x) => x.id === state.railPeer);
  tileEdit = { peer: p?.id ?? null, icon: undefined };
  $("tile_alias").value = (p ? p.alias : state.settings.me_alias) ?? "";
  $("tile_alias").placeholder = p ? p.name : "나";
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
  const p = state.peers.find((x) => x.id === tileEdit.peer);
  const icon = tileEdit.icon !== undefined ? tileEdit.icon : p ? p.icon : state.settings.me_icon;
  const name = $("tile_alias").value.trim() || (p ? p.name : "나");
  const box = $("tile_preview");
  box.className = `rail-btn tile-preview ${p ? tileColor(p) : "me"}`;
  box.replaceChildren(tileFace(name, icon));
}

/// 고른 그림 → 가운데 정사각형을 잘라 `ICON_PX` 크기 PNG data URL(서버는 PNG data URL 만 받는다).
async function iconFromFile(file) {
  const url = URL.createObjectURL(file);
  try {
    const img = new Image();
    img.src = url;
    await img.decode();
    const s = Math.min(img.naturalWidth, img.naturalHeight);
    if (!s) throw new Error("크기를 알 수 없는 그림");
    const canvas = el("canvas", { width: ICON_PX, height: ICON_PX });
    canvas.getContext("2d").drawImage(img, (img.naturalWidth - s) / 2, (img.naturalHeight - s) / 2, s, s, 0, 0, ICON_PX, ICON_PX);
    return canvas.toDataURL("image/png");
  } finally {
    URL.revokeObjectURL(url);
  }
}

$("tile_edit_btn").addEventListener("click", () => ($("tile_form").hidden ? openTileEdit() : closeTileEdit()));
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
    $("tile_msg").textContent = "아이콘 준비 — 저장하면 바뀝니다";
  } catch (e) {
    $("tile_msg").textContent = "아이콘을 만들지 못했다: " + e.message;
  }
});
$("tile_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const edit = tileEdit;
  if (!edit) return;
  const { peer, icon } = edit;
  const alias = $("tile_alias").value;
  act(
    "타일 저장",
    async () => {
      if (peer == null) {
        fillPrefs(await api("PUT", "/api/settings", icon === undefined ? { me_alias: alias } : { me_alias: alias, me_icon: icon }));
      } else {
        await api("PATCH", `/api/peers/${peer}`, icon === undefined ? { alias } : { alias, icon });
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

$("rail_add").addEventListener("click", () => goSettings("invite_accept", "invite_text"));
$("rail_settings").addEventListener("click", () => go("#settings"));

/// 펼침 상태를 기억하는 `<details>`.
function remembered(key, summary, ...body) {
  const d = el("details", { class: "item", open: state.open.has(key) }, el("summary", {}, ...summary), el("div", { class: "item-body" }, ...body));
  d.addEventListener("toggle", () => (d.open ? state.open.add(key) : state.open.delete(key)));
  return d;
}

function groupBox(g, items) {
  return el("details", { class: "group", open: true }, el("summary", { text: `${g.group} · ${g.items.length}` }), ...items);
}

function fillGroupSelect(sel, items) {
  const keep = sel.value;
  sel.replaceChildren(el("option", { value: "", text: "전체" }), ...groupNames(items).map((g) => el("option", { value: g, text: g })));
  if ([...sel.options].some((o) => o.value === keep)) sel.value = keep;
}

function kvTable(rows) {
  return el(
    "table",
    { class: "kv" },
    el("tbody", {}, ...rows.map(([k, v, cls]) => el("tr", {}, el("th", { scope: "row", text: k }), el("td", {}, el("span", { class: cls, text: v }), " ", el("span", { class: "ro", text: "읽기 전용" }))))),
  );
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

/// 목록을 다시 그려도 되나 — 같은 값이면 건너뛰고, 사용자가 그 안에서 입력 중이면(배경 새로 고침) 미룬다. 저장 · 필터는 `force`.
function needsRender(box, sig, force) {
  if (!force && (box.dataset.sig === sig || box.contains(document.activeElement))) return false;
  box.dataset.sig = sig;
  return true;
}

function renderPeerList(force = false) {
  // 접속 상태(`online` · `focus`)는 이 목록에 그리지 않는다 — 30초 Ping 이 저장 전 입력을 지우지 않게 서명에서 뺀다.
  const shown = state.peers.map(({ online, focus, ...p }) => p);
  const sig = JSON.stringify([shown, $("peer_group").value, $("peer_q").value, state.settings.claude_model, state.settings.claude_effort]);
  if (!needsRender($("peer_groups"), sig, force)) return;
  fillGroupSelect($("peer_group"), state.peers);
  const peerAct = (label, fn) => act(label, fn, "peer_msg", () => loadPeers(true));
  const defaults = state.settings;
  const groups = groupFilter(state.peers, { group: $("peer_group").value, q: $("peer_q").value, fields: ["alias", "name", "host", "fingerprint"] });
  $("peer_groups").replaceChildren(
    ...groups.map((g) =>
      groupBox(
        g,
        g.items.map((p) => {
          const group = el("input", { value: p.group ?? "", maxlength: "32", placeholder: "그룹 없음" });
          const model = el("input", { value: p.model ?? "", list: "model_names", maxlength: "64", placeholder: `기본값(${defaults.claude_model ?? "sonnet"})` });
          const effort = el(
            "select",
            {},
            el("option", { value: "", text: `기본값(${defaults.claude_effort ?? "high"})` }),
            ...EFFORTS.map((e) => el("option", { value: e, text: e })),
          );
          effort.value = p.effort ?? "";
          const actions = el("div", { class: "actions" });
          actions.append(
            button(
              "저장",
              () => peerAct("연결 설정 저장", () => api("PATCH", `/api/peers/${p.id}`, { group: group.value, model: model.value, effort: effort.value })),
              "primary",
            ),
          );
          if (!p.my_ok) actions.append(button("승인", () => peerAct("승인", () => api("POST", `/api/peers/${p.id}/approve`))));
          if (p.my_ok && !p.paired) actions.append(button("다시 알리기", () => peerAct("승인 재통지", () => api("POST", `/api/peers/${p.id}/approve`))));
          if (p.paired) actions.append(button("연결 확인", () => peerAct("연결 확인", () => api("POST", `/api/peers/${p.id}/ping`))));
          actions.append(
            button("내 에이전트 도구 편집", () => goSettings("agents")),
            button(
              "삭제",
              () =>
                peerAct("삭제", async () => {
                  const r = await api("DELETE", `/api/peers/${p.id}`);
                  if (!r.notified) throw new Error("내 쪽은 삭제했지만 상대에 알리지 못했다 — 상대 목록에는 남아 있다");
                }),
              "danger",
            ),
          );
          return remembered(
            "peer" + p.id,
            [
              el("span", { class: "dot" + (p.paired ? " ok" : ""), "aria-hidden": "true" }),
              el("strong", { text: displayName(p) }),
              el("span", { class: "hint", text: p.alias ? `${p.name} · ${p.host}` : p.host }),
              el("span", { class: p.paired ? "state-ok" : "state-wait", text: peerState(p) }),
            ],
            kvTable([
              ["지문", p.fingerprint, "mono"],
              ["주소", p.addr, "mono"],
            ]),
            el("div", { class: "form-grid" }, ...field("그룹", group), ...field("모델", model), ...field("effort", effort)),
            actions,
          );
        }),
      ),
    ),
  );
  if (!groups.length) $("peer_groups").append(el("p", { class: "hint", text: state.peers.length ? "맞는 연결 없음" : "연결 없음 — 초대 생성 · 초대 수락으로 잇습니다" }));
}

for (const id of ["peer_group", "peer_q"]) $(id).addEventListener("input", () => renderPeerList(true));

$("window_btn").addEventListener("click", () =>
  act(
    "초대 생성",
    async () => {
      const r = await api("POST", "/api/pairing-window");
      $("invite").textContent = r.invite;
      $("invite_box").hidden = false;
      return `초대 생성 — ${r.seconds / 60}분 동안 이 초대로 들어온 요청 1건을 바로 등록합니다`;
    },
    "peer_msg",
    loadPeers,
  ),
);

$("copy_btn").addEventListener("click", () =>
  act(
    "초대 복사",
    async () => {
      await navigator.clipboard.writeText($("invite").textContent);
      return "초대를 복사했다 — 상대에게 보내세요";
    },
    "peer_msg",
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
      return "추가 · 승인 완료";
    },
    "peer_msg",
    loadPeers,
  );
});

$("add_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const addr = $("add_addr").value;
  const code = $("add_code").value;
  act("추가", () => api("POST", "/api/peers", { addr, code }), "peer_msg", loadPeers);
});

// ---- 내 에이전트 ----

function toolChecks(selected) {
  const set = new Set(selected.split(",").map((t) => t.trim()));
  const boxes = READ_ONLY_TOOLS.map((t) => el("input", { type: "checkbox", value: t, checked: set.has(t) }));
  const wrap = el("div", { class: "checks" }, ...boxes.map((b) => el("label", {}, b, " " + b.value)));
  wrap.value = () =>
    boxes
      .filter((b) => b.checked)
      .map((b) => b.value)
      .join(",");
  return wrap;
}

/// 고른 도구 — 하나도 없으면 서버가 조용히 기본값으로 바꾸므로 여기서 막는다.
function chosenTools(checks) {
  const v = checks.value();
  if (!v) throw new Error("허용 도구를 하나 이상 고르세요");
  return v;
}

async function loadAgents() {
  try {
    state.agents = await api("GET", "/api/agents");
  } catch (e) {
    $("agent_msg").textContent = "에이전트 목록 오류: " + e.message;
    return;
  }
  // 새 대화의 "내 에이전트로 자동 왕복" 후보.
  const relaySel = $("chat_relay_agent");
  const keep = relaySel.value;
  relaySel.replaceChildren(
    el("option", { value: "", text: "안 함 — 상대 에이전트에게만" }),
    ...state.agents.map((a) => el("option", { value: a.name, text: a.name })),
  );
  if (state.agents.some((a) => a.name === keep)) relaySel.value = keep;
  renderAgentList(true);
  renderSideAgents();
}

function renderAgentList(force = false) {
  const sig = JSON.stringify([state.agents, $("agent_group").value, $("agent_q").value]);
  if (!needsRender($("agent_groups"), sig, force)) return;
  fillGroupSelect($("agent_group"), state.agents);
  const groups = groupFilter(state.agents, { group: $("agent_group").value, q: $("agent_q").value, fields: ["name", "folder"] });
  $("agent_groups").replaceChildren(
    ...groups.map((g) =>
      groupBox(
        g,
        g.items.map((a) => {
          const folder = el("input", { value: a.folder, class: "mono" });
          const tools = toolChecks(a.tools);
          const prompt = el("textarea", { rows: "3" });
          prompt.value = a.prompt;
          const promptBox = el("div", { class: "field" }, prompt);
          const group = el("input", { value: a.group ?? "", maxlength: "32", placeholder: "그룹 없음" });
          const item = remembered(
            "agent" + a.id,
            [el("strong", { text: a.name }), a.group ? el("span", { class: "tag plain", text: a.group }) : "", el("span", { class: "hint mono", text: a.folder })],
            kvTable([["이름", a.name]]),
            el(
              "div",
              { class: "form-grid" },
              ...field("폴더(절대 경로)", folder),
              el("span", { class: "label hint", text: "허용 도구" }),
              tools,
              ...field("추가 지시", prompt, promptBox),
              ...field("그룹", group),
            ),
            el(
              "div",
              { class: "actions" },
              button(
                "저장",
                () =>
                  act(
                    "에이전트 저장",
                    () => api("PATCH", `/api/agents/${a.id}`, { folder: folder.value, tools: chosenTools(tools), prompt: prompt.value, group: group.value }),
                    "agent_msg",
                    loadAgents,
                  ),
                "primary",
              ),
              button("삭제", () => act("삭제", () => api("DELETE", `/api/agents/${a.id}`), "agent_msg", loadAgents), "danger"),
            ),
          );
          return item;
        }),
      ),
    ),
  );
  if (!groups.length) $("agent_groups").append(el("p", { class: "hint", text: state.agents.length ? "맞는 에이전트 없음" : "등록된 에이전트 없음 — 아래에서 등록" }));
}

for (const id of ["agent_group", "agent_q"]) $(id).addEventListener("input", () => renderAgentList(true));

const newTools = toolChecks("Read,Grep,Glob");
$("agent_tools").replaceWith(Object.assign(newTools, { id: "agent_tools" }));

$("agent_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = {
    name: $("agent_name").value,
    folder: $("agent_folder").value,
    prompt: $("agent_prompt").value,
  };
  act(
    "등록",
    async () => {
      await api("POST", "/api/agents", { ...body, tools: chosenTools(newTools) });
      $("agent_form").reset();
    },
    "agent_msg",
    loadAgents,
  );
});

// ---- 대화 목록 ----

async function loadConvs() {
  try {
    state.convs = await api("GET", "/api/conversations");
  } catch (e) {
    $("status_msg").textContent = "대화 목록 오류: " + e.message;
    return;
  }
  if (state.current) state.current = state.convs.find((c) => c.id === state.current.id) ?? state.current;
  renderConvList();
  renderRail();
}

function tagEl(tag) {
  return el("span", { class: "tag" + (tag === "받음" ? " recv" : ""), text: tag });
}

/// 대화 행 — 방향 태그 · 제목 · `{에이전트} · {상태}` · 실행 중 표시 · 자동 왕복 칩 · 안 읽음(굵게 + 점). 상대 이름은 툴팁('나' 범위에서 섞여 보인다).
function convItem({ c, label }) {
  const meta = convMeta(c);
  const cls = [state.current?.id === c.id ? "current" : "", meta.unread ? "unread" : ""].filter(Boolean).join(" ");
  return el(
    "li",
    { class: cls || null },
    el(
      "a",
      { href: `#c/${c.id}`, title: label.sub },
      el(
        "span",
        { class: "title" },
        meta.unread ? el("span", { class: "unread-dot", role: "img", "aria-label": "안 읽음" }) : "",
        tagEl(label.tag),
        el("span", { text: label.title }),
      ),
      el(
        "span",
        { class: "sub" },
        el("span", { text: meta.state ? `${meta.agent} ·` : meta.agent }),
        meta.running ? el("span", { class: "spin", "aria-hidden": "true" }) : "",
        meta.state ? el("span", { class: `st-${c.last_state}`, text: meta.state }) : "",
        meta.relay ? el("span", { class: "chip", text: meta.relay }) : "",
      ),
    ),
  );
}

function renderConvList() {
  const q = $("conv_search").value.trim();
  const rows = convRows(state.convs, state.peerNames, state.railPeer, q);
  if (unchanged($("conv_sections"), JSON.stringify([q, state.railPeer, state.current?.id, rows]))) return;
  const empty = el("li", { class: "hint", text: q ? "맞는 대화 없음" : "대화 없음" });
  $("conv_sections").replaceChildren(
    ...convSections(rows).map((s) =>
      el(
        "section",
        { class: "side-section" },
        el(
          "h2",
          { class: "side-title" },
          el("span", { text: s.title }),
          s.key === "convs" ? el("a", { href: "#new", class: "side-add", title: "새 대화", "aria-label": "새 대화", text: "+" }) : "",
        ),
        el("ul", { class: "side-list" }, ...(s.rows.length ? s.rows.map(convItem) : rows.length ? [] : [empty])),
      ),
    ),
  );
}

$("conv_search").addEventListener("input", renderConvList);

// ---- 화면 전환 ----

/// 화면 → 메인의 절 id(보낸 메시지는 대화 절의 탭).
const VIEW_EL = {
  home: "home",
  activity: "activity",
  saved: "saved",
  agents: "agents_view",
  peer: "peer",
  conversation: "conversation",
  sent: "conversation",
  new: "new",
  settings: "settings",
};
const PANELS = { conversation: "panel_conv", sent: "panel_sent" };

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
  $("rail_me").setAttribute("aria-pressed", String(state.railPeer == null));
  if (r.view in PANELS) {
    const conv = await findConv(r.id);
    if (!conv) {
      $("status_msg").textContent = `대화 ${r.id} 를 찾지 못했다`;
      show("home");
      return;
    }
    state.current = conv;
    // 뒤로 가기 등으로 레일에서 고른 PC 와 다른 대화를 열면 레일을 '나'(전부)로 — 열린 대화로 보낼 수 있게.
    if (!keepsConversation(conv, state.railPeer)) pickRail(null);
    renderConvList();
    showConversation(conv, r.view);
    state.focusMsg = r.view === "conversation" ? (r.msg ?? null) : null;
    if (r.view === "conversation") await openConversation(conv);
    else await openSent(conv);
    applyFocus();
    return;
  }
  state.current = null;
  // 뒤로 가기 등으로 온 상대 글 목록이면 레일도 맞춘다.
  if (r.view === "peer" && state.railPeer !== r.id) pickRail(r.id);
  renderConvList();
  show(r.view);
  if (r.view === "activity") await loadActivity(true);
  if (r.view === "saved") await loadSaved();
  if (r.view === "agents") loadAgentsView();
  if (r.view === "peer") await loadPeerMessages(r.id);
  if (r.view === "settings" && r.focus) $(r.focus).scrollIntoView({ block: "start" });
  applyFocus();
}

/// 대화 머리글 — 방향 태그 · 제목 · ★ · 부제 `{상대} 의 {에이전트} · {model} · {effort}`(마지막 답의 값, 없으면 뺀다).
function renderConvHead(conv) {
  const label = convLabel(conv, state.peerNames);
  $("conv_tag").replaceWith(Object.assign(tagEl(label.tag), { id: "conv_tag" }));
  $("conv_title").textContent = label.title;
  const run = state.msgsOf === conv.id ? lastRun(state.msgs, conv.dir !== "out") : "";
  $("conv_sub").textContent = [label.sub, run].filter(Boolean).join(" · ");
  $("star_btn").setAttribute("aria-pressed", String(conv.starred === true));
  $("star_btn").textContent = conv.starred ? "★" : "☆";
  const pr = presence(state.peers.find((p) => p.id === conv.peer_id));
  Object.assign($("conv_presence"), { textContent: pr.text, className: `presence ${pr.key}`, hidden: false });
}

/// 대화 머리글 · 탭 — 대화 / 보낸 메시지는 같은 머리글 아래 패널만 바뀐다. 입력창 안내 = 대화 에이전트에게.
function showConversation(conv, view) {
  show("conversation");
  editTitle(false);
  renderConvHead(conv);
  for (const [v, tab, href] of [
    ["conversation", "tab_conv", `#c/${conv.id}`],
    ["sent", "tab_sent", `#s/${conv.id}`],
  ]) {
    $(tab).href = href;
    if (v === view) $(tab).setAttribute("aria-current", "page");
    else $(tab).removeAttribute("aria-current");
  }
  for (const [v, panel] of Object.entries(PANELS)) $(panel).hidden = v !== view;
  $("reply_text").placeholder = `${conv.agent} 에게 보내기 — markdown 가능`;
  // 멘션 — 내가 연 대화(자동 왕복 아님)만. 후보 = 상대 PC 에이전트(타일을 고를 때와 같은 목록 — 없거나 못 받았으면 이때 다시 묻는다).
  const names = state.peerAgents.get(conv.peer_id);
  if (conv.dir === "out" && !conv.relay_agent && (!state.peerAgents.has(conv.peer_id) || names === null)) loadPeerAgents(conv.peer_id);
  renderMentionHint();
  closeMention();
  if (state.msgsOf !== conv.id) $("conv_relay").hidden = $("conv_relay_btn").hidden = true;
}

// ---- 대화 ----

/// 실행 중 경과 — 받은 때부터(대기 포함, 실행 시작 시각은 남지 않는다).
function elapsed(since) {
  return `실행 중 · ${Math.max(0, Math.round(Date.now() / 1000 - since))}초`;
}

function messageEl(conv, m) {
  const who = authorLabel(conv, m, state.peerNames.get(conv.peer_id));
  const running = m.state === "running";
  const meta = [formatTime(m.created_at)];
  if (m.state !== "done" && !running) meta.push(stateText(m.state));
  meta.push(refusedText(m));
  if (m.author === "agent" && (m.turns != null || m.duration_ms != null)) {
    meta.push([m.turns != null ? `${m.turns}턴` : "", formatDuration(m.duration_ms)].filter(Boolean).join(" · "));
  }
  const article = el(
    "article",
    { class: "msg " + m.author, "data-msg": String(m.id), tabindex: "0" },
    el("div", { class: "avatar", "aria-hidden": "true", text: who.slice(0, 1) }),
    el(
      "div",
      { class: "msg-body" },
      el(
        "div",
        { class: "msg-meta" },
        el("strong", { text: who }),
        el("span", { class: "tag plain", text: kindText(m) }),
        askTag(m) ? el("span", { class: m.done ? "tag ask done" : "tag ask", text: askTag(m) }) : "",
        el("span", { text: meta.filter(Boolean).join(" · ") }),
        // 5초마다 바뀌는 경과는 읽어 주지 않는다(대화가 aria-live).
        running ? el("span", { class: "st-running", "data-since": String(m.created_at), "aria-live": "off", text: elapsed(m.created_at) }) : "",
      ),
      ...bodyPair(m.text),
    ),
    el(
      "div",
      { class: "msg-actions", role: "group", "aria-label": "메시지 작업" },
      ...messageActions(m).map((a) => el("button", { type: "button", "data-act": a.key, text: a.label, onclick: () => messageAct(m, a.key) })),
    ),
  );
  // 누르면(Enter) 그 글의 요청이 실행 상세의 '진행' 에 — 작업 단추 · 글자 고르기(끌어서 선택)는 빼고.
  const pick = () => {
    state.pickMsg = m.id;
    renderDetail();
  };
  article.addEventListener("click", (ev) => {
    if (ev.target.closest(".msg-actions") || !window.getSelection().isCollapsed) return;
    pick();
  });
  article.addEventListener("keydown", (ev) => {
    if (ev.target === article && ev.key === "Enter") pick();
  });
  return article;
}

/// 메시지 작업 — 저장 · 해제(이 PC 에만), 복사(본문을 클립보드로), 다시 보내기(같은 대화 · 같은 대상에 새 요청 — 같은 종류),
/// 완료 표시 · 취소(받은 요청 — 쓴 쪽에 알린다, 알리지 못하면 경고).
function messageAct(m, key) {
  if (key === "copy") {
    act("복사", async () => {
      await navigator.clipboard.writeText(m.text);
      return "복사했다";
    });
  } else if (key === "resend") {
    sendText(m.text, { request: m.request === true, agent: m.agent });
  } else if (key === "done") {
    // 그 글의 태그 · 단추 · 실행 상세만 바꾸고 대화 서명을 맞춘다 — 대화를 다시 그리면 누른 단추의 초점이 사라지고 대화 전체를 다시 읽어 준다.
    // 상대가 바꾼 완료는 서명이 달라 새로 고침이 그린다.
    const cur = state.msgs.find((x) => x.id === m.id) ?? m;
    const on = !cur.done;
    act(on ? "완료 표시" : "완료 취소", async () => {
      const r = await api("PATCH", `/api/messages/${m.id}`, { done: on });
      cur.done = r.done;
      const box = $("messages").querySelector(`[data-msg="${m.id}"]`);
      if (box) {
        box.querySelector('[data-act="done"]').textContent = messageActions(cur).find((a) => a.key === "done").label;
        Object.assign(box.querySelector(".tag.ask"), { textContent: askTag(cur), className: cur.done ? "tag ask done" : "tag ask" });
      }
      if (state.msgs.includes(cur)) $("messages").dataset.sig = messagesSig(state.current, state.msgs);
      renderDetail();
      return r.warning ?? (on ? "완료 표시 — 쓴 쪽에 알렸다" : "완료 취소 — 쓴 쪽에 알렸다");
    });
  } else {
    // 저장 여부는 마지막으로 읽어 온 대화 기준. 바꾼 뒤 그 단추 글자 · 실행 상세만 — 대화를 다시 그리면 HTML 본문이 다시 읽힌다.
    const cur = state.msgs.find((x) => x.id === m.id) ?? m;
    const on = !cur.saved;
    act(on ? "저장" : "저장 해제", async () => {
      await api("PATCH", `/api/messages/${m.id}`, { saved: on });
      cur.saved = on;
      const btn = $("messages").querySelector(`[data-msg="${m.id}"] [data-act="save"]`);
      if (btn) btn.textContent = messageActions(cur).find((a) => a.key === "save").label;
      renderDetail();
      return on ? "저장 — 저장된 항목에 모았다(이 PC 에만)" : "저장 해제";
    });
  }
}

/// 대화 글 서명 — 바뀐 것이 없으면 다시 그리지 않는다(HTML 본문 · 스크롤 상태가 5초마다 풀리지 않게). 상대 표시 이름 · 요청 완료 · 거절 사유가
/// 바뀌면 다시. 저장 여부는 넣지 않는다 — 저장 단추가 그 글자만 바꾼다.
function messagesSig(conv, ms) {
  return [state.peerNames.get(conv.peer_id), ...ms.map((m) => `${m.id}:${m.state}:${m.text.length}:${m.done ? 1 : 0}:${m.refused ?? ""}`)].join(",");
}

/// 그 글로 — 가운데로 스크롤해 잠깐 강조하고 실행 상세의 '진행' 을 그 요청으로(`#c/{대화}/m/{글}`).
function focusMessage(id) {
  const box = $("messages").querySelector(`[data-msg="${id}"]`);
  if (!box) {
    $("status_msg").textContent = `글 ${id} 를 이 대화에서 찾지 못했다`;
    return;
  }
  state.pickMsg = id;
  renderDetail();
  box.scrollIntoView({ block: "center" });
  box.classList.add("flash");
  setTimeout(() => box.classList.remove("flash"), 2000);
}

/// 열린 대화의 글 — 머리글(부제 · 자동 왕복) · 실행 상세가 쓴다(대화 · 보낸 메시지 탭 공용).
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
    box.replaceChildren(...ms.map((m) => messageEl(conv, m)));
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
  btn.hidden = !r?.action;
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

for (const id of ["conv_relay_btn", "detail_relay_btn"]) $(id).addEventListener("click", () => switchRelay($(id).dataset.on === "true"));

/// ★ 즐겨찾기 켜고 끄기(이 PC 에만 — 상대에게 보내지 않는다). 켠 대화는 사이드바 '★ 즐겨찾기' 섹션에.
$("star_btn").addEventListener("click", () => {
  const conv = state.current;
  if (!conv) return;
  const on = !conv.starred;
  act(on ? "즐겨찾기" : "즐겨찾기 해제", async () => {
    await api("PATCH", `/api/conversations/${conv.id}`, { starred: on });
    await loadConvs();
    if (state.current?.id === conv.id) renderConvHead(state.current);
    return on ? "즐겨찾기 — 사이드바 ★ 즐겨찾기에(이 PC 에만)" : "즐겨찾기 해제";
  });
});

// ---- 실행 상세 — 진행 · 대화 정보 · 자동 왕복 · 저장된 항목(이 대화) ----

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
  $("info_agent_label").textContent = out ? "상대 에이전트" : "내 에이전트";
  $("info_agent").textContent = conv.agent;
  const run = lastRun(ms, !out);
  $("info_defaults").textContent = out
    ? run
      ? `${run} — 마지막 실행`
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
    btn.hidden = !relay.action;
    btn.textContent = relay.action === "resume" ? "재개" : "멈춤";
    btn.dataset.on = String(relay.action === "resume");
  }
  // 요청 — 이 대화의 요청 글(최근 글부터, 완료 표시).
  const peerName = state.peerNames.get(conv.peer_id);
  const reqs = ms.filter((m) => m.author === "human" && m.request).reverse();
  $("detail_req_n").textContent = reqs.length ? String(reqs.length) : "";
  if (!unchanged($("detail_req"), JSON.stringify([conv.id, ready, peerName, reqs.map((m) => [m.id, m.done, m.text.length])])))
    $("detail_req").replaceChildren(
      ...(reqs.length
        ? reqs.map((m) => detailItem(conv, m, `${askTag(m)} · ${authorLabel(conv, m, peerName)} · ${formatTime(m.created_at)}`))
        : [el("li", { class: "hint", text: ready ? "요청 없음 — 입력창에서 '요청' 을 골라 보낸 글이 모입니다" : "불러오는 중 …" })]),
    );
  // 저장된 항목 — 이 대화에서 저장한 글(최근 글부터).
  const saved = ms.filter((m) => m.saved).reverse();
  $("detail_saved_n").textContent = saved.length ? String(saved.length) : "";
  if (unchanged($("detail_saved"), JSON.stringify([conv.id, ready, peerName, saved.map((m) => [m.id, m.text.length])]))) return;
  $("detail_saved").replaceChildren(
    ...(saved.length
      ? saved.map((m) => detailItem(conv, m, `${authorLabel(conv, m, peerName)} · ${formatTime(m.created_at)}`))
      : [el("li", { class: "hint", text: ready ? "저장한 글 없음 — 메시지 작업 '저장' 으로 모읍니다" : "불러오는 중 …" })]),
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

// 머리글 "저장된 항목" — 실행 상세를 열고 그 절로.
$("saved_btn").addEventListener("click", () => {
  state.detailOpen = true;
  showDetail();
  $("detail_saved_sec").scrollIntoView({ block: "start" });
});

/// 열린 대화에 새 요청 — 입력창 · 다시 보내기. 보내는 대상은 지금 화면에 열린 대화(해시)이고, 레일에서 고른 상대와 맞아야 한다.
/// 글은 대화 에이전트에게 — 멘션이면 부른 에이전트에게. `opts` = `{request, agent}`(글 종류 · 멘션). `sent` = 보낸 뒤 할 일(입력창 비우기).
function sendText(text, opts, sent) {
  const r = route(location.hash);
  const conv = state.current;
  if (r.view !== "conversation" || conv?.id !== r.id || !keepsConversation(conv, state.railPeer)) {
    $("status_msg").textContent = "보내지 않음 — 열린 대화와 고른 PC 가 맞지 않습니다. 대화를 다시 여세요.";
    return;
  }
  if (!text.trim()) {
    $("status_msg").textContent = "보내지 않음 — 내용이 비었습니다";
    return;
  }
  if (state.sending) {
    $("status_msg").textContent = "보내는 중 — 앞 글을 보낸 뒤 다시 하세요";
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
      return res.warning ?? "보냄 — 답은 도착하면 보입니다";
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
  const conv = state.current;
  const names = mentionNames();
  const canMention = conv?.dir === "out" && !conv.relay_agent;
  // 멘션을 쓰는 대화인데 후보 목록이 없으면(받는 중 · 못 받음) 멘션처럼 보이는 글은 보내지 않는다 — 대화 에이전트로 조용히 가지 않게. 목록을 다시 묻는다.
  if (names == null && canMention && hasMention(text)) {
    $("status_msg").textContent = "보내지 않음 — 상대 에이전트 목록을 받지 못해 @멘션을 가릴 수 없습니다. 잠시 뒤 다시 보내세요";
    if (state.peerAgents.get(conv.peer_id) !== undefined || !state.peerAgents.has(conv.peer_id)) loadPeerAgents(conv.peer_id);
    return;
  }
  const agent = mentionTarget(text, names ?? []);
  // 멘션 꼴 `@이름` 이 후보와 하나도 맞지 않으면(오타 · 낡은 목록) 한 번 막고 알린다 — 목록을 다시 받고, 같은 글을 다시 보내면 멘션 없이 보낸다
  // (코드의 `@Override` 같은 글도 보낼 수 있게).
  if (canMention && names && !agent && hasMention(text) && state.mentionWarned !== text) {
    state.mentionWarned = text;
    $("status_msg").textContent = `보내지 않음 — ${mentionToken(text)}: 상대 에이전트 목록에 없는 이름입니다(목록을 다시 받습니다). 멘션이 아니면 한 번 더 보내세요`;
    loadPeerAgents(conv.peer_id);
    return;
  }
  state.mentionWarned = null;
  const opts = { request: ask.value === "request", agent };
  sendText(text, opts, () => {
    $("reply_text").value = "";
    ask.value = "query";
  });
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

/// 멘션 힌트 — 후보(상대 PC 에이전트 목록)를 받은 대화에서만 보인다.
function renderMentionHint() {
  $("mention_hint").hidden = mentionNames() == null;
}

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

// ---- 내 활동 · 상대와 주고받은 글 · 보낸 메시지 ----

function rowEls(rows, empty) {
  if (!rows.length) return [el("li", { class: "hint", text: empty })];
  return rows.map((r) =>
    el(
      "li",
      {},
      el(
        "a",
        { href: r.href },
        el("span", { class: "meta" }, Object.assign(tagEl(r.tag), { title: "대화 방향" }), el("strong", { text: r.who }), el("span", { text: r.kind }), el("span", { text: formatTime(r.at) })),
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
          Object.assign(tagEl(r.tag), { title: "대화 방향" }),
          el("strong", { text: r.who }),
          el("span", { text: r.label }),
          el("span", { text: formatTime(r.at) }),
          el("span", { class: "hint", text: r.title }),
        ),
        r.preview ? el("span", { text: r.preview }) : "",
      ),
    ),
  );
}

/// 내 활동 범위 — 고른 타일(상대 하나 · '나' = 전부). '나' 일 때만 상대 거르기를 쓴다.
function activityPeer() {
  return state.railPeer ?? ($("act_peer").value ? Number($("act_peer").value) : null);
}

/// 내 활동 — 열 때(`open`) 거르기를 처음으로 되돌리고(본 것으로 표시하는 범위 = 보이는 범위) 그 타일 범위를 본 것으로(탭이 보일 때만).
/// 그때 안 본 항목은 다시 그려도 강조를 남긴다. 여는 동안의 새로 고침은 건너뛴다(늦게 온 것으로 버려져 표시를 놓치지 않게).
async function loadActivity(open = false) {
  if (!open && state.actOpening) return;
  if (open) {
    state.actOpening = true;
    $("act_kind").value = "all";
    $("act_q").value = "";
    $("act_peer").value = "";
  }
  try {
    $("act_peer_box").hidden = state.railPeer != null;
    const params = new URLSearchParams({ kind: $("act_kind").value });
    const peer = activityPeer();
    if (peer != null) params.set("peer", peer);
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
    if (needsRender($("act_list"), JSON.stringify(rows), open)) $("act_list").replaceChildren(...activityEls(rows));
    if (open && document.visibilityState === "visible") {
      // 받은 범위(`upto`)까지만 — 받은 뒤 온 것은 안 본 것으로 남게. `peer` 가 없으면 전부.
      const seen = { ...r.upto, ...(state.railPeer != null ? { peer: state.railPeer } : {}) };
      await api("POST", "/api/activity/seen", seen).catch(() => {});
      await loadCounts();
    }
  } finally {
    if (open) state.actOpening = false;
  }
}

let actTimer;
for (const id of ["act_peer", "act_kind", "act_q"])
  $(id).addEventListener("input", () => {
    clearTimeout(actTimer);
    actTimer = setTimeout(() => loadActivity(), 250);
  });

/// 안 본 내 활동 수 — 사이드바 내비(고른 타일 범위) · 종 점(전부).
async function loadCounts() {
  const peer = state.railPeer;
  let r;
  try {
    r = await api("GET", "/api/activity?limit=0" + (peer != null ? `&peer=${peer}` : ""));
  } catch {
    return; // 다음 새로 고침에 다시
  }
  if (state.railPeer !== peer) return; // 그 사이 다른 타일을 골랐다
  state.unseen = peer != null ? r.unseen : r.unseen_all;
  state.unseenAll = r.unseen_all;
  $("bell_dot").hidden = state.unseenAll === 0;
  renderSideNav();
}

/// 저장된 항목 — 고른 타일 범위, 저장 순 최신부터, 누르면 그 글로.
async function loadSaved() {
  const ticket = latest.next();
  let ms;
  try {
    ms = await api("GET", "/api/saved" + (state.railPeer != null ? `?peer=${state.railPeer}` : ""));
  } catch (e) {
    $("status_msg").textContent = "저장된 항목 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const rows = inboxRows(ms, state.peerNames).map((r) => ({ ...r, href: `${r.href}/m/${r.id}` }));
  if (needsRender($("saved_list"), JSON.stringify([state.railPeer, rows]), false))
    $("saved_list").replaceChildren(...rowEls(rows, "저장된 항목 없음 — 메시지 작업 '저장' 으로 모읍니다"));
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
  $("agents_title").textContent = p ? `${displayName(p)}의 에이전트` : "에이전트";
  const row = (name, sub, props) =>
    el(
      "li",
      {},
      el(
        "a",
        props,
        el("span", { class: "meta" }, el("span", { class: "agent-dot", "aria-hidden": "true" }), el("strong", { class: "agent-name", text: name })),
        el("span", { class: "hint", text: sub }),
      ),
    );
  const rows = p ? [] : state.agents.map((a) => row(a.name, `내 에이전트 · ${a.folder}`, { href: "#settings/agents" }));
  for (const x of peers) {
    const names = state.peerAgents.get(x.id);
    if (names === undefined) rows.push(el("li", { class: "hint", text: `${displayName(x)} — 불러오는 중 …` }));
    else if (names === null) rows.push(el("li", { class: "hint", text: `${displayName(x)} — 목록을 받지 못했습니다` }));
    else
      for (const n of names)
        rows.push(
          row(n, `${displayName(x)} 의 에이전트 — 누르면 새 대화`, {
            href: "#new",
            onclick: (ev) => {
              ev.preventDefault();
              newChat(x.id, n);
            },
          }),
        );
  }
  $("agents_list").replaceChildren(...(rows.length ? rows : [el("li", { class: "hint", text: "에이전트 없음" })]));
}

async function loadPeerMessages(id) {
  const p = state.peers.find((x) => x.id === id);
  $("peer_title").textContent = p ? `${displayName(p)} · ${p.host}` : `상대 ${id}`;
  const ticket = latest.next();
  let lists;
  try {
    lists = await Promise.all(["theirs", "mine"].map((side) => api("GET", `/api/messages?side=${side}&peer=${id}`)));
  } catch (e) {
    $("status_msg").textContent = "주고받은 글 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  const rows = inboxRows(peerMessages(...lists), state.peerNames);
  if (needsRender($("peer_list"), JSON.stringify([id, rows]), false)) $("peer_list").replaceChildren(...rowEls(rows, "주고받은 글 없음"));
}

async function openSent(conv) {
  const ticket = latest.next();
  // 종류 거르기(질의 · 요청)는 서버가 건수 제한 전에 한다.
  const ask = $("sent_ask").value;
  let ms;
  let convMs;
  try {
    [ms, convMs] = await Promise.all([
      api("GET", `/api/messages?side=mine&peer=${conv.peer_id}` + (ask ? `&ask=${ask}` : "")),
      api("GET", `/api/conversations/${conv.id}/messages`),
    ]);
  } catch (e) {
    $("status_msg").textContent = "보낸 메시지 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  setConvMessages(conv, convMs);
  renderDetail();
  const rows = inboxRows(ms, state.peerNames);
  if (needsRender($("sent_list"), JSON.stringify([conv.peer_id, ask, rows]), false)) $("sent_list").replaceChildren(...rowEls(rows, "보낸 메시지 없음"));
}
$("sent_ask").addEventListener("change", () => {
  if (state.current) openSent(state.current);
});

// ---- 새 대화 ----

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
  const actSel = $("act_peer");
  const keepAct = actSel.value;
  actSel.replaceChildren(el("option", { value: "", text: "전체" }), ...state.peers.map((p) => el("option", { value: p.id, text: `${displayName(p)} · ${p.host}` })));
  if (state.peers.some((p) => String(p.id) === keepAct)) actSel.value = keepAct;
}

const agentsAsk = new Latest();
/// 새 대화의 상대 에이전트 — 고른 상대 PC 에 묻는다. `want` 가 그 목록에 있으면 그것을 고른다.
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
    $("chat_msg").textContent = r.names.length ? "" : "상대 PC 에 등록된 에이전트가 없다";
  } catch (e) {
    $("chat_msg").textContent = "상대 에이전트 목록 실패: " + e.message;
  }
}
$("chat_peer").addEventListener("change", () => loadChatAgents());

$("chat_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const ask = $("chat_form").elements.ask;
  const body = {
    peer_id: Number($("chat_peer").value),
    agent: $("chat_agent").value,
    text: $("chat_text").value,
    title: $("chat_title").value,
    request: ask.value === "request",
  };
  if (!body.text.trim()) {
    $("chat_msg").textContent = "보내지 않음 — 내용이 비었습니다";
    return;
  }
  if ($("chat_relay_agent").value) {
    body.relay_agent = $("chat_relay_agent").value;
    body.relay_cap = Number($("chat_relay_cap").value);
  }
  act(
    "보내기",
    async () => {
      const r = await api("POST", "/api/conversations", body);
      $("chat_text").value = "";
      $("chat_title").value = "";
      ask.value = "query";
      await loadConvs();
      location.hash = `#c/${r.conversation.id}`;
      return r.warning ?? "보냄 — 답은 도착하면 보입니다";
    },
    "chat_msg",
  );
});

// ---- 설정(트리 · 실행 · 화면 · 알림) ----

function renderTree() {
  const item = (n) =>
    el(
      "li",
      {},
      el("a", {
        href: `#settings/${n.id}`,
        text: n.label,
        onclick: (ev) => {
          ev.preventDefault();
          goSettings(n.id);
        },
      }),
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
  if (s.notify_paused !== undefined || s.focus !== undefined) $("bell").classList.toggle("paused", s.notify_paused === true || s.focus === true);
  if (s.focus !== undefined) renderFocusChip();
  if (s.me_alias !== undefined || s.me_icon !== undefined) {
    renderRail();
    renderSideHead();
    renderAvatar();
  }
  // 새 대화의 상한 칸 처음 값.
  if (s.relay_cap_default !== undefined) $("chat_relay_cap").value = String(s.relay_cap_default);
}

$("claude_effort").replaceChildren(...EFFORTS.map((e) => el("option", { value: e, text: e })));

for (const key of PREFS) {
  const input = $(key);
  input.addEventListener("change", () => {
    const value = input.type === "checkbox" ? input.checked : input.type === "number" ? Number(input.value) : input.value.trim();
    input.disabled = true;
    act("설정", async () => {
      try {
        fillPrefs(await api("PUT", "/api/settings", { [key]: value }));
        if (key === "claude_model" || key === "claude_effort") renderPeerList(); // 상대별 칸의 "기본값(…)" 문구
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

async function showSettings() {
  const box = $("notify_inbound");
  try {
    const s = await api("GET", "/api/settings");
    box.checked = s.notify_inbound === true;
    fillPrefs(s);
    // 폭은 시작할 때만 — 다른 설정 저장 응답(fillPrefs)이 막 끈 폭을 이전 값으로 되돌리지 않게.
    setSideWidth(s.ui_side_width ?? 272);
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
        if (!box.checked) return "인바운드 알림 끔";
        const p = await permission;
        return p === "granted"
          ? "인바운드 알림 켬 — 브라우저 알림 · 트레이 표시"
          : "인바운드 알림 켬 — 브라우저 알림 불가(권한 거부 또는 미지원), 트레이 표시만 씁니다";
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

async function refresh() {
  await loadPeers();
  await loadConvs();
  const r = route(location.hash);
  if (state.current && r.view === "conversation") await openConversation(state.current);
  if (state.current && r.view === "sent") await openSent(state.current);
  if (r.view === "activity") await loadActivity();
  if (r.view === "saved") await loadSaved();
  if (r.view === "peer") await loadPeerMessages(r.id);
  await loadCounts();
}

/// 페어링 종류 알림 — 누르면 설정의 연결 목록, 그 밖(메시지 · 자동 왕복)은 내 활동.
const PAIRING_KINDS = new Set(["pair_request", "auto_paired", "approved"]);

/// 서버 → 웹UI 알림 스트림. 붙는 순간 트레이 "새 요청" 이 지워진다. 원격 이름은 텍스트로만(브라우저 알림 본문도 텍스트).
function listen() {
  const es = new EventSource("/api/events");
  // 붙은 뒤 "봤다" 를 알린다(트레이 새 요청 지움) — 상태 변경은 POST 로만.
  es.onopen = () => api("POST", "/api/seen").catch(() => {});
  es.onmessage = (ev) => {
    let n;
    try {
      n = JSON.parse(ev.data);
    } catch {
      return;
    }
    $("status_msg").textContent = `${n.title} — ${n.body.split("\n")[0]}`;
    // 종류와 무관하게 다시 그린다 — 연결 · 대화 · 내 활동 수가 함께 바뀐다(새로 고침이 연결 목록부터 읽는다).
    refresh();
    if ($("notify_inbound").checked && "Notification" in window && Notification.permission === "granted") {
      const shown = new Notification(n.title, { body: n.body, tag: "coworkagent-" + n.kind });
      shown.onclick = () => {
        window.focus();
        if (PAIRING_KINDS.has(n.kind)) {
          go("#settings/peers");
        } else {
          pickRail(null); // 내 활동 전부
          go("#activity");
        }
      };
    }
  };
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
    el("button", {
      type: "button",
      role: "menuitem",
      text: i.label,
      onclick: () => {
        closeMenus();
        state.focusAfter = i.focus;
        go(i.href);
      },
    }),
  ),
);
$("new_btn").addEventListener("click", () => toggleMenu("new_btn", "new_menu"));

/// 상단 바 상태 칩 — 내 집중(이 PC 설정). 누르면 바꾼다(프로필 메뉴 "집중 — 요청 받지 않음" 과 같은 값).
function renderFocusChip() {
  const on = state.settings.focus === true;
  $("focus_chip").textContent = on ? "집중" : "받기 가능";
  $("focus_chip").setAttribute("aria-pressed", String(on));
}

/// 집중 켜고 끄기 — 켜 있는 동안 상대 PC 의 새 요청을 받지 않고 알림을 멈춘다(알림 일시 중지 값은 그대로).
function setFocus(on) {
  act("집중", async () => {
    fillPrefs(await api("PUT", "/api/settings", { focus: on }));
    return on ? "집중 — 상대 PC 의 새 요청을 받지 않고 알림을 멈춥니다" : "받기 가능 — 상대 PC 의 요청을 다시 받습니다";
  });
}

$("focus_chip").addEventListener("click", () => setFocus(state.settings.focus !== true));

/// 프로필 메뉴 — 집중 · 알림 일시 중지의 켜짐은 설정 값으로(열 때마다 다시 그린다).
function renderProfileMenu() {
  $("profile_menu").replaceChildren(
    ...profileMenu(state.settings.notify_paused, state.settings.focus).map((i) =>
      el("button", {
        type: "button",
        role: i.checked === undefined ? "menuitem" : "menuitemcheckbox",
        "aria-checked": i.checked === undefined ? null : String(i.checked),
        text: i.label,
        onclick: () => profileAct(i),
      }),
    ),
  );
}

function profileAct(i) {
  closeMenus();
  if (i.key === "settings") {
    go(i.href);
  } else if (i.key === "tile") {
    pickTile(null); // '나' 타일 — 사이드바 머리의 편집 폼
    openTileEdit();
    // 설정 화면에서 왔으면 사이드바가 아직 숨어 있다 — 안내 화면을 그린 뒤 초점.
    if ($("sidebar").hidden) state.focusAfter = "tile_alias";
  } else if (i.key === "focus") {
    setFocus(!i.checked);
  } else {
    // 알림 일시 중지(이 PC 에만) — 켜 있는 동안 브라우저 알림 · 트레이 표시를 내지 않는다, 풀면 알림 설정대로.
    const on = !i.checked;
    act("알림 일시 중지", async () => {
      fillPrefs(await api("PUT", "/api/settings", { notify_paused: on }));
      return on ? "알림 일시 중지 — 다시 켜기 전까지 브라우저 알림 · 트레이 표시를 내지 않습니다" : "알림 일시 중지 풀림";
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

/// 검색 결과로 — 메시지 = 그 글 위치, 대화 = 그 대화, 사람 = 그 타일, 상대 에이전트 = 새 대화 채움, 내 에이전트 = 설정.
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
await showSettings(); // 테마 · 기본값을 먼저 — 연결 목록의 "기본값(…)" 문구가 쓴다
await Promise.all([showStatus(), showMe(), loadPeers(), loadAgents(), loadConvs()]);
await render();
await loadCounts();
listen();
// ponytail: 알림이 꺼져 있으면 이벤트가 오지 않으므로 5초마다 다시 읽는다 — 대화가 많아지면 대화 이벤트 스트림으로 바꾼다.
setInterval(refresh, 5000);
