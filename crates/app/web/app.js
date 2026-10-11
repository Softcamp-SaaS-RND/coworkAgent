// 화면 조립 — 시안 A(레일 · 사이드바 · 대화) + C(타임라인) + 설정. 값은 textContent 로만 넣는다(HTML 주입 없음) — 원격에서 온
// 이름 · 본문도 같다. 판단 로직은 view.js(node 테스트).
import {
  stateText,
  peerState,
  convLabel,
  filterByPeer,
  authorLabel,
  relayText,
  keepsConversation,
  formatDuration,
  formatTime,
  timeline,
  unlinkedReplies,
  route,
  Latest,
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
  const data = text ? JSON.parse(text) : null;
  if (!res.ok) throw new Error(data?.error ?? "HTTP " + res.status);
  return data;
}

/// 요소 만들기 — `text` 는 textContent, `class` · 속성, `on*` 는 이벤트.
function el(tag, props = {}, ...children) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (v == null) continue;
    if (k === "text") e.textContent = v;
    else if (k === "class") e.className = v;
    else if (k.startsWith("on")) e.addEventListener(k.slice(2), v);
    else e.setAttribute(k, v);
  }
  e.append(...children);
  return e;
}

function button(label, onClick) {
  return el("button", { type: "button", text: label, onclick: onClick });
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
  peers: [],
  peerNames: new Map(),
  convs: [],
  railPeer: null, // 레일에서 고른 상대(null = 내 PC = 전부)
  current: null, // 열린 대화(요약)
};
const latest = new Latest();

// ---- 상태 · 내 정보 ----

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
    const me = await api("GET", "/api/me");
    $("me_name").textContent = me.name;
    $("me_host").textContent = me.host;
    $("me_fingerprint").textContent = me.fingerprint;
    $("me_addrs").textContent = me.addrs.length ? me.addrs.join(", ") : "포트 " + me.port;
    $("side_name").textContent = me.host;
    $("side_sub").textContent = `${me.name} · 지문 ${me.fingerprint.split(" ").slice(0, 2).join("·")}`;
  } catch (e) {
    for (const id of ["me_name", "me_host", "me_fingerprint", "me_addrs", "side_name"]) $(id).textContent = "오류: " + e.message;
  }
}

// ---- 피어 · 레일 ----

async function loadPeers() {
  try {
    state.peers = await api("GET", "/api/peers");
  } catch (e) {
    $("peer_msg").textContent = "피어 목록 오류: " + e.message;
    return;
  }
  state.peerNames = new Map(state.peers.map((p) => [p.id, `${p.name} · ${p.host}`]));
  renderRail();
  renderSidePeers();
  renderPeerList();
  fillPeerSelect();
}

const RAIL_COLORS = ["#4f6f8f", "#5e7d4f", "#7a5a3f", "#6b4f7a", "#3f6f6b"];

function renderRail() {
  const paired = state.peers.filter((p) => p.paired);
  if (state.railPeer != null && !paired.some((p) => p.id === state.railPeer)) state.railPeer = null;
  const pc = (label, short, bg, id) =>
    el("button", {
      type: "button",
      class: "rail-btn",
      "aria-label": label,
      "aria-pressed": String(state.railPeer === id),
      style: `background: ${bg}`,
      text: short,
      onclick: () => {
        state.railPeer = id;
        renderRail();
        // 다른 PC 를 고르면 열린 대화를 닫는다 — 입력창이 이전 상대를 가리킨 채 남지 않게.
        if (state.current && !keepsConversation(state.current, id)) {
          state.current = null;
          location.hash = "#";
        }
        renderConvList();
      },
    });
  $("rail_pcs").replaceChildren(
    pc("내 PC — 모든 대화", "나", "#c4622d", null),
    ...paired.map((p, i) => pc(`${p.name} · ${p.host} 의 대화만`, p.name.slice(0, 2), RAIL_COLORS[i % RAIL_COLORS.length], p.id)),
  );
}

function renderSidePeers() {
  $("side_peers").replaceChildren(
    ...state.peers.map((p) =>
      el(
        "li",
        {},
        el("span", { class: "dot " + (p.paired ? "ok" : "wait"), "aria-hidden": "true" }),
        el("span", { text: `${p.name} · ${p.host}` }),
        el("span", { class: "meta", text: peerState(p) }),
      ),
    ),
  );
}

