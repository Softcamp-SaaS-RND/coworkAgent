// 내 활동 — 거르기 탭 · 줄.
import { authorLabel } from "./author.js";
import { convLabel } from "./conv.js";
import { preview, shortName } from "./format.js";
import { REQ_STATES } from "./request.js";

/// 내 활동 거르기 탭 — `kind` = `GET /api/activity?kind=`, `icon` = `.ico-{이름}`.
export const ACT_KINDS = [
  { kind: "all", label: "전체", icon: "bell" },
  { kind: "request", label: "받은 요청", icon: "arrow-down" },
  { kind: "mine", label: "내 요청", icon: "arrow-up" },
  { kind: "answer", label: "받은 답", icon: "chat" },
  { kind: "failed", label: "실패", icon: "x" },
];

/// 내 활동 항목(`GET /api/activity`) → 목록 행. 라벨 = "받은 요청 · {상태}" · "받은 답"(실패한 답이면 "· 실패") · "내 요청 · {상태}"(+ 메모) ·
/// "실패"(내 쪽) · "상대 반영 실패 · {사유}". 글은 누르면 그 글로(`#c/{대화}/m/{글}`), 상대 반영 실패는 그 대화로. 대화를 모르는 항목 · 모르는 종류는 뺀다.
/// `key` = 다시 그려도 같은 항목을 가리키는 열쇠. `tone` = 칩 class(거르기 탭 `ACT_KINDS.kind` 와 같은 이름, 색은 `failed` 만): `request` · `mine` · `answer` ·
/// `failed`(내 쪽 실패 · 실패한 답 · 상대 반영 실패).
export function activityRows(items, peerNames) {
  return items
    .filter((a) => a.conversation && (a.message || a.kind === "sync_failed"))
    .map((a) => {
      const conv = a.conversation;
      const peer = peerNames.get(conv.peer_id);
      const m = a.message;
      const row = {
        key: `${a.kind}:${conv.id}:${m?.id ?? `${a.at}:${a.detail ?? ""}`}`,
        peer: conv.peer_id,
        at: a.at,
        unseen: a.unseen === true,
        title: convLabel(conv, peerNames).title,
        note: "",
      };
      if (!m) return { ...row, label: `상대 반영 실패 · ${a.detail ?? ""}`, tone: "failed", who: shortName(peer), preview: "", href: `#c/${conv.id}` };
      const state = REQ_STATES[m.req_state] ?? REQ_STATES.wait;
      const label =
        a.kind === "failed"
          ? "실패"
          : a.kind === "my_request"
            ? `내 요청 · ${state}`
            : m.author === "human"
              ? `받은 요청 · ${state}`
              : m.state === "error"
                ? "받은 답 · 실패"
                : "받은 답";
      const tone =
        a.kind === "failed" || (a.kind !== "my_request" && m.author !== "human" && m.state === "error")
          ? "failed"
          : a.kind === "my_request"
            ? "mine"
            : m.author === "human"
              ? "request"
              : "answer";
      const note = a.kind === "my_request" ? (m.req_note ?? "") : "";
      return { ...row, label, tone, note, author: m.author, who: authorLabel(conv, m, peer), preview: preview(m.text), href: `#c/${conv.id}/m/${m.id}` };
    });
}
