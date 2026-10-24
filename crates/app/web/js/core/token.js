// 진입 토큰 — 트레이 "열기" 가 준 `/?t=` 를 탭 메모리로 가져가고, 없으면 다시 열기 안내 후 멈춘다.
import { $, el } from "./dom.js";
import { DISCONNECTED, entryToken } from "../logic/shell.js";

// 탭 메모리 토큰 — 트레이 "열기" 가 준 `/?t=` 를 가져가고 주소창에서 지운다(쿠키 없음: 쿠키는 포트를 가리지 않는다).
export const TOKEN_KEY = "an_token";
export const entry = entryToken(location.search, sessionStorage.getItem(TOKEN_KEY));
if (entry.clean) history.replaceState(null, "", "/" + location.hash);
if (entry.token) sessionStorage.setItem(TOKEN_KEY, entry.token);
export const token = entry.token;

/// 토큰이 없거나 거절됨 — 화면 위에 다시 열기 안내(한 번만).
export function disconnected() {
  if ($("disconnected")) return;
  document.body.prepend(el("div", { id: "disconnected", class: "disconnected", role: "alert", text: DISCONNECTED }));
}
if (!token) {
  disconnected();
  throw new Error("토큰 없음");
}
