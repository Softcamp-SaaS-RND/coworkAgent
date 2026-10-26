// 레일 · 그룹 · 순서 — 타일 · 끌기 · 그룹 소속.
import { convLabel, keepsConversation } from "./conv.js";

/// 타일을 눌러도 열린 대화를 그대로 둘까 — 대화 화면(옛 `#s/` 포함)이고 그 대화가 그 타일 것일 때만.
export function tileKeeps(view, conv, railPeer) {
  return ["conversation", "sent"].includes(view) && keepsConversation(conv, railPeer);
}

/// 그룹설정 왼쪽 목록 — 검색(`q`)에 맞는 그룹 · 사용자와 각 대화 수. 사용자 = 이름 · 별칭 · PC · 지문 또는 그 사용자 대화 제목이 맞으면,
/// 그룹 = 이름이 맞거나 그 그룹 대화(제목 · 사용자)가 맞으면. 빈 검색이면 전부(순서 그대로). 그룹 `convs` = 오른쪽 상세에 보일 대화 id —
/// 이름이 맞으면(또는 빈 검색) 그 그룹 대화 전부, 아니면 맞는 대화만(이름으로 찾은 그룹의 상세가 비지 않게).
export function groupSettingsList(groups, peers, convs, q, peerNames) {
  const needle = (q ?? "").trim().toLowerCase();
  const hit = (s) => String(s ?? "").toLowerCase().includes(needle);
  const userHit = (p) => !!p && ["alias", "name", "host", "fingerprint"].some((f) => hit(p[f]));
  const convHit = (c) => hit(convLabel(c, peerNames).title) || userHit(peers.find((p) => p.id === c.peer_id));
  const mine = (p) => convs.filter((c) => c.peer_id === p.id);
  return {
    groups: groups
      .filter((g) => !needle || hit(g.name) || inGroup(convs, g).some(convHit))
      .map((g) => {
        const all = inGroup(convs, g);
        const shown = !needle || hit(g.name) ? all : all.filter(convHit);
        return { id: g.id, name: g.name, icon: g.icon, color: g.color, count: all.length, convs: shown.map((c) => c.id) };
      }),
    users: peers.filter((p) => !needle || userHit(p) || mine(p).some(convHit)).map((p) => ({ id: p.id, count: mine(p).length })),
  };
}

/// 끌기 시작 거리(레일 타일) — 누른 채 이만큼(px) 움직여야 끌기, 그 미만은 그냥 누름(타일 고르기).
export const DRAG_SLOP = 5;

export function pastSlop(dx, dy) {
  return dx * dx + dy * dy >= DRAG_SLOP * DRAG_SLOP;
}

/// 레일 타일을 놓을 자리 — 같은 묶음 타일 가운데(`centers`, 보이는 순서 · 세로 레일 = Y, 가로 레일 = X)에서 포인터(`pos`)와 가장 가까운 타일과
/// 그 앞 / 뒤(가운데를 넘으면 뒤). 틈 · 타일 아래 이름 · 양 끝 밖에 놓아도 자리가 정해진다 — 놓는 점 아래 요소만 보면 거기서는 아무 일도 일어나지 않는다. 타일이 없으면 `null`.
export function railSlot(centers, pos) {
  if (!centers.length) return null;
  let i = 0;
  for (let k = 1; k < centers.length; k++) if (Math.abs(centers[k] - pos) < Math.abs(centers[i] - pos)) i = k;
  return { index: i, after: pos > centers[i] };
}

/// 대화 줄의 끌기 묶음 열쇠 — 끌어 옮기기 · Alt + ↑ / ↓ 는 같은 열쇠 안에서만(`dropKind`).
export function splitKey(section, dir) {
  return `${section}:${dir}`;
}

/// 그룹 타일을 고른 채 열 수 없는 화면 — 내 활동 · 주고받은 글('나' 에만) · 에이전트(대화 단위 아님) → '나' 로 바꿔 연다.
export function groupLeaves(view) {
  return view === "activity" || view === "agents" || view === "agentConvs" || view === "peer";
}

