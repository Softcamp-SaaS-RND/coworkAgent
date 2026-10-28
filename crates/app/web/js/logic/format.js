// 형식 — 상태 글 · 날짜 · 시각 · 기간 · 조사 · 짧은 이름 · 크기.

export const STATE_TEXT = {
  sent: "보냄",
  queued: "실행 대기",
  running: "실행 중",
  done: "완료",
  error: "오류",
  undelivered: "답 미전달",
};

/// 메시지 상태 → 문구(모르는 값은 그대로).
export function stateText(state) {
  return STATE_TEXT[state] ?? state ?? "";
}

/// 날짜 구분선 열쇠 — 지역 날짜(`YYYY-M-D`).
export function dayKey(ts) {
  const d = new Date(ts * 1000);
  return `${d.getFullYear()}-${d.getMonth() + 1}-${d.getDate()}`;
}

/// 날짜 글자 — `M.D`(올해) · `YYYY.M.D`.
export function dateText(ts, now = Date.now() / 1000) {
  const d = new Date(ts * 1000);
  const md = `${d.getMonth() + 1}.${d.getDate()}`;
  return d.getFullYear() === new Date(now * 1000).getFullYear() ? md : `${d.getFullYear()}.${md}`;
}

/// 날짜 구분선 글자 — 오늘 · 어제 · `M.D`(올해) · `YYYY.M.D`.
export function dayLabel(ts, now = Date.now() / 1000) {
  if (dayKey(ts) === dayKey(now)) return "오늘";
  if (dayKey(ts) === dayKey(now - 86400)) return "어제";
  return dateText(ts, now);
}

/// 말풍선 목록 사이 날짜 구분선 자리 — 글마다 `{day}`(새 날짜면 그 글자, 아니면 `null`).
export function dayBreaks(ms, now = Date.now() / 1000) {
  let prev = null;
  return ms.map((m) => {
    const k = dayKey(m.created_at);
    const day = k === prev ? null : dayLabel(m.created_at, now);
    prev = k;
    return day;
  });
}

/// 이 PC 에서 부르는 상대 이름 — 표시 이름(별칭)이 있으면 그것.
export function displayName(p) {
  return p.alias || p.name;
}

/// "이름 · PC" 의 이름 — 마지막 구분 앞까지(표시 이름에 " · " 가 있어도 자르지 않는다). 모르는 상대 = "상대".
export function shortName(full) {
  const s = full ?? "상대";
  const cut = s.lastIndexOf(" · ");
  return cut < 0 ? s : s.slice(0, cut);
}

/// 실행 시간 표기.
export function formatDuration(ms) {
  if (ms == null) return "";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}초`;
  const m = Math.floor(ms / 60000);
  const s = Math.round((ms % 60000) / 1000);
  return `${m}분 ${s}초`;
}

/// 날짜 표시방식(설정 `ui_date_format`) — 첫 값이 기본.
export const DATE_FORMATS = ["md", "relative", "ymd", "time"];
let dateFormat = DATE_FORMATS[0];

/// 날짜 표시방식을 바꾼다(모르는 값 = 기본). 바뀌었으면 `true` — 이미 그린 시각을 다시 그릴 때.
export function setDateFormat(f) {
  const next = DATE_FORMATS.includes(f) ? f : DATE_FORMATS[0];
  const changed = next !== dateFormat;
  dateFormat = next;
  return changed;
}

/// 그린 시각이 기대는 값 — 날짜 표시방식 · 지역 날짜(오늘 · 어제 · 연도 생략은 날이 바뀌면 달라진다). 다시 그리기 서명에 붙여
/// 자정 · 해 넘김 · 방식 바꿈에 이미 그린 시각을 다시 그린다.
export function timeKey(now = Date.now() / 1000) {
  return `${dateFormat}|${dayKey(now)}`;
}

/// 유닉스 초 → 날짜 표시방식에 맞춘 지역 시각 — md `M.D HH:MM` · relative `오늘 | 어제 HH:MM`(그 전은 md) · ymd `YYYY-MM-DD HH:MM` · time `HH:MM`.
/// md · relative 는 다른 해면 `YYYY.M.D`. `seconds` = `:SS` 붙임(실행 상세).
export function formatTime(unix, { seconds = false, fmt = dateFormat, now = Date.now() / 1000 } = {}) {
  if (unix == null) return "";
  const d = new Date(unix * 1000);
  const two = (n) => String(n).padStart(2, "0");
  const clock = [d.getHours(), d.getMinutes(), ...(seconds ? [d.getSeconds()] : [])].map(two).join(":");
  const date = {
    md: () => dateText(unix, now),
    relative: () => dayLabel(unix, now),
    ymd: () => `${d.getFullYear()}-${two(d.getMonth() + 1)}-${two(d.getDate())}`,
    time: () => "",
  }[fmt]();
  return date ? `${date} ${clock}` : clock;
}

/// 유닉스 초 → 지역 시각 "HH:MM:SS"(디버깅 로그 — 날짜는 부르는 쪽이 붙인다).
export function formatClock(unix) {
  if (unix == null) return "";
  const d = new Date(unix * 1000);
  return [d.getHours(), d.getMinutes(), d.getSeconds()].map((n) => String(n).padStart(2, "0")).join(":");
}

/// 받침에 맞춘 조사 — 한글로 끝나면 받침 있음 `withB` · 없음 `withoutB`, 그 밖(영문 · 숫자 등)은 `withB(withoutB)`.
export function josa(word, withB, withoutB) {
  const last = (word ?? "").at(-1) ?? "";
  const code = last.charCodeAt(0) - 0xac00;
  if (!(code >= 0 && code <= 11171)) return `${withB}(${withoutB})`; // 빈 글자(NaN)도
  return code % 28 ? withB : withoutB;
}

/// 목록에 보일 본문 앞부분 — 줄바꿈은 공백으로, 120자까지.
export function preview(text) {
  const one = (text ?? "").replace(/\s+/g, " ").trim();
  return one.length > 120 ? one.slice(0, 120) + "…" : one;
}

/// 파일 크기(디버깅) — 바이트 → "123 B" · "12.4 KB" · "1.2 MB"(1024 단위), 없으면 "없음".
export function fileSize(n) {
  if (n == null) return "없음";
  if (n < 1024) return `${n} B`;
  const [v, u] = n < 1024 * 1024 ? [n / 1024, "KB"] : [n / 1024 / 1024, "MB"];
  return `${v.toFixed(1)} ${u}`;
}
