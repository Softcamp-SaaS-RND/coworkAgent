// 개요 — 목록 · 미리보기 · 미리보기 크기.
import { api } from "./core/api.js";
import { $, button, el, needsRender } from "./core/dom.js";
import { previewLatest, state } from "./core/state.js";
import { kindChip, rowTile, speakerEl } from "./core/widgets.js";
import { KIND_NAMES, convLabel, convMeta } from "./logic/conv.js";
import { displayName, formatTime, stateText } from "./logic/format.js";
import { PREVIEW_MSGS, previewBubbles, previewKey, previewSections, previewZoom } from "./logic/home.js";
import { peerSummary, presence } from "./logic/peer.js";
import { groupPeers } from "./logic/rail.js";
import { SESSION_NEW, route } from "./logic/shell.js";
import { SendLatest } from "./logic/sync.js";
import { newChat } from "./compose.js";
import { pickRail, pickTile, railGroupOf, scopedConvs } from "./rail.js";
import { foldEl, pairedIds } from "./sidebar.js";

/// '나' 개요(목록 · 미리보기)에서 대화를 열면 레일 · 사이드바도 그 상대로 — 상대 · 그룹 개요는 그대로.
export function followFromHome(c) {
  if (state.railPeer == null && state.railGroup == null) pickRail(c.peer_id);
}

/// 개요 화면이면 다시 그린다.
export function refreshHome() {
  if (route(location.hash).view === "home") renderHome();
}

