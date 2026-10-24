// 이 PC 앱 API 호출 · 연결 끊김 판정.
import { state } from "./state.js";
import { disconnected, token } from "./token.js";
import { renderAvatar } from "../shell.js";

/// JSON 을 받거나, 실패 사유를 담은 Error 를 던진다.
export async function api(method, path, body) {
  const init = { method, headers: { Authorization: `Bearer ${token}` } };
  if (body !== undefined) {
    init.headers["Content-Type"] = "application/json";
    init.body = JSON.stringify(body);
  }
  let res;
  try {
    res = await fetch(path, init);
  } catch (e) {
    netResult(false);
    throw e;
  }
  netResult(true);
  if (res.status === 401) disconnected();
  const text = await res.text();
  let data = null;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    // JSON 이 아닌 오류 본문(요청 형식 거부 등)은 그대로 사유로.
    if (res.ok) throw new Error("응답이 JSON 이 아닙니다");
    throw new Error(`HTTP ${res.status} ${text.slice(0, 200)}`);
  }
  if (!res.ok) throw new Error(data?.error ?? "HTTP " + res.status);
  return data;
}

/// 앱(이 PC 서버)과의 연결 — 요청이 닿지 못한 것(fetch 실패)이 연달아 `OFFLINE_AFTER` 번이면 끊김(프로필 붉음), 한 번이라도 닿으면 되돌린다.
/// HTTP 오류 · 401 은 닿은 것이다(401 은 `api` 가 다시 열기 안내). 5초 새로 고침이 계속 묻는다.
export const OFFLINE_AFTER = 2;

export function netResult(ok) {
  const was = appOffline();
  state.netFails = ok ? 0 : state.netFails + 1;
  if (was !== appOffline()) renderAvatar();
}

export function appOffline() {
  return state.netFails >= OFFLINE_AFTER;
}
