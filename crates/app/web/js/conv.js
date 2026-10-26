// 대화방 — 머리 · 글 · 본문 뷰어 · 컨텍스트 · 제목 · 자동 왕복 · 관람자.
import { api } from "./core/api.js";
import { $, act, button, el, ico, unchanged } from "./core/dom.js";
import { closeFloat, focusAfterFloat, openFloat } from "./core/float.js";
import { latest, state } from "./core/state.js";
import { iconButton } from "./core/widgets.js";
import { authorLabel, bubbleSide } from "./logic/author.js";
import { contextBusy, contextLabel, contextNote, contextSize, ctxMenu } from "./logic/context.js";
import { KIND_NAMES, MAX_OBSERVERS, convClosed, convKindIcon, convLabel, shareCandidates, shareHead } from "./logic/conv.js";
import { dayBreaks, displayName, formatDuration, formatTime, josa, shortName, stateText } from "./logic/format.js";
import { htmlSrcdoc, isHtmlDoc, markdownBlocks } from "./logic/markdown.js";
import { editedTag, enterSends, lastRun, messageActions, messagesSigOf, noteCount, outboxText, progressText, refusedText, relayText } from "./logic/message.js";
import { initialOf, presence, tileTint } from "./logic/peer.js";
import { askTag, reqFeedback } from "./logic/request.js";
import { closeMention } from "./composer.js";
import { convEndForm } from "./conv-menu.js";
import { renderDetail } from "./detail.js";
import { refresh } from "./events.js";
import { loadPeerAgents } from "./rail.js";
import { pick } from "./settings/index.js";
import { show } from "./shell.js";
import { loadConvs, starButton } from "./sidebar.js";

export function inlineEls(list) {
  return list.map((x) =>
    x.t === "code" ? el("code", { text: x.v }) : x.t === "b" ? el("strong", { text: x.v }) : x.t === "i" ? el("em", { text: x.v }) : x.v,
  );
}

