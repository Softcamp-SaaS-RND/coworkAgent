// 공용 조각 — 타일 얼굴 · 유형 칩 · 에이전트 타일 · 발화자 아이콘 · 아이콘 단추.
import { el, ico } from "./dom.js";
import { agentColor } from "../logic/agent.js";
import { KIND_NAMES, convKindIcon } from "../logic/conv.js";
import { speakerIcon } from "../logic/message.js";
import { initialOf } from "../logic/peer.js";

/// 대화 유형 표시 — 무채 선 아이콘(내 에이전트 응답 = 로봇 · 내가 문의 = 사람 · 공유 대화 = 눈, `convKindIcon`).
/// 읽는 이름 = 유형 이름(같은 제목의 응답 · 문의 줄을 스크린리더도 가른다, 링크 이름에 들어간다).
export function kindChip(c, cls) {
  const n = convKindIcon(c);
  return n ? el("span", { class: `${cls} kind-chip`, role: "img", "aria-label": KIND_NAMES[c.dir] }, ico(n)) : "";
}

/// 내 에이전트 로봇 타일 — 고른 아이콘 색.
export function agentTile(color, cls = "") {
  return el("span", { class: `pa agent-pa ${agentColor(color)} ${cls}`.trim(), "aria-hidden": "true" }, ico("bot"));
}

/// 타일 얼굴 — 그림이 있으면 그림, 없으면 이름 머리글자 네모(나 · 상대 · 그룹 같은 모양). 누구인지는 아래 이름(`tileName`).
export function tileFace(icon, name) {
  if (icon) return el("img", { src: icon, alt: "", draggable: "false" }); // 그림 기본 끌기가 레일 포인터 끌기를 가로채지 않게
  return el("span", { class: "tile-shape", "aria-hidden": "true", text: initialOf(name) });
}

/// 타일 아래 이름 — 11px · 넘치면 말줄임, 전체 이름은 단추 `title`.
export function tileName(name) {
  return el("span", { class: "tile-name", "aria-hidden": "true", text: name });
}

/// 줄 타일 — 이름 머리글자 네모(그림이 있으면 그림). 내 에이전트는 `agentTile`.
export function rowTile(cls, icon, name) {
  return el("span", { class: `pa ${cls}`.trim(), "aria-hidden": "true" }, tileFace(icon, name));
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
