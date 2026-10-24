// 글 본문 — HTML 문서 판별 · 마크다운 블록 · 인라인.

/// HTML 문서로 보이는 본문(`<!doctype html` · `<html` 로 시작) — sandbox iframe 으로 그린다.
export function isHtmlDoc(text) {
  return /^\s*(<!doctype\s+html|<html[\s>])/i.test(text ?? "");
}

/// iframe `srcdoc` 앞에 붙이는 CSP — 스크립트 · 외부 리소스 차단, 인라인 스타일 · data: 그림만. `<base target="_top">` — 본문 링크를 눌러도
/// 프레임이 외부로 가지 않는다(top 이동은 sandbox 가 막는다).
export const HTML_CSP = "default-src 'none'; style-src 'unsafe-inline'; img-src data:";

export function htmlSrcdoc(text) {
  return `<meta http-equiv="Content-Security-Policy" content="${HTML_CSP}"><base target="_top">` + text;
}

/// 인라인 — `코드` · **굵게** · *기울임* · _기울임_. 그 밖(링크 · HTML 태그 포함)은 글자 그대로.
export function inlines(text) {
  const out = [];
  const re = /`([^`]+)`|\*\*([^*]+)\*\*|\*([^*\s][^*]*)\*|(?<![\p{L}\p{N}_])_([^_\s][^_]*)_(?![\p{L}\p{N}_])/gu;
  let at = 0;
  for (const m of text.matchAll(re)) {
    if (m.index > at) out.push({ t: "text", v: text.slice(at, m.index) });
    if (m[1] != null) out.push({ t: "code", v: m[1] });
    else if (m[2] != null) out.push({ t: "b", v: m[2] });
    else out.push({ t: "i", v: m[3] ?? m[4] });
    at = m.index + m[0].length;
  }
  if (at < text.length) out.push({ t: "text", v: text.slice(at) });
  return out;
}

export const ROW = /^\s*\|.*\|\s*$/;
export const RULE = /^\s*\|?\s*:?-{3,}/;
export const ITEM = /^\s*([-*+]|\d+[.)])\s+(.*)$/;

export function cells(line) {
  return line
    .trim()
    .replace(/^\|/, "")
    .replace(/\|$/, "")
    .split("|")
    .map((c) => inlines(c.trim()));
}

/// markdown → 블록 목록(제목 · 목록 · 코드 블록 · 인용 · 표 · 문단). 값은 글자뿐 — 그리는 쪽(UI `conv.js` `markdownEl`)이 textContent 로만 넣는다.
export function markdownBlocks(text) {
  const lines = (text ?? "").replace(/\r\n?/g, "\n").split("\n");
  const blocks = [];
  let para = [];
  const flush = () => {
    if (para.length) blocks.push({ type: "p", inlines: inlines(para.join(" ")) });
    para = [];
  };
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    let m;
    if (/^\s*```/.test(line)) {
      flush();
      const code = [];
      for (i++; i < lines.length && !/^\s*```/.test(lines[i]); i++) code.push(lines[i]);
      blocks.push({ type: "code", text: code.join("\n") });
    } else if ((m = /^(#{1,6})\s+(.*)$/.exec(line))) {
      flush();
      blocks.push({ type: "h", level: m[1].length, inlines: inlines(m[2].trim()) });
    } else if (ROW.test(line) && RULE.test(lines[i + 1] ?? "")) {
      flush();
      const head = cells(line);
      const rows = [];
      for (i += 2; i < lines.length && ROW.test(lines[i]); i++) rows.push(cells(lines[i]));
      i--;
      blocks.push({ type: "table", head, rows });
    } else if ((m = ITEM.exec(line))) {
      flush();
      const ordered = /\d/.test(m[1]);
      const items = [inlines(m[2])];
      for (let n; i + 1 < lines.length && (n = ITEM.exec(lines[i + 1])) && /\d/.test(n[1]) === ordered; i++) items.push(inlines(n[2]));
      blocks.push({ type: ordered ? "ol" : "ul", items });
    } else if ((m = /^\s*>\s?(.*)$/.exec(line))) {
      flush();
      const quote = [m[1]];
      for (; i + 1 < lines.length && /^\s*>/.test(lines[i + 1]); i++) quote.push(lines[i + 1].replace(/^\s*>\s?/, ""));
      blocks.push({ type: "quote", inlines: inlines(quote.join(" ")) });
    } else if (!line.trim()) {
      flush();
    } else {
      para.push(line.trim());
    }
  }
  flush();
  return blocks;
}