/// markdown 블록 → 요소(정해진 태그만, 값은 textContent).
export function markdownEl(text) {
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
export function bodyEl(text) {
  if (isHtmlDoc(text)) {
    const frame = el("iframe", { class: "html-doc", sandbox: "", title: "HTML 본문(격리)", referrerpolicy: "no-referrer" });
    frame.srcdoc = htmlSrcdoc(text);
    return frame;
  }
  return markdownEl(text);
}

/// 본문 = 뷰어 + 원문 — 머리글 "원문 보기" 버튼(`#conversation.show-raw`)이 CSS 로 둘 중 하나만 보인다(다시 그리지 않음 · iframe 유지).
export function bodyPair(text, cls = "") {
  const view = bodyEl(text);
  view.classList.add("body-view");
  if (cls) view.classList.add(cls);
  return [view, el("pre", { class: "raw", text })];
}

/// "대화 보기" · "원문 보기" — 둘 중 하나가 눌린 상태.
export function showRaw(on) {
  $("conversation").classList.toggle("show-raw", on);
  $("raw_btn").setAttribute("aria-pressed", String(on));
  $("view_btn").setAttribute("aria-pressed", String(!on));
}
$("raw_btn").addEventListener("click", () => showRaw(true));
$("view_btn").addEventListener("click", () => showRaw(false));

export async function findConv(id) {
  let conv = state.convs.find((c) => c.id === id);
  if (!conv) {
    await loadConvs();
    conv = state.convs.find((c) => c.id === id);
  }
  return conv;
}

/// 대화 머리글 — 유형 아이콘 · 제목 · ★ · 부제 `{상대} 의 {에이전트} · {model} · {effort}`(마지막 답의 값, 없으면 뺀다).
export function renderConvHead(conv) {
  const label = convLabel(conv, state.peerNames);
  $("conv_title").textContent = label.title;
  // 유형 아이콘 · 머리 선 색 — 공유 사본은 아이콘 없음.
  const kind = convKindIcon(conv);
  $("conv_title_line").closest(".conv-head").dataset.kind = conv.dir;
  Object.assign($("conv_kind"), { hidden: !kind, title: KIND_NAMES[conv.dir] ?? "" });
  $("conv_kind").setAttribute("aria-label", KIND_NAMES[conv.dir] ?? "");
  $("conv_kind").replaceChildren(...(kind ? [ico(kind)] : []));
  const run = state.msgsOf === conv.id ? lastRun(state.msgs, conv.dir !== "out") : "";
  // 닫힌 대화 — 나감 · 상대가 나감(공유 사본은 `shareHead` 가 말한다).
  const closed = conv.dir === "shared" ? "" : convClosed(conv);
  $("conv_sub").textContent = [label.sub, shareHead(conv), closed, run].filter(Boolean).join(" · ");
  $("star_btn").replaceWith(Object.assign(starButton(conv, label.title), { id: "star_btn" }));
  // 머리 "북마크 n" — 이 대화에서 북마크한 글 수, 누르면 세부정보 패널의 북마크 절.
  const saved = state.msgsOf === conv.id ? state.msgs.filter((m) => m.saved).length : 0;
  $("saved_n").textContent = saved ? String(saved) : "";
  const pr = presence(state.peers.find((p) => p.id === conv.peer_id));
  Object.assign($("conv_presence"), { textContent: pr.text, className: `presence ${pr.key}`, hidden: false });
  renderCtx(conv);
}

/// 머리 컨텍스트 표시 겸 단추 — "42k / 1M" · 60% 주황 · 80% 붉음 · 모름.
export function renderCtx(conv) {
  const ms = state.msgsOf === conv.id ? state.msgs : [];
  const lab = contextLabel(contextSize(ms));
  const b = $("ctx_btn");
  b.dataset.level = lab.level;
  $("ctx_text").textContent = lab.text;
  b.title = `컨텍스트 ${lab.text}`;
}

/// 압축 · 비우기 메뉴 — 고르면 확인 한 번.
export function openCtxMenu(conv, anchor) {
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

/// 압축 · 비우기 확인.
export function ctxConfirm(conv, op) {
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

/// 대화 머리글(대화 보기 · 원문 보기).
export function showConversation(conv) {
  show("conversation");
  editTitle(false);
  renderConvHead(conv);
  // 멘션 — 내가 연 대화(자동 왕복 아님)만. 후보 = 상대 PC 에이전트(타일을 고를 때와 같은 목록 — 없거나 못 받았으면 이때 다시 묻는다).
  const names = state.peerAgents.get(conv.peer_id);
  if (conv.dir === "out" && !conv.relay_agent && (!state.peerAgents.has(conv.peer_id) || names === null)) loadPeerAgents(conv.peer_id);
  closeMention();
  if (state.msgsOf !== conv.id) $("conv_relay").hidden = $("conv_relay_btn").hidden = true;
  // 공유 대화 · 닫힌 대화(나감 · 상대가 나감) — 보기만: 입력창 · 컨텍스트 단추 없음.
  const readOnly = conv.dir === "shared" || !!convClosed(conv);
  $("reply_form").hidden = $("ctx_btn").hidden = readOnly;
}

/// 실행 중 경과 — 받은 때부터(대기 포함, 실행 시작 시각은 남지 않는다).
export function elapsed(since) {
  return `실행 중 · ${Math.max(0, Math.round(Date.now() / 1000 - since))}초`;
}

export function messageEl(conv, m) {
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
  // 말풍선 — 이 PC 쪽 글은 오른쪽(이름은 "나" 가 아닐 때만), 상대 쪽은 왼쪽에 머리글자 아바타 · 이름.
  // 에이전트 글은 어느 쪽이든 로봇 아바타 + "에이전트" 글자(말풍선 모양은 `app.css` `.msg.agent`).
  const agent = m.author === "agent";
  const article = el(
    "article",
    { class: `msg ${m.author} side-${side}`, "data-msg": String(m.id), tabindex: "0" },
    agent
      ? el("div", { class: "avatar agent-av", "aria-hidden": "true" }, ico("bot"))
      : side === "theirs"
        ? el("div", { class: `avatar ${peerTint(conv)}`.trim(), "aria-hidden": "true", text: initialOf(who.slice(who.lastIndexOf("/") + 1)) })
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

/// 메시지 작업 — 다시 알리기 · 요청 상태 바꾸기(`req:{상태}` — 받은 요청, 의견 칸을 열고 확인하면 쓴 쪽에 간다).
export function messageAct(m, key) {
  if (key === "req-again") {
    resendState(m.id, m.req_state, m.req_note);
  } else if (key.startsWith("req:")) {
    const bar = $("messages").querySelector(`[data-msg="${m.id}"] .msg-actions`);
    if (bar) noteForm(bar, m.id, key.slice(4), messageActions(m, !!convClosed(state.current)).find((a) => a.key === key)?.label ?? "");
  }
}

/// 말풍선 북마크 아이콘 — 넣기 · 빼기(이 PC 에만). 북마크한 글은 노란 아이콘이 늘 보인다.
export function bookmarkButton(m) {
  const on = m.saved === true;
  const b = iconButton(BOOKMARK_SVG, on ? "북마크 해제" : "북마크", () => toggleBookmark(m));
  b.classList.add("bookmark");
  if (on) b.classList.add("on");
  b.setAttribute("aria-pressed", String(on));
  b.dataset.act = "save";
  return b;
}

/// 북마크 넣기 · 빼기 — 마지막으로 읽어 온 대화 기준. 바꾼 뒤 그 아이콘 · 머리 수 · 실행 상세만(대화를 다시 그리면 HTML 본문이 다시 읽힌다).
export function toggleBookmark(m) {
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

export const COPY_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><rect x="9" y="9" width="12" height="12" rx="2"></rect><path d="M5 15H4a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1"></path></svg>';
export const BOOKMARK_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M19 21l-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path></svg>';
export const PENCIL_SVG =
  '<svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" stroke-width="1.8" aria-hidden="true"><path d="M12 20h9"></path><path d="M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4z"></path></svg>';

/// 복사 아이콘 — 본문 markdown 원문을 클립보드로, 아이콘 옆에 잠깐 "복사했습니다".
export async function copyText(m, btn) {
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

/// 말풍선 목록 — 날짜가 바뀌면 구분선. 컨텍스트 압축 · 비우기는 말풍선이 아니라 구분선(압축은 요약 펼치기).
export function messageEls(conv, ms) {
  const days = dayBreaks(ms);
  return ms.flatMap((m, i) =>
    [
      days[i] ? el("div", { class: "day-break", role: "separator", text: days[i] }) : "",
      m.author === "context" ? contextEl(m) : messageEl(conv, m),
      feedbackEl(conv, m),
    ].filter(Boolean),
  );
}

/// 요청 의견 말풍선 — 대기가 아닌 요청 바로 아래, 받은 쪽 사람 쪽에 지금 상태(크게) · 의견(줄바꿈 그대로) · 바뀐 시각. 상태가 바뀌면 다시 그린다(서명에 `req_*`).
export function feedbackEl(conv, m) {
  const f = reqFeedback(m);
  if (!f) return "";
  const who = f.side === "mine" ? "나" : shortName(state.peerNames.get(conv.peer_id));
  return el(
    "article",
    { class: `msg feedback side-${f.side}`, "data-feedback": String(m.id) },
    f.side === "theirs" ? el("div", { class: `avatar ${peerTint(conv)}`.trim(), "aria-hidden": "true", text: initialOf(who) }) : "",
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

export function contextEl(m) {
  return el(
    "div",
    { class: "ctx-break", role: "separator", "data-msg": String(m.id), "data-state": m.state },
    el("span", { class: "ctx-line" }, ico(m.ctx_op === "clear" ? "reset" : "compress"), el("span", { text: `${contextNote(m)} · ${formatTime(m.created_at)}` })),
    m.text ? el("details", { class: "ctx-summary" }, el("summary", { text: "요약 보기" }), el("div", { class: "ctx-text", text: m.text })) : "",
  );
}


/// 요청 상태 바꾸기 — `box` 끝에 의견 칸(3줄 · 글자 수) + 취소 · 확인을 연다(이미 열려 있으면 바꾼다). Enter = 확인 · Shift+Enter = 줄바꿈(입력창과 같다).
/// 확인하면 `PATCH` 뒤 새로 고침(대화 · 요청 목록 · 배지).
export function noteForm(box, id, to, label, onCancel) {
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

/// 알리지 못한 요청 상태를 같은 상태 · 의견으로 다시 보낸다.
export function resendState(id, state, note) {
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

/// 상대 쪽 말풍선 아바타 색 — 그 대화 상대 타일 색(`tileTint`).
export function peerTint(conv) {
  return tileTint(state.peers.find((p) => p.id === conv.peer_id)?.color);
}

export function messagesSig(conv, ms) {
  return `${messagesSigOf(state.peerNames.get(conv.peer_id), ms)}|${peerTint(conv)}`;
}

/// 그 글로 — 가운데로 스크롤해 잠깐 강조하고 실행 상세의 '진행' 을 그 요청으로(`#c/{대화}/m/{글}`).
export function focusMessage(id) {
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
export function setConvMessages(conv, ms) {
  state.msgs = ms;
  state.msgsOf = conv.id;
  showRelay(conv, ms);
  renderConvHead(conv);
}

export async function openConversation(conv) {
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

/// 제목 고치기(이 PC 에만) 켜고 끄기 — 켜면 머리글 제목 줄 대신 입력칸(처음 값 = 지금 제목, 비우면 첫 메시지 앞부분).
export function editTitle(on) {
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
export function showRelay(conv, ms) {
  const r = relayText(conv, ms);
  const btn = $("conv_relay_btn");
  $("conv_relay").hidden = r == null;
  $("conv_relay").textContent = r?.text ?? "";
  btn.hidden = !r?.action || !!convClosed(conv);
  btn.textContent = r?.action === "resume" ? "재개" : "멈춤";
  btn.dataset.on = String(r?.action === "resume");
}

/// 자동 왕복 멈춤 · 재개 — 머리글 버튼.
export function switchRelay(on) {
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

/// 세부정보 "관람자" — 내 대화 · agent 대화: 관람자 목록(빼기) · 추가(페어링된 상대, 5명까지). 공유 대화(사본): 누가 공유했는지 · 나가기.
/// 원 상대 쪽(관람자 수만 안다)은 머리 "관람자 n" 만.
export function renderShare(conv) {
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
export function josaName(name) {
  return `${name}${josa(name, "이", "가")}`;
}

export function addObserver() {
  const conv = state.current;
  const id = Number($("share_peer").value);
  if (!conv || !id) return;
  act("관람자 추가", async () => {
    await api("POST", `/api/conversations/${conv.id}/observers`, { peer_id: id });
    return "관람자를 넣었습니다";
  }, "status_msg", loadConvs);
}

export function removeObserver(conv, id, name) {
  act("관람자 빼기", async () => {
    await api("DELETE", `/api/conversations/${conv.id}/observers/${id}`);
    return `${name} 을 관람자에서 뺐습니다`;
  }, "status_msg", loadConvs);
}

$("share_add_btn").addEventListener("click", addObserver);
// 공유 대화 나가기 — ⋯ "나가기" 와 같다(확인 뒤 공유를 끝내고 사본을 이 PC 목록에서 삭제).
$("share_leave_btn").addEventListener("click", (ev) => {
  const conv = state.current;
  if (!conv) return;
  openFloat(ev.currentTarget, "dialog", "대화 나가기", [convEndForm(conv, "leave")]);
});

$("conv_relay_btn").addEventListener("click", () => switchRelay($("conv_relay_btn").dataset.on === "true"));
