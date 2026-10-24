// 개요 미리보기 — 크기 · 절 · 다시 읽기 · 말풍선.
import { authorLabel, bubbleSide } from "./author.js";
import { preview, stateText } from "./format.js";

/// 개요 미리보기 카드마다 받는 마지막 글 수 — 카드 높이에 최신 쪽부터 보이고 위는 잘린다(200% 본문도 채우게 12).
export const PREVIEW_MSGS = 12;

/// 개요 미리보기 배율(%, 설정 `ui_home_zoom`, 서버 `HOME_ZOOMS`) — 첫 값이 기본. 축소 없음(처음 화면이 이미 작다).
export const HOME_ZOOMS = [100, 125, 150, 175, 200];

/// 배율 → 카드 폭 · 본문 높이 배수(글자 크기 · 말풍선 2줄은 그대로). 목록 밖 값은 100%.
export function previewZoom(z) {
  const pct = HOME_ZOOMS.includes(Number(z)) ? Number(z) : HOME_ZOOMS[0];
  return { scale: pct / 100 };
}

/// 개요 미리보기 절 — 상대마다 절, 카드 = 그 상대 대화(최근 활동 순 — `last_at` 큰 것부터, 없으면 뒤). '나' · 그룹은 한 줄(`cols` = 폭에 맞춘 열 수)만 —
/// 넘치면 `cols - 1` 개 + 숨은 수 `more`(끝 칸 "+n개"), 상대 타일(`single`)은 전부. `groups` = `peerSummary` 결과(대화 없는 상대는 `cards` 빈 배열).
export function previewSections(groups, single, cols) {
  return groups.map((g) => {
    const rows = [...g.rows].sort((a, b) => (b.last_at ?? -1) - (a.last_at ?? -1));
    const cards = single || rows.length <= cols ? rows : rows.slice(0, Math.max(1, cols - 1));
    return { ...g, cards, more: rows.length - cards.length };
  });
}

/// 미리보기 다시 읽을 때 — 대화 요약의 마지막 글 시각 · 상태 · 앞부분이 바뀌었을 때. 마지막이 아닌 글의 변경은
/// 변경 신호(그 대화) · 개요에 다시 들어옴 · 탭 고름이 다시 읽게 한다(UI `home.js` `renderHomePreview`).
export function previewKey(c) {
  return JSON.stringify([c.id, c.last_at ?? null, c.last_state ?? null, preview(c.last?.text)]);
}

/// 변경 신호 → 미리보기를 다시 읽을 대화 — `null` = 전부(빈 · 배열 아님 · id 없는 대화 · 대화 아닌 자원), 아니면 대화 id 들.
export function previewStaleOf(res) {
  if (!Array.isArray(res) || !res.length || res.some((r) => r?.kind !== "conv" || r.id == null)) return null;
  return res.map((r) => r.id);
}

/// 미리보기 말풍선 — 마지막 `n` 개, 요약 글(`context`) 뺌. 쪽 = `bubbleSide`, 이름 = `authorLabel`, 본문 = 글자만(`preview`, 비면 상태 글자).
/// `tone` = `error`(오류 · 답 미전달) · `running`(실행 대기 · 실행 중) · `agent` · `human`.
export function previewBubbles(conv, ms, peerName, n = PREVIEW_MSGS) {
  return ms
    .filter((m) => m.author !== "context")
    .slice(-n)
    .map((m) => ({
      id: m.id,
      side: bubbleSide(m),
      who: authorLabel(conv, m, peerName),
      text: preview(m.text) || stateText(m.state),
      tone: ["error", "undelivered"].includes(m.state)
        ? "error"
        : ["queued", "running"].includes(m.state)
          ? "running"
          : m.author === "agent"
            ? "agent"
            : "human",
    }));
}