/// 개요 = 상대별 요약 — 목록 보기는 머리(상대 타일 · 이름 · 안 읽음 · 대기 요청) + 그 상대 대화(누르면 그 대화), 미리보기는 `renderHomePreview`.
/// 페어링된 상대가 없고 대화도 없으면 처음 안내.
export function renderHome() {
  const group = railGroupOf();
  const convs = scopedConvs();
  // 그룹 요약은 그 그룹에 대화가 있는 상대만 — '나' 는 대화 없는 페어링 상대도, 상대 타일은 그 상대 하나.
  const peer = state.peers.find((x) => x.id === state.railPeer);
  const ids = group ? groupPeers(convs, pairedIds()) : state.railPeer != null ? [state.railPeer] : pairedIds();
  const groups = peerSummary(convs, ids, state.peerNames, state.waitingBy);
  $("home_title").textContent = group ? `그룹 · ${group.name}` : peer ? `개요 · ${displayName(peer)}` : "개요";
  $("group_empty").hidden = !group || convs.length > 0;
  $("home_intro").hidden = groups.length > 0 || group != null; // 두 보기 공통
  // 보기 — 미리보기(기본) · 목록(아래 접는 절).
  const pv = state.homeView === "preview";
  $("home_view_list").setAttribute("aria-pressed", String(!pv));
  $("home_view_preview").setAttribute("aria-pressed", String(pv));
  $("home_summary").hidden = pv;
  $("home_preview").hidden = !pv;
  $("home_zoom_pick").hidden = !pv; // 미리보기 크기는 미리보기에서만
  if (pv) return renderHomePreview(groups, !group && state.railPeer != null);
  const peerOf = (id) => state.peers.find((x) => x.id === id);
  const sig = JSON.stringify([groups.map(({ rows, ...g }) => [g, rows.map((c) => [c.id, c.title, c.last_state, c.unread, c.last_at, c.starred])]), groups.map((g) => presence(peerOf(g.peer)).text)]);
  if (!needsRender($("home_summary"), sig, false)) return;
  $("home_summary").replaceChildren(
    ...groups.map((g) => {
      const p = peerOf(g.peer);
      // 머리 = 그 상대 타일(머리글자 · 그림) · 줄 = 에이전트 로봇 아이콘 — 사람 · 에이전트를 가른다.
      const head = [
        rowTile("small", p?.icon, g.name),
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
              { class: meta.unread ? "unseen" : null, "data-kind": c.dir },
              el(
                "a",
                { href: `#c/${c.id}`, onclick: () => followFromHome(c) },
                kindChip(c, "home-kind"),
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

/// 개요 미리보기 — 상대마다 절(머리 + 대화방마다 축소 채팅 카드), 카드는 그 대화의 마지막 글(`PREVIEW_MSGS`, 서버가 SQL 에서 자름)을
/// 말풍선으로. '나' · 그룹은 절마다 한 줄(열 수 = 실제 격자에서 잰 `state.previewCols`) + 넘치면 "+n개", 상대 타일은 전부. 다시 읽기는 필요한 대화만
/// (5초마다 전부 읽지 않는다): 받은 적 없음 · 요약이 바뀜(`previewKey`) · 변경 신호가 그 대화를 가리킴(`previewStale`) · 개요에 들어옴 ·
/// 보기 바꿈(`previewAll`). 실패한 대화는 다음에 다시. 서명 = 실제로 그릴 말풍선(실패하면 지난 글로 그리고 다음 성공 때 바뀐다). 늦게 온 응답은 차례(`seq`)가
/// 앞선 것으로 덮지 않고, 다른 화면이면 그리지 않는다. 읽음은 바꾸지 않는다(대화를 열어야 읽음).
// ponytail: 이 PC 에서 마지막이 아닌 글만 바뀌는 경우(신호 없음 · 요약 그대로)는 개요에 다시 들어올 때 보인다 — 이 PC 변경 신호가 생기면 그것으로(D39 ②).
export async function renderHomePreview(groups, single) {
  const secs = previewSections(groups, single, state.previewCols);
  const ticket = previewLatest.next();
  const all = state.previewAll;
  const stale = state.previewStale;
  state.previewAll = false;
  state.previewStale = new Set();
  const convs = secs.flatMap((s) => s.cards).filter((c) => all || stale.has(c.id) || state.previewMsgs.get(c.id)?.key !== previewKey(c));
  const seq = ticket;
  const got = await Promise.allSettled(convs.map((c) => api("GET", `/api/conversations/${c.id}/messages?limit=${PREVIEW_MSGS}`)));
  got.forEach((r, i) => {
    const c = convs[i];
    if (r.status !== "fulfilled") state.previewStale.add(c.id);
    else if ((state.previewMsgs.get(c.id)?.seq ?? -1) < seq) state.previewMsgs.set(c.id, { key: previewKey(c), ms: r.value, seq });
  });
  // 이번에 보이지 않은 대화의 표시(한 줄 밖 · 다른 화면)는 남긴다 — 보일 때 다시 읽는다.
  for (const id of stale) if (!convs.some((c) => c.id === id)) state.previewStale.add(id);
  // 전체 무효화(개요에 들어옴 · 다시 붙음 · 보기 바꿈)는 한 줄 밖에 숨은 카드의 받아 둔 글에도 — 다시 보일 때 읽는다.
  if (all) for (const id of state.previewMsgs.keys()) if (!convs.some((c) => c.id === id)) state.previewStale.add(id);
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
    state.previewCols,
    secs.map((s) => [
      s.peer,
      s.name,
      s.convs,
      s.unread,
      s.waiting,
      s.more,
      s.cards.map((c) => [
        c.id,
        c.title,
        c.last_state,
        c.unread,
        c.last_at,
        convMeta(c).agent,
        state.previewMsgs.has(c.id) && previewBubbles(c, state.previewMsgs.get(c.id).ms, state.peerNames.get(c.peer_id)),
      ]),
      presence(peerOf(s.peer)).text,
      peerOf(s.peer)?.paired,
    ]),
  ]);
  const box = $("home_preview");
  if (needsRender(box, sig, false)) box.replaceChildren(...secs.map((s) => previewSectionEl(s, peerOf(s.peer))));
  // 한 줄 열 수 = 그린 격자의 실제 열(CSS `auto-fill`) — 처음 값과 다르면 그 수로 다시 그린다(창 폭이 바뀌면 `ResizeObserver` 가 다시 잰다).
  if (fitPreviewCols()) renderHome();
}

/// 그려진 카드 격자의 열 수를 `state.previewCols` 에 — 바뀌었으면 `true`. 격자가 없거나(대화 없음) 안 보이면 그대로.
export function fitPreviewCols() {
  const grid = $("home_preview").querySelector(".pv-cards");
  // 안 보이는 격자는 열을 풀지 않은 값(`repeat(…)`)을 준다 — 잰 트랙(`240px 240px …`)일 때만.
  const tracks = grid ? getComputedStyle(grid).gridTemplateColumns.split(" ") : [];
  if (!tracks.length || !tracks.every((t) => t.endsWith("px"))) return false;
  const n = tracks.length;
  if (n === state.previewCols) return false;
  state.previewCols = n;
  return true;
}
/// 개요 미리보기가 보이면 열 수를 다시 재고, 바뀌었으면 다시 그린다(창 폭 · 미리보기 크기).
export function refitPreview() {
  if (route(location.hash).view === "home" && state.homeView === "preview" && fitPreviewCols()) renderHome();
}
new ResizeObserver(refitPreview).observe($("home_preview"));

/// 미리보기 절 하나 — 머리(이름 · 접속 · 대화 수 · 안 읽음 · 대기 요청) + 카드 한 줄(끝 칸 "+n개 더 보기" = 그 상대 개요).
/// 대화 없는 상대 = 머리 한 줄 + "세션 생성"(페어링된 상대만).
export function previewSectionEl(s, p) {
  const pres = presence(p);
  const head = [
    el("strong", { text: s.name }),
    el("span", { class: "pv-pres" }, el("span", { class: `st-dot st-${pres.key}`, "aria-hidden": "true" }), el("span", { text: p ? pres.text : "페어링 해제" })),
    el("span", { text: s.convs ? `· 대화 ${s.convs}` : "· 대화 없음" }),
    s.unread ? el("span", { class: "stat", text: `· 안 읽음 ${s.unread}` }) : "",
    s.waiting ? el("span", { class: "stat pv-wait", text: `· 대기 요청 ${s.waiting}` }) : "",
  ];
  if (!s.cards.length) return el("section", { class: "pv-sec pv-none" }, el("div", { class: "pv-head" }, ...head, p?.paired ? button(SESSION_NEW, () => newChat(s.peer), "small") : ""));
  return el(
    "section",
    { class: "pv-sec" },
    el("div", { class: "pv-head" }, ...head),
    el(
      "div",
      { class: "pv-cards" },
      ...s.cards.map(previewCardEl),
      s.more
        ? el("button", { type: "button", class: "pv-more", title: `${s.name} 대화방 ${s.convs}개 모두 보기`, onclick: () => pickTile(s.peer), text: `+${s.more}개 더 보기` })
        : "",
    ),
  );
}

/// 미리보기 카드 하나(누르면 그 대화) — 머리(유형 아이콘 · 대화 제목 · "새 글 n") · 본문(마지막 글 말풍선, 최신이 아래) · 바닥("에이전트 · 시각" · 상태 점 + 글자).
/// 본문은 글자로만 — 상대가 쓴 글을 HTML · 마크다운으로 해석하지 않는다.
export function previewCardEl(c) {
  const title = convLabel(c, state.peerNames).title;
  const cached = state.previewMsgs.get(c.id);
  const bubbles = cached ? previewBubbles(c, cached.ms, state.peerNames.get(c.peer_id)) : [];
  return el(
    "a",
    { class: c.unread ? "pv-card unseen" : "pv-card", href: `#c/${c.id}`, "aria-label": `${[KIND_NAMES[c.dir], title].filter(Boolean).join(" · ")} 대화 열기`, onclick: () => followFromHome(c), "data-kind": c.dir },
    el(
      "div",
      { class: "pv-card-head" },
      kindChip(c, "pv-kind"),
      el("strong", { text: title }),
      c.unread ? el("span", { class: "pv-new", text: `새 글 ${c.unread}` }) : "",
    ),
    el(
      "div",
      { class: "pv-body" },
      ...(!cached
        ? [el("span", { class: "hint", text: "글을 읽지 못했습니다" })]
        : bubbles.length
          ? bubbles.map((b) => el("div", { class: `pv-msg side-${b.side} ${b.tone}` }, el("span", { class: "pv-text" }, b.who !== "나" ? el("b", { text: b.who }) : "", b.text)))
          : [el("span", { class: "hint", text: "글 없음" })]),
    ),
    el(
      "div",
      { class: "pv-card-foot" },
      el("span", { class: "pv-who", text: [convMeta(c).agent, c.last_at != null ? formatTime(c.last_at) : ""].filter(Boolean).join(" · ") }),
      c.last_state ? el("span", { class: `pv-state st-${c.last_state}`, text: stateText(c.last_state) }) : "",
    ),
  );
}

/// 개요 미리보기 크기 — 카드 폭 · 본문 높이(글자 크기는 그대로). 카드가 넓어져 한 줄 열 수가 바뀌면 그 수로 다시 그린다.
/// 값은 시작할 때(`showSettings`)와 고를 때만 — 다른 설정 저장의 늦은 응답(`fillPrefs`)이 막 고른 배율을 되돌리지 않게(`ui_home_view` 와 같은 규칙).
export function applyHomeZoom(z) {
  const { scale } = previewZoom(z);
  const box = $("home_preview");
  box.style.setProperty("--pv-scale", String(scale));
  $("ui_home_zoom").value = String(Math.round(scale * 100));
  refitPreview();
}
/// 배율 저장 — 한 번에 하나, 보내는 동안 또 고르면 마지막 값만 이어서(`SendLatest` — 동시 요청이 앞 값으로 끝나지 않게). 실패는 상태 줄, 화면은 고른 대로.
export const homeZoomSaver = new SendLatest((z) =>
  api("PUT", "/api/settings", { ui_home_zoom: z }).catch((err) => {
    $("status_msg").textContent = "미리보기 크기 저장 실패: " + err.message;
  }),
);
$("ui_home_zoom").addEventListener("change", (e) => {
  applyHomeZoom(e.target.value);
  homeZoomSaver.push(Number(e.target.value));
});

/// 개요 보기 바꾸기 — 바로 그리고 설정 `ui_home_view` 에 남긴다(실패는 상태 줄, 화면은 바꾼 대로).
export function setHomeView(v) {
  state.homeView = v;
  state.previewAll = true;
  renderHome();
  api("PUT", "/api/settings", { ui_home_view: v }).catch((e) => {
    $("status_msg").textContent = "보기 저장 실패: " + e.message;
  });
}
for (const v of ["list", "preview"]) $(`home_view_${v}`).addEventListener("click", () => setHomeView(v));
