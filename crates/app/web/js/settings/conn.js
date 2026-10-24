// 설정 · 연결 — 사용자 목록 · 상세 · 초대.
import { api } from "../core/api.js";
import { $, act, button, el, field, lead, needsRender } from "../core/dom.js";
import { state } from "../core/state.js";
import { rowTile, tileColor } from "../core/widgets.js";
import { convClosed, convLabel } from "../logic/conv.js";
import { displayName } from "../logic/format.js";
import { sortByPos } from "../logic/order.js";
import { colorOf, peerState, presence } from "../logic/peer.js";
import { groupSettingsList } from "../logic/rail.js";
import { EFFORTS, MODELS, draftOf, pruneDrafts } from "../logic/settings.js";
import { loadPeers } from "../rail.js";
import { renderGroupPage } from "./groups.js";
import { goSettings, mdFoot, mdHead, mdRow, pick, setPick } from "./index.js";

/// 연결 줄 부제 — `{이름} · {PC}` · 접속 상태(페어링됨이면 "방금 · n분 전 통신", 아니면 연결 단계).
export function peerSub(p) {
  return [p.alias ? `${p.name} · ${p.host}` : p.host, p.paired ? presence(p).text : peerState(p)].join(" · ");
}

/// 대화 서명 — 설정 목록 · 상세가 쓰는 칸만.
export function convSig() {
  return state.convs.map((c) => [c.id, c.peer_id, c.pos, c.title, c.agent, c.dir, convClosed(c)]);
}

/// 설정 연결 · 그룹설정 두 화면을 다시 그린다.
export function renderPeerList(force = false) {
  renderConnPage(force);
  renderGroupPage(force);
}

/// 연결 — 왼쪽 사용자 목록(검색 = 이름 · PC · 지문 · 대화 제목), 오른쪽 상세(사용자 · 초대 생성 · 초대 수락).
export function renderConnPage(force) {
  const box = $("conn");
  // 접속 상태(`online` · `focus`)는 서명에 넣지 않는다 — 30초 Ping 이 저장 전 입력을 지우지 않게. 시각이 흘러 바뀌는 글자 · 점만 제자리에서 고친다.
  const shown = state.peers.map(({ online, focus, ...p }) => p);
  const sig = JSON.stringify([shown, convSig(), $("peer_q").value, pick.conn, state.settings.claude_model, state.settings.claude_effort]);
  for (const n of box.querySelectorAll("[data-peer-sub]")) {
    const p = state.peers.find((x) => x.id === Number(n.dataset.peerSub));
    if (p) n.textContent = peerSub(p);
  }
  for (const n of box.querySelectorAll("[data-peer-dot]")) {
    const p = state.peers.find((x) => x.id === Number(n.dataset.peerDot));
    if (p) n.replaceWith(gsDot(p));
  }
  if (!needsRender(box, sig, force)) return;
  if (typeof pick.conn === "number" && !state.peers.some((p) => p.id === pick.conn)) pick.conn = null; // 지운 사용자
  pruneDrafts(gsDrafts, state.peers.map((p) => p.id));
  const q = $("peer_q").value;
  const convs = sortByPos(state.convs);
  const users = groupSettingsList([], state.peers, convs, q, state.peerNames).users.map((u) => state.peers.find((p) => p.id === u.id));
  $("conn_list").replaceChildren(
    ...users.map((p) => mdRow("conn", p.id, rowTile(`small ${tileColor(p)}`, p.icon), displayName(p), p.host, gsDot(p))),
    ...(users.length ? [] : [el("li", { class: "hint md-none", text: q ? "맞는 사용자 없음" : "연결 없음 — 초대 생성 · 초대 수락" })]),
  );
  const mode = pick.conn;
  $("conn_empty").hidden = mode != null;
  $("invite_make").hidden = mode !== "invite";
  $("invite_accept").hidden = mode !== "accept";
  $("accept_btn").setAttribute("aria-expanded", String(mode === "accept"));
  const p = typeof mode === "number" ? state.peers.find((x) => x.id === mode) : null;
  $("conn_user").hidden = !p;
  $("conn_user").replaceChildren(...(p ? userDetail(p, convs) : []));
}

/// 사용자 접속 점 · 글자 — 서명에 접속 상태가 없어(입력 중 다시 그리지 않게) `renderConnPage` 가 매번 이것만 고친다(시간이 흘러 끊김이 돼도 녹색이 남지 않게).
export function gsDot(p, pr = presence(p)) {
  const text = p.paired ? pr.text : peerState(p);
  return el("span", { class: "gs-state", "data-peer-dot": String(p.id) }, el("span", { class: `gs-dot ${pr.key}`, "aria-hidden": "true" }), el("span", { text }));
}

/// 설정 상세의 대화 줄 — 상대 타일 · 제목 · 부제 · 닫힌 대화 태그 + 끝 단추.
export function convLine(c, end = "") {
  const label = convLabel(c, state.peerNames);
  const closed = convClosed(c);
  const p = state.peers.find((x) => x.id === c.peer_id);
  return el(
    "li",
    { class: "md-conv" },
    rowTile(`small ${colorOf(c.peer_id)}`, p?.icon),
    el("span", { class: "md-row-text" }, el("strong", { text: label.title }), el("span", { class: "hint", text: label.sub })),
    closed ? el("span", { class: "chip", text: closed }) : "",
    end,
  );
}

