// 공용 조각 — 타일 얼굴 · 유형 칩 · 에이전트 타일 · 발화자 아이콘 · 아이콘 단추.
import { el, ico } from "./dom.js";
import { agentColor } from "../logic/agent.js";
import { KIND_NAMES, convKindIcon } from "../logic/conv.js";
import { speakerIcon } from "../logic/message.js";
import { meTint, tileTint } from "../logic/peer.js";

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

export const PERSON_SVG =
  '<svg viewBox="0 0 24 24" width="22" height="22" aria-hidden="true"><circle cx="12" cy="8" r="4" fill="currentColor"></circle><path d="M4 21c0-4.4 3.6-7 8-7s8 2.6 8 7z" fill="currentColor"></path></svg>';
/// 그룹 = 앞 사람 가운데 + 뒤 좌우 옅은 사람 둘.
export const GROUP_SVG =
  '<svg viewBox="0 0 24 24" width="24" height="24" aria-hidden="true"><circle cx="5.6" cy="8.4" r="2.8" fill="currentColor" opacity=".45"></circle><path d="M.6 19.5c0-3.3 2.2-5.4 5-5.4s5 2.1 5 5.4z" fill="currentColor" opacity=".45"></path><circle cx="18.4" cy="8.4" r="2.8" fill="currentColor" opacity=".45"></circle><path d="M13.4 19.5c0-3.3 2.2-5.4 5-5.4s5 2.1 5 5.4z" fill="currentColor" opacity=".45"></path><circle cx="12" cy="9" r="3.6" fill="currentColor"></circle><path d="M5.6 21.5c0-4 2.8-6.4 6.4-6.4s6.4 2.4 6.4 6.4z" fill="currentColor"></path></svg>';

/// 사람 · 그룹 얼굴 — 레일 · 줄 타일 · 말풍선 · 주고받은 메시지 · 프로필이 모두 이 모양. 그림이 있으면 그림, 없으면 사람(나 · 상대) · 세 사람(그룹) 모양 +
/// 고른 타일 색(`tileTint` — 없으면 무채, `me` 면 흑연 — `meTint`). 누구인지는 곁의 이름. 그림은 고정 문자열만.
export function railFace(icon, group = false, color, me = false) {
  if (icon) return el("img", { src: icon, alt: "", draggable: "false" }); // 그림 기본 끌기가 레일 포인터 끌기를 가로채지 않게
  const s = el("span", { class: `tile-shape ${me ? meTint(color) : tileTint(color)}`.trim(), "aria-hidden": "true" });
  s.innerHTML = group ? GROUP_SVG : PERSON_SVG;
  return s;
}

/// 타일 아래 이름 — 11px · 넘치면 말줄임, 전체 이름은 단추 `title`.
export function tileName(name) {
  return el("span", { class: "tile-name", "aria-hidden": "true", text: name });
}

/// 줄 타일 — 레일과 같은 얼굴(`railFace` — `group` 이면 세 사람). 내 에이전트는 `agentTile`.
export function rowTile(cls, icon, color, group = false) {
  return el("span", { class: `pa ${cls}`.trim(), "aria-hidden": "true" }, railFace(icon, group, color));
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