function renderPeerList() {
  const peerAct = (label, fn) => act(label, fn, "peer_msg", loadPeers);
  $("peer_list").replaceChildren(
    ...state.peers.map((p) => {
      const li = el(
        "li",
        {},
        el("strong", { text: `${p.name} · ${p.host}` }),
        el("span", { text: ` ${p.addr} — ${peerState(p)}` }),
        el("div", { class: "fp", text: p.fingerprint }),
      );
      if (!p.my_ok) li.append(button("승인", () => peerAct("승인", () => api("POST", `/api/peers/${p.id}/approve`))));
      if (p.my_ok && !p.paired)
        li.append(button("다시 알리기", () => peerAct("승인 재통지", () => api("POST", `/api/peers/${p.id}/approve`))));
      if (p.paired) li.append(button("연결 확인", () => peerAct("연결 확인", () => api("POST", `/api/peers/${p.id}/ping`))));
      li.append(
        button("삭제", () =>
          peerAct("삭제", async () => {
            const r = await api("DELETE", `/api/peers/${p.id}`);
            if (!r.notified) throw new Error("내 쪽은 삭제했지만 상대에 알리지 못했다 — 상대 목록에는 남아 있다");
          }),
        ),
      );
      return li;
    }),
  );
}

$("rail_add").addEventListener("click", () => {
  location.hash = "#settings";
  $("invite_text").focus();
});
$("rail_settings").addEventListener("click", () => {
  location.hash = "#settings";
});

$("window_btn").addEventListener("click", () =>
  act(
    "페어링 받기",
    async () => {
      const r = await api("POST", "/api/pairing-window");
      $("invite").textContent = r.invite;
      $("invite_box").hidden = false;
      return `페어링 받기 — ${r.seconds / 60}분 동안 이 초대로 들어온 요청 1건`;
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
      return "초대를 복사했다 — 메신저에 붙여 넣어 보내세요";
    },
    "peer_msg",
  ),
);

