// 공용 조각 — 타일 얼굴 · 유형 칩 · 에이전트 타일 · 발화자 아이콘 · 아이콘 단추.
import { el, ico } from "./dom.js";
import { agentColor } from "../logic/agent.js";
import { KIND_NAMES, convKindIcon } from "../logic/conv.js";
import { speakerIcon } from "../logic/message.js";
import { colorOf } from "../logic/peer.js";

/// 대화 유형 칩 — 유형 색 면 + 아이콘(내 에이전트 응답 = 로봇 · 내가 문의 = 사람), 색은 둘레의 `data-kind` 가 정한다. 공유 사본은 없음.
/// 읽는 이름 = 유형 이름(같은 제목의 응답 · 문의 줄을 스크린리더도 가른다, 링크 이름에 들어간다).
export function kindChip(c, cls) {
  const n = convKindIcon(c);
  return n ? el("span", { class: `${cls} kind-chip`, role: "img", "aria-label": KIND_NAMES[c.dir] }, ico(n)) : "";
}

/// 내 에이전트 로봇 타일 — 고른 아이콘 색.
export function agentTile(color, cls = "") {
  return el("span", { class: `pa agent-pa ${agentColor(color)} ${cls}`.trim(), "aria-hidden": "true" }, ico("bot"));
}

export const PERSON_SVG =
  '<svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true"><circle cx="12" cy="8" r="4" fill="currentColor"></circle><path d="M4 21c0-4.4 3.6-7 8-7s8 2.6 8 7z" fill="currentColor"></path></svg>';
export const GROUP_SVG =
  '<svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true"><circle cx="15.5" cy="7.5" r="3.4" fill="currentColor" opacity=".45"></circle><path d="M10 20c.3-3.8 2.8-6 5.5-6s5.3 2.2 5.5 6z" fill="currentColor" opacity=".45"></path><circle cx="9.5" cy="9" r="3.8" fill="currentColor"></circle><path d="M2.5 21.5c0-4.1 3.1-6.6 7-6.6s7 2.5 7 6.6z" fill="currentColor"></path></svg>';

/// 타일 얼굴 — 그림이 있으면 그림, 없으면 사람(나 · 상대) · 사람 + 그림자(그룹) 모양. 누구인지는 아래 이름(`tileName`). 그림은 고정 문자열만.
export function tileFace(icon, group = false) {
  if (icon) return el("img", { src: icon, alt: "", draggable: "false" }); // 그림 기본 끌기가 레일 포인터 끌기를 가로채지 않게
  const s = el("span", { class: "tile-shape", "aria-hidden": "true" });
  s.innerHTML = group ? GROUP_SVG : PERSON_SVG;
  return s;
}

/// 타일 아래 이름 — 11px · 넘치면 말줄임, 전체 이름은 단추 `title`.
export function tileName(name) {
  return el("span", { class: "tile-name", "aria-hidden": "true", text: name });
}

/// 타일 색 — 상대 id 로 다섯 색을 돈다(자리를 옮겨도 색이 타일을 따라간다).
export function tileColor(p) {
  return colorOf(p.id);
}

/// 줄 타일 — 상대 사람 모양(그림이 있으면 그림), 그룹은 사람 + 그림자. 내 에이전트는 `agentTile`.
export function rowTile(cls, icon, group = false) {
  return el("span", { class: `pa ${cls}`, "aria-hidden": "true" }, tileFace(icon, group));
}

/// 발화자 종류 아이콘(사람 ↔ 에이전트) — 모양 · 이름은 `logic/message.js` `speakerIcon`.
export function speakerEl(author) {
  const s = speakerIcon(author);
  return el("span", { class: `speaker ${s.kind}`, title: s.label, "aria-label": s.label, role: "img" }, ico(s.icon));
}

/// 말풍선 머리줄 아이콘 단추 — 그림은 고정 문자열(`conv.js` `*_SVG` 상수)만.
export function iconButton(svg, label, onClick) {
  const b = el("button", { type: "button", class: "icon-btn msg-icon", title: label, "aria-label": label, onclick: onClick });
  b.innerHTML = svg;
  return b;
}

/// 글자 단추 — 테두리 없이 아이콘 + 이름, `look` = `REQ_BTN` 값(없으면 강조색 글자만).
export function textButton(label, look, onClick) {
  return el("button", { type: "button", class: look ? `text-btn ${look.tone}` : "text-btn", onclick: onClick }, look ? ico(look.icon) : "", el("span", { text: label }));
}
