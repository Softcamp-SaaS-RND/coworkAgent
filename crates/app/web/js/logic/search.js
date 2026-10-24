// 통합 검색 — 설정 항목 · 결과 묶음.
import { authorLabel } from "./author.js";
import { convClosed, convLabel } from "./conv.js";
import { displayName, preview, shortName } from "./format.js";
import { peerState } from "./peer.js";

/// 검색 — `fields` 값에 부분 일치(대소문자 · 앞뒤 공백 무시), 빈 검색이면 전부(순서 그대로).
export function searchItems(items, q, fields) {
  const needle = (q ?? "").trim().toLowerCase();
  return needle ? items.filter((it) => fields.some((f) => String(it[f] ?? "").toLowerCase().includes(needle))) : items;
}

/// 통합 검색 결과(`GET /api/search`) → 묶음(메시지 · 대화 · 사람 · 에이전트 — 빈 묶음 없음). 행 = `{label, sub}` + 가는 곳: `href` · `tile`(그 타일
/// 선택 — 페어링된 사람) · `chat`(세션 생성에 상대 · 에이전트 채움 — 상대 에이전트). `mark` = 줄 앞 칩(메시지 · 대화 `{type, kind: dir}`,
/// 사람 `{type: "peer", peer}`, 에이전트 `{type: "agent", owner, name}`), `tag` = 닫힌 대화 문구(`convClosed` — 공유 끝남은 검색 응답에 없어 `convs` 의 같은 대화로, 없으면 "").
export function searchGroups(r, peerNames, convs = []) {
  const closed = (c) => convClosed(convs.find((x) => x.id === c.id) ?? c);
  const groups = [
    {
      title: "메시지",
      rows: (r.messages ?? [])
        .filter((m) => m.conversation)
        .map((m) => {
          const conv = m.conversation;
          const who = authorLabel(conv, m, peerNames.get(conv.peer_id));
          return { label: preview(m.text), sub: `${who} · ${convLabel(conv, peerNames).title}`, href: `#c/${conv.id}/m/${m.id}`, mark: { type: "msg", kind: conv.dir }, tag: closed(conv) };
        }),
    },
    {
      title: "대화",
      rows: (r.conversations ?? []).map((c) => {
        const l = convLabel(c, peerNames);
        return { label: l.title, sub: l.sub, href: `#c/${c.id}`, mark: { type: "conv", kind: c.dir }, tag: closed(c) };
      }),
    },
    {
      title: "사람",
      rows: (r.peers ?? []).map((p) => ({
        ...(p.paired ? { label: displayName(p), sub: p.host, tile: p.id } : { label: displayName(p), sub: `${p.host} · ${peerState(p)}`, href: "#settings/peers" }),
        mark: { type: "peer", peer: p.id },
        tag: "",
      })),
    },
    {
      title: "에이전트",
      rows: (r.agents ?? []).map((a) => ({
        ...(a.owner === "me"
          ? { label: a.name, sub: "내 에이전트", href: "#settings/agents" }
          : { label: a.name, sub: `${shortName(peerNames.get(a.owner))} 의 에이전트`, chat: { peer: a.owner, agent: a.name } }),
        mark: { type: "agent", owner: a.owner, name: a.name },
        tag: "",
      })),
    },
  ];
  return groups.filter((g) => g.rows.length);
}
