// DOM 도우미 — 요소 · 아이콘 · 단추 · 칸 · 다시 그릴지 판정.
import { holdsRender } from "../logic/shell.js";

export const $ = (id) => document.getElementById(id);

/// 아이콘 — `app.css` `.ico-{이름}` 모양 · 크기 16px 한 벌. 읽기 도구에는 숨긴다(옆 글자가 이름).
export function ico(name) {
  return el("span", { class: `ico ico-${name}`, "aria-hidden": "true" });
}

/// 글자 앞 아이콘 — 라벨 · 제목 · 단추 글자와 한 칸 띄운다(`.ico.lead`).
export function lead(name) {
  return el("span", { class: `ico ico-${name} lead`, "aria-hidden": "true" });
}

/// 요소 만들기 — `text` 는 textContent, `class` · 속성, `on*` 는 이벤트.
export function el(tag, props = {}, ...children) {
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

/// `icon` 이 있으면 글자 앞에 아이콘.
export function button(label, onClick, cls, icon) {
  return icon
    ? el("button", { type: "button", onclick: onClick, class: cls }, lead(icon), label)
    : el("button", { type: "button", text: label, onclick: onClick, class: cls });
}

/// `label` 작업을 하고 결과를 `out`(상태 줄)에 적는다.
export async function act(label, fn, out = "status_msg", after) {
  $(out).textContent = label + " …";
  try {
    const done = await fn();
    $(out).textContent = typeof done === "string" ? done : label + " 완료";
  } catch (e) {
    $(out).textContent = label + " 실패: " + e.message;
  }
  if (after) await after();
}

export let uid = 0;
/// 라벨 + 컨트롤 한 쌍(id 자동) — `box` 는 컨트롤을 감싼 칸.
export function field(label, control, box = control, icon) {
  control.id ||= "f" + ++uid;
  return [icon ? el("label", { for: control.id }, lead(icon), label) : el("label", { for: control.id, text: label }), box];
}

/// 같은 내용이면 다시 그리지 않는다 — 5초 새로 고침 · 이벤트가 레일 · 내비 · 대화 목록의 초점과 누르는 중인 클릭을 빼앗지 않게.
/// 서명에 그리는 데 쓰는 값을 모두 넣는다(다르면 서명을 새로 적고 그린다).
export function unchanged(box, sig) {
  if (box.dataset.sig === sig) return true;
  box.dataset.sig = sig;
  return false;
}

/// 목록을 다시 그려도 되나 — 같은 값이면 건너뛰고, 사용자가 그 안 입력칸에서 입력 중이면(배경 새로 고침) 미룬다. 저장 · 필터 · 고르기는 `force`.
/// 입력칸만 본다 — 상대별 절 머리(`summary`)를 눌러 초점이 남아도 목록은 계속 새로 고친다.
export function needsRender(box, sig, force) {
  const typing = box.contains(document.activeElement) && holdsRender(document.activeElement.tagName);
  if (!force && (box.dataset.sig === sig || typing)) return false;
  box.dataset.sig = sig;
  return true;
}