/// 상대 타일을 고른 채 열 수 없는 화면 — 에이전트 응답('나' 에만).
export function peerLeaves(view) {
  return view === "agentConvs";
}

/// 그룹의 안 읽음 합(레일 배지).
export function groupUnread(convs, group) {
  return inGroup(convs, group).reduce((n, c) => n + (c.unread ?? 0), 0);
}

/// 키보드 한 칸(Alt + ↑ / ↓) — 보이는 같은 묶음(`visible` — 화면 순서 id)에서 `dir`(-1 위 · +1 아래) 쪽 이웃의 앞 · 뒤. 끝이면 `null`.
export function stepTarget(visible, id, dir) {
  const i = visible.indexOf(id);
  const j = i + dir;
  if (i < 0 || j < 0 || j >= visible.length) return null;
  return { target: visible[j], after: dir > 0 };
}

/// 끌어 놓기 판정 — 끄는 것(`drag` = `{kind, section}`) → 놓을 곳(`target` = `{kind, section}`): `"move"`(같은 종류 · 대화는 같은 절) ·
/// `"join"`(대화 → 그룹 타일, 그룹에 넣기) · `null`(놓을 수 없음 — '나' · "+" · 설정 · 다른 묶음 · 다른 절).
export function dropKind(drag, target) {
  if (!drag || !target) return null;
  if (drag.kind === "conv" && target.kind === "group") return "join";
  if (drag.kind !== target.kind || !["peer", "group", "conv"].includes(drag.kind)) return null;
  if (drag.kind === "conv" && drag.section !== target.section) return null;
  return "move";
}

/// 그룹의 대화 — 넣은 대화(`convs` — 그룹 = 대화만). 순서는 들어온 그대로.
export function inGroup(convs, group) {
  const ids = new Set(group?.convs ?? []);
  return convs.filter((c) => ids.has(c.id));
}

/// 그룹 요약에 머리를 둘 페어링된 상대 — 레일 순서 중 그 그룹에 대화가 있는 상대만('나' 처럼 대화 없는 상대를 늘어놓지 않는다).
export function groupPeers(convs, paired) {
  const has = new Set(convs.map((c) => c.peer_id));
  return paired.filter((id) => has.has(id));
}

/// 목록 범위 쿼리(`peer=` · `group=`, 없으면 빈 문자열) — 상대 · 그룹 중 하나만.
export function scopeQuery(peer, group) {
  if (group != null) return `group=${group}`;
  if (peer != null) return `peer=${peer}`;
  return "";
}

/// 그룹 이름 미리 거르기(서버가 정본) — 앞뒤 공백 뺀 1 ~ 40자 · 같은 이름(`except` = 이름 바꾸는 그 그룹 id) 없음. 맞으면 `null`.
export function groupNameError(name, groups, except = null) {
  const n = (name ?? "").trim();
  if (!n || [...n].length > 40) return "그룹 이름은 1 ~ 40자";
  if (groups.some((g) => g.name === n && g.id !== except)) return "같은 이름의 그룹이 있습니다";
  return null;
}

/// 그룹 소속 바꾸기 — 고른 대화(`picked`) 와 지금 소속(`now`)의 차이 → `{add, remove}`(작은 id 부터).
export function membershipDiff(now, picked) {
  const a = new Set(now);
  const b = new Set(picked);
  return {
    add: [...b].filter((id) => !a.has(id)).sort((x, y) => x - y),
    remove: [...a].filter((id) => !b.has(id)).sort((x, y) => x - y),
  };
}

/// 레일 폭(손잡이 끌기) — 끄는 동안 `min` ~ `max`, `split` 이상이면 작은 아이콘, 놓으면 `normal`(크게) · `small`(작게) 폭으로 맞춘다.
export const RAIL_W = { normal: 72, small: 168, split: 120, min: 64, max: 240 };

export function railSize(w) {
  return w >= RAIL_W.split ? "small" : "normal";
}