$("invite_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const invite = $("invite_text").value;
  act(
    "초대로 추가",
    async () => {
      await api("POST", "/api/peers", { invite });
      $("invite_text").value = "";
      return "추가 · 승인 완료 — 상대가 승인하면 페어링됩니다";
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

async function loadAgents() {
  let agents;
  try {
    agents = await api("GET", "/api/agents");
  } catch (e) {
    $("agent_msg").textContent = "에이전트 목록 오류: " + e.message;
    return;
  }
  // 새 대화의 "내 에이전트로 자동 왕복" 후보.
  const relaySel = $("chat_relay_agent");
  const keep = relaySel.value;
  relaySel.replaceChildren(
    el("option", { value: "", text: "안 함 — 상대 에이전트에게만" }),
    ...agents.map((a) => el("option", { value: a.name, text: a.name })),
  );
  if (agents.some((a) => a.name === keep)) relaySel.value = keep;
  $("side_agents").replaceChildren(
    ...agents.map((a) =>
      el(
        "li",
        {},
        el("span", { class: "dot ok", "aria-hidden": "true" }),
        el("span", { text: a.name }),
        el("span", { class: "meta", text: a.folder, title: a.folder }),
      ),
    ),
  );
  if (!agents.length) $("side_agents").append(el("li", { class: "hint", text: "설정 → 내 에이전트에서 등록" }));
  $("agent_list").replaceChildren(
    ...agents.map((a) => {
      const li = el("li", {}, el("strong", { text: a.name }), el("span", { text: ` ${a.folder} — 도구 ${a.tools}` }));
      if (a.prompt) li.append(el("div", { class: "hint", text: a.prompt }));
      li.append(button("삭제", () => act("삭제", () => api("DELETE", `/api/agents/${a.id}`), "agent_msg", loadAgents)));
      return li;
    }),
  );
}

$("agent_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = {
    name: $("agent_name").value,
    folder: $("agent_folder").value,
    tools: $("agent_tools").value,
    prompt: $("agent_prompt").value,
  };
  act(
    "등록",
    async () => {
      await api("POST", "/api/agents", body);
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
}

function renderConvList() {
  const convs = filterByPeer(state.convs, state.railPeer);
  $("conv_list").replaceChildren(
    ...convs.map((c) => {
      const label = convLabel(c, state.peerNames);
      return el(
        "li",
        { class: state.current?.id === c.id ? "current" : "" },
        el("a", { href: `#c/${c.id}` }, el("span", { class: "title", text: label.title }), el("span", { class: "sub", text: label.sub })),
        c.last_state ? el("span", { class: "badge " + c.last_state, text: stateText(c.last_state) }) : "",
      );
    }),
  );
  if (!convs.length) $("conv_list").append(el("li", { class: "hint", text: "대화 없음" }));
}

// ---- 화면 전환 ----

const VIEWS = ["home", "conversation", "timeline", "new", "settings"];

function show(view) {
  for (const v of VIEWS) $(v).hidden = v !== view;
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
  if (r.view === "conversation" || r.view === "timeline") {
    const conv = await findConv(r.id);
    if (!conv) {
      $("status_msg").textContent = `대화 ${r.id} 를 찾지 못했다`;
      show("home");
      return;
    }
    state.current = conv;
    renderConvList();
    if (r.view === "conversation") await openConversation(conv);
    else await openTimeline(conv);
    return;
  }
  state.current = null;
  renderConvList();
  show(r.view);
  if (r.view === "settings" && r.focus === "me") $("me").scrollIntoView();
}

// ---- 대화(A) ----

function messageEl(conv, m) {
  const who = authorLabel(conv, m, state.peerNames.get(conv.peer_id));
  const meta = [formatTime(m.created_at)];
  if (m.state !== "done") meta.push(stateText(m.state));
  if (m.author === "agent" && (m.turns != null || m.duration_ms != null)) {
    meta.push([m.turns != null ? `${m.turns}턴` : "", formatDuration(m.duration_ms)].filter(Boolean).join(" · "));
  }
  return el(
    "article",
    { class: "msg " + m.author },
    el("div", { class: "avatar", "aria-hidden": "true", text: who.slice(0, 1) }),
    el(
      "div",
      { class: "msg-body" },
      el(
        "div",
        { class: "msg-meta" },
        el("strong", { text: who }),
        el("span", { class: "kind", text: m.author === "agent" ? "에이전트" : m.exec ? "사람 → 에이전트" : "사람" }),
        el("span", { text: meta.filter(Boolean).join(" · ") }),
      ),
      el("p", { class: "msg-text", text: m.text }),
    ),
  );
}

async function openConversation(conv) {
  show("conversation");
  const label = convLabel(conv, state.peerNames);
  $("conv_title").textContent = label.title;
  $("conv_sub").textContent = label.sub;
  $("conv_timeline_link").href = `#t/${conv.id}`;
  $("reply_agent_label").textContent =
    conv.dir === "out" ? `에이전트에게 — ${authorLabel(conv, { author: "agent" }, state.peerNames.get(conv.peer_id))} 가 실행` : `에이전트에게 — 내 ${conv.agent} 가 실행`;
  const box = $("messages");
  // 다른 대화로 바꾸면 이전 메시지를 먼저 지운다 — 제목과 본문이 다른 대화를 가리키지 않게.
  if (box.dataset.conv !== String(conv.id)) box.replaceChildren();
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
  showRelay(conv, ms);
  const atBottom = box.scrollHeight - box.scrollTop - box.clientHeight < 40;
  box.replaceChildren(...ms.map((m) => messageEl(conv, m)));
  if (atBottom) box.scrollTop = box.scrollHeight;
}

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

$("conv_relay_btn").addEventListener("click", () => {
  const conv = state.current;
  if (!conv) return;
  const on = $("conv_relay_btn").dataset.on === "true";
  act(
    on ? "재개" : "멈춤",
    async () => {
      await api("POST", `/api/conversations/${conv.id}/relay`, { on });
      return on ? "자동 왕복 재개" : "자동 왕복 멈춤";
    },
    "status_msg",
    refresh,
  );
});

$("reply_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  // 보내는 대상은 지금 화면에 열린 대화(해시)이고, 레일에서 고른 상대와 맞아야 한다.
  const r = route(location.hash);
  const conv = state.current;
  const text = $("reply_text").value;
  const to = $("reply_to_human").checked ? "human" : "agent";
  if (r.view !== "conversation" || conv?.id !== r.id || !keepsConversation(conv, state.railPeer)) {
    $("status_msg").textContent = "보내지 않음 — 열린 대화와 고른 PC 가 맞지 않습니다. 대화를 다시 여세요.";
    return;
  }
  const id = conv.id;
  act(
    "보내기",
    async () => {
      const r = await api("POST", `/api/conversations/${id}/messages`, { text, to });
      $("reply_text").value = "";
      return r.warning ?? "보냄 — 답은 도착하면 보입니다";
    },
    "status_msg",
    refresh,
  );
});

// ---- 타임라인(C) ----

async function openTimeline(conv) {
  show("timeline");
  const label = convLabel(conv, state.peerNames);
  $("tl_back").href = `#c/${conv.id}`;
  $("tl_kicker").textContent = conv.dir === "out" ? "보낸 요청" : "받은 요청";
  $("tl_title").textContent = label.title;
  $("tl_sub").textContent = label.sub;
  const ticket = latest.next();
  let ms;
  try {
    ms = await api("GET", `/api/conversations/${conv.id}/messages`);
  } catch (e) {
    $("status_msg").textContent = "타임라인 오류: " + e.message;
    return;
  }
  if (!latest.is(ticket)) return;
  $("tl_list").replaceChildren(
    ...timeline(ms).map((item) =>
      el(
        "li",
        { class: "tl-req" },
        ...item.steps.map((s) =>
          el(
            "div",
            { class: `tl-step ${s.kind} ${s.state ?? ""}` },
            el(
              "div",
              { class: "head" },
              el("strong", { text: s.title }),
              el("span", { text: [formatTime(s.at), s.meta].filter(Boolean).join(" · ") }),
            ),
            s.text ? el("p", { class: "text", text: s.text }) : "",
          ),
        ),
      ),
    ),
  );
  const loose = unlinkedReplies(ms);
  if (loose.length) {
    $("tl_list").append(
      el(
        "li",
        { class: "tl-req" },
        el("div", { class: "tl-step" }, el("div", { class: "head" }, el("strong", { text: "어느 요청의 답인지 모르는 이전 이력 답" }))),
        ...loose.map((m) =>
          el(
            "div",
            { class: `tl-step reply ${m.state}` },
            el("div", { class: "head" }, el("span", { text: [formatTime(m.created_at), stateText(m.state)].join(" · ") })),
            el("p", { class: "text", text: m.text }),
          ),
        ),
      ),
    );
  }
  if (!ms.length) $("tl_list").append(el("li", { class: "hint", text: "요청 없음" }));
}

// ---- 새 대화 ----

function fillPeerSelect() {
  const sel = $("chat_peer");
  const keep = sel.value;
  const paired = state.peers.filter((p) => p.paired);
  sel.replaceChildren(
    el("option", { value: "", text: paired.length ? "상대 선택" : "페어링된 상대 없음" }),
    ...paired.map((p) => el("option", { value: p.id, text: `${p.name} · ${p.host}` })),
  );
  if (paired.some((p) => String(p.id) === keep)) sel.value = keep;
}

$("chat_peer").addEventListener("change", async () => {
  const id = $("chat_peer").value;
  const sel = $("chat_agent");
  sel.replaceChildren();
  if (!id) return;
  $("chat_msg").textContent = "상대 에이전트 불러오는 중 …";
  try {
    const r = await api("GET", `/api/peers/${id}/agents`);
    sel.replaceChildren(...r.names.map((n) => el("option", { value: n, text: n })));
    $("chat_msg").textContent = r.names.length ? "" : "상대 PC 에 등록된 에이전트가 없다";
  } catch (e) {
    $("chat_msg").textContent = "상대 에이전트 목록 실패: " + e.message;
  }
});

$("chat_form").addEventListener("submit", (ev) => {
  ev.preventDefault();
  const body = { peer_id: Number($("chat_peer").value), agent: $("chat_agent").value, text: $("chat_text").value };
  if ($("chat_relay_agent").value) {
    body.relay_agent = $("chat_relay_agent").value;
    body.relay_cap = Number($("chat_relay_cap").value);
  }
  act(
    "보내기",
    async () => {
      const r = await api("POST", "/api/conversations", body);
      $("chat_text").value = "";
      await loadConvs();
      location.hash = `#c/${r.conversation.id}`;
      return "보냄 — 답은 도착하면 보입니다";
    },
    "chat_msg",
  );
});

// ---- 알림 설정 ----

/// 알림 세부 설정 · 자동 왕복 상한 기본값 — 입력 id = 설정 이름.
const PREFS = ["notify_pair", "notify_request", "notify_reply", "notify_relay", "notify_preview", "relay_cap_default"];

function fillPrefs(s) {
  for (const key of PREFS) {
    if (s[key] === undefined) continue;
    const input = $(key);
    if (input.type === "checkbox") input.checked = s[key] === true;
    else input.value = String(s[key]);
  }
  // 새 대화의 상한 칸 처음 값.
  if (s.relay_cap_default !== undefined) $("chat_relay_cap").value = String(s.relay_cap_default);
}

for (const key of PREFS) {
  const input = $(key);
  input.addEventListener("change", () => {
    const value = input.type === "checkbox" ? input.checked : input.type === "number" ? Number(input.value) : input.value;
    input.disabled = true;
    act("알림 설정", async () => {
      try {
        fillPrefs(await api("PUT", "/api/settings", { [key]: value }));
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
  await loadConvs();
  const r = route(location.hash);
  if (state.current && r.view === "conversation") await openConversation(state.current);
  if (state.current && r.view === "timeline") await openTimeline(state.current);
}

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
    loadPeers();
    if (n.kind === "message") refresh();
    if ($("notify_inbound").checked && "Notification" in window && Notification.permission === "granted") {
      const shown = new Notification(n.title, { body: n.body, tag: "coworkagent-" + n.kind });
      shown.onclick = () => {
        window.focus();
        if (n.kind !== "message") location.hash = "#settings";
      };
    }
  };
}

window.addEventListener("hashchange", render);
await Promise.all([showStatus(), showMe(), loadPeers(), loadAgents(), showSettings(), loadConvs()]);
await render();
listen();
// ponytail: 알림이 꺼져 있으면 이벤트가 오지 않으므로 5초마다 다시 읽는다 — 대화가 많아지면 대화 이벤트 스트림으로 바꾼다.
setInterval(refresh, 5000);
