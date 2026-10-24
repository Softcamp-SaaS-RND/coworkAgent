// 떠 있는 상자(메뉴 · 이름 · 대화 추가) — 열기 · 닫기 · 초점 돌려주기.
import { $ } from "./dom.js";

export let floatFrom = null; // 떠 있는 상자를 연 요소 — 닫으면 초점을 돌려준다

/// 떠 있는 상자 — `anchor` 옆(레일이면 오른쪽, 그 밖은 아래)에 연다. 레일은 넘치면 잘리므로 화면 기준(`position: fixed`)으로 둔다.
export function openFloat(anchor, role, label, children) {
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
export function closeFloat(refocus) {
  $("float").hidden = true;
  $("float").replaceChildren();
  if (refocus) (floatFrom?.isConnected ? floatFrom : $("rail_me")).focus();
  floatFrom = null;
}

/// 작업 뒤 초점 — 다시 그린 새 요소(`sel`)가 있으면 거기, 없으면 `fallback`(키보드 사용자가 위치를 잃지 않게).
export function focusAfterFloat(sel, fallback = "rail_me") {
  (document.querySelector(sel) ?? $(fallback))?.focus();
}