/// 상세 줄 끝 작은 단추(넣기 · 빼기) — 읽는 이름에 대화 제목.
export function convAct(label, title, fn, icon) {
  return el("button", { type: "button", class: "small", "aria-label": `${title} ${label}`, onclick: fn }, ...(icon ? [lead(icon)] : []), label);
}

/// 사용자 상세의 저장 전 모델 · effort — 다른 사용자를 골랐다 돌아와도(다시 그려도) 상대 id 별로 들고 있다가 되살린다.
export const gsDrafts = new Map();

/// 연결 사용자 상세 — 머리(타일 · 이름 · 부제 · ✕) · 모델 · effort · 상태 · 지문 · 주소 · 대화 목록 · 단추(저장 · 승인 / 다시 알리기 / 연결 확인 ·
/// 내 에이전트 편집 · 삭제 · 닫기).
export function userDetail(p, convs) {
  const peerAct = (label, fn) => act(label, fn, "peer_msg", () => loadPeers(true));
  const defaults = state.settings;
  const draft = draftOf(gsDrafts, p.id, p.fingerprint);
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
  const keep = () => gsDrafts.set(p.id, { owner: p.fingerprint, model: model.value, effort: effort.value });
  model.addEventListener("change", keep);
  effort.addEventListener("change", keep);
  const acts = [
    button(
      "저장",
      () =>
        peerAct("연결 설정 저장", async () => {
          const sent = gsDrafts.get(p.id); // 보낸 때의 고친 값 — 응답을 기다리는 사이 또 고치면 새 객체라 남는다
          await api("PATCH", `/api/peers/${p.id}`, { model: model.value, effort: effort.value });
          if (gsDrafts.get(p.id) === sent) gsDrafts.delete(p.id); // 저장한 값이 서버 값 — 실패면 고친 값을 그대로 둔다
        }),
      "primary",
      "save",
    ),
  ];
  if (!p.my_ok) acts.push(button("승인", () => peerAct("승인", () => api("POST", `/api/peers/${p.id}/approve`)), undefined, "check"));
  if (p.my_ok && !p.paired) acts.push(button("다시 알리기", () => peerAct("승인 재통지", () => api("POST", `/api/peers/${p.id}/approve`)), undefined, "bell"));
  if (p.paired) acts.push(button("연결 확인", () => peerAct("연결 확인", () => api("POST", `/api/peers/${p.id}/ping`)), undefined, "refresh"));
  acts.push(
    button("내 에이전트 편집", () => goSettings("agents"), undefined, "pencil"),
    button(
      "삭제",
      () =>
        peerAct("삭제", async () => {
          const r = await api("DELETE", `/api/peers/${p.id}`);
          gsDrafts.delete(p.id);
          if (!r.notified) throw new Error("삭제했지만 상대에게 알리지 못했습니다. 상대 목록에는 남아 있습니다.");
        }),
      "danger",
      "trash",
    ),
  );
  const mine = convs.filter((c) => c.peer_id === p.id);
  return [
    mdHead("conn", rowTile(tileColor(p), p.icon), displayName(p), el("span", { class: "hint", "data-peer-sub": String(p.id), text: peerSub(p) })),
    el(
      "div",
      { class: "md-body" },
      el(
        "div",
        { class: "form-grid" },
        ...field("모델", model, model, "cpu"),
        ...field("effort", effort, effort, "sliders"),
        el("span", { class: "label" }, lead("activity"), "상태"),
        gsDot(p),
        el("span", { class: "label" }, lead("key"), "지문"),
        el("span", { class: "mono", text: p.fingerprint }),
        el("span", { class: "label" }, lead("globe"), "주소"),
        el("span", { class: "mono", text: p.addr ?? "" }),
      ),
      el("strong", { class: "md-sec" }, lead("chat"), `대화 ${mine.length}`),
      mine.length ? el("ul", { class: "md-convs" }, ...mine.map((c) => convLine(c))) : el("p", { class: "hint", text: "대화 없음" }),
    ),
    mdFoot("conn", ...acts),
  ];
}

$("peer_q").addEventListener("input", () => renderConnPage(true));

// 초대 생성 — 한 번에 하나(머리 단추 · 새로 만들기 모두 이 단추를 누른다). 도는 동안 온 누름은 버려 겹친 요청이 서버 코드를 바꿔
// 화면 초대와 어긋나는 일이 없다. `disabled` 가 아니라 `aria-disabled` — 새로 만들기가 이 단추에 주는 초점을 잃지 않게.
// 만든 초대는 연결 상세 칸.
export let inviteBusy = false;
$("window_btn").addEventListener("click", () => {
  if (inviteBusy) return;
  inviteBusy = true;
  $("window_btn").setAttribute("aria-disabled", "true");
  setPick("conn", "invite");
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
