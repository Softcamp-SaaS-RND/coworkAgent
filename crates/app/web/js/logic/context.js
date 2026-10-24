// 컨텍스트 — 크기 · 이름 · 메뉴 · 사유.

/// 컨텍스트 상한(토큰, 1M 고정) · 경고 문턱(60% 주황 · 80% 붉음 — 80% 는 자동 압축 문턱과 같다).
export const CONTEXT_LIMIT = 1_000_000;

/// 대화 컨텍스트 크기 — 마지막으로 끝난 압축 · 비우기 구분선 뒤 답들에서 세션(에이전트 · 쪽)별 마지막 창 사용량의 최대. 그 뒤 답이 없으면 0,
/// 구분선도 크기 있는 답도 없으면 `null`(모름 — 이전 버전 상대).
export function contextSize(ms) {
  const last = new Map();
  let reset = false;
  for (const m of ms) {
    if (m.author === "agent" && m.ctx_tokens != null) last.set(`${m.mine ? 1 : 0}:${m.agent ?? ""}`, m.ctx_tokens);
    if (m.author !== "context" || m.state !== "done") continue;
    reset = true;
    // 자동 왕복 부분 처리(상대 PC 쪽 실패 — `peer_*`) — 이 PC 쪽 세션만 비웠다, 상대 세션 값은 남긴다.
    if (m.refused?.startsWith("peer_")) {
      for (const k of [...last.keys()]) if (k.startsWith("1:")) last.delete(k);
    } else {
      last.clear();
    }
  }
  if (last.size) return Math.max(...last.values());
  return reset ? 0 : null;
}

/// 머리 표시 — `{text, level}`: "42k / 1M" · 단계 `ok` · `warn`(60%) · `high`(80%) · `unknown`.
export function contextLabel(n) {
  if (n == null) return { text: "컨텍스트 알 수 없음", level: "unknown" };
  const v = n < 1000 ? String(n) : n < CONTEXT_LIMIT ? `${Math.round(n / 1000)}k` : `${(n / CONTEXT_LIMIT).toFixed(1)}M`;
  const level = n >= CONTEXT_LIMIT * 0.8 ? "high" : n >= CONTEXT_LIMIT * 0.6 ? "warn" : "ok";
  return { text: `${v} / 1M`, level };
}

/// 실행 · 압축이 걸린 대화인가(압축 · 비우기 메뉴를 끈다): 대기 · 실행 중 글, 상대 답을 기다리는 실행 요청, 진행 중 구분선.
export function contextBusy(ms) {
  return ms.some((m) => m.state === "queued" || m.state === "running" || (m.state === "sent" && (m.exec || m.author === "context")));
}

/// 압축 · 비우기 메뉴 — 실행 중이거나 답(세션)이 아직 없으면 꺼진다.
export function ctxMenu(busy, answered) {
  const off = busy || !answered;
  return [
    { key: "compact", label: "압축", icon: "compress", disabled: off },
    { key: "clear", label: "비우기", icon: "reset", disabled: off },
  ];
}

export function contextWhy(code) {
  if (code === "unsupported") return "상대가 0.5.0 이상이어야 합니다";
  if (code === "unreachable") return "상대에 닿지 않음";
  if (code === "compact_failed") return "요약 실행 실패 · 세션은 그대로";
  return `실패(${code ?? "알 수 없음"})`;
}

/// 컨텍스트 구분선 문구 — 종류 · 상태 · 실패 사유.
export function contextNote(m) {
  const compact = m.ctx_op !== "clear";
  const name = m.ctx_op === "auto_compact" ? "자동 압축(80%)" : compact ? "컨텍스트 압축" : "컨텍스트 비움";
  if (m.state === "queued" || m.state === "running") return `${name} · ${compact ? "요약 중…" : "비우는 중…"}`;
  if (m.state === "sent") return `${name} · 상대 PC 처리 대기`;
  if (m.state === "error") return `${name} · ${contextWhy(m.refused)}`;
  const done = compact ? `${name} · 이후 새 세션이 요약을 이어 받습니다` : `${name} · 이후 새 세션(앞 대화 기억 없음)`;
  // 자동 왕복 — 이 PC 는 처리했고 상대 PC 쪽이 실패(부분 처리를 숨기지 않는다).
  if (m.refused?.startsWith("peer_")) return `${done} · 상대 PC 쪽은 실패(${contextWhy(m.refused.slice(5))}) · 다시 누르면 다시 요청`;
  return done;
}
