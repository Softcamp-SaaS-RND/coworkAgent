// 설정 — 메뉴 나무 · 초안 · 진단 정보 · 모델 · effort.
import { fileSize } from "./format.js";

/// 설정 화면 왼쪽 메뉴(평평한 8개) — 항목 `id` = 그 화면(`<section id>`), `icon` = `.ico-{이름}`. 고르면 그 화면만 보인다.
/// 내 정보는 프로필 메뉴 "사용자 정보" 팝업.
export function settingsTree() {
  return [
    { id: "conn", label: "연결", icon: "link" },
    { id: "groups", label: "그룹설정", icon: "users" },
    { id: "agents", label: "에이전트", icon: "bot" },
    { id: "run", label: "실행", icon: "play" },
    { id: "screen", label: "화면", icon: "monitor" },
    { id: "startup", label: "시작", icon: "power" },
    { id: "notify", label: "알림", icon: "bell" },
    { id: "debug", label: "디버깅", icon: "bug" },
  ];
}

/// 설정 해시의 옛 · 진입 이름 → 그 화면. `invite_make` = 초대 생성(새로 만들기가 단추를 누른다) · `invite_accept` = 초대 수락 칸 · `peers` = 사용자 표(옛 그룹설정 카드 —
/// 검색 결과 · 페어링 전 사람) · `agent_new` = 새 에이전트 폼(새로 만들기 "에이전트 등록").
export const SETTINGS_ALIAS = { invite_make: "conn", invite_accept: "conn", peers: "conn", agent_new: "agents" };

/// 내 정보 포트 줄(`GET /api/me`) — 웹UI 포트 · 피어 포트(고정 | 임의 — 잡은 프로세스) · 열린 피어 연결 수.
export function portText(me) {
  const holder = me.peer_port_holder;
  const peer = me.peer_port_fixed
    ? `${me.port}(고정)`
    : holder
      ? `${me.port}(임의 — 47100 사용 중: ${holder.name}, PID ${holder.pid})`
      : `${me.port}(임의)`;
  return { ui: me.ui_port != null ? String(me.ui_port) : "알 수 없음", peer, conns: String(me.open_connections ?? 0) };
}

/// 저장 전 값 꺼내기 — 초안은 그 id 의 주인(`owner` — 에이전트 등록 세대 `uid` · 상대 지문)과 함께 둔다. 주인이 다르면(지운 뒤 새 프로필이 같은 id · 같은 이름을 다시 씀) 버리고 없다고 본다.
export function draftOf(drafts, id, owner) {
  const d = drafts.get(id);
  if (d && d.owner !== owner) drafts.delete(id);
  return d?.owner === owner ? d : undefined;
}

/// 목록에 없는 id 의 저장 전 값을 버린다(지운 항목).
export function pruneDrafts(drafts, ids) {
  for (const id of drafts.keys()) if (!ids.includes(id)) drafts.delete(id);
}

/// 로그 줄 — `{유닉스 초} {수준} {나머지}` → `{at, level, rest}`(그 꼴이 아니면 `at`·`level` null, `rest` = 줄 그대로). `level` 을 주면 그 수준만.
export function logLines(lines, level = "") {
  return lines
    .map((l) => {
      const m = /^(\d{9,})\s+([A-Z]+)\s+(.*)$/.exec(l);
      return m ? { at: Number(m[1]), level: m[2], rest: m[3] } : { at: null, level: null, rest: l };
    })
    .filter((x) => !level || x.level === level);
}

/// 진단 정보 복사 글 — 버전 · OS · 경로 · 크기 · 포트 · 열린 연결 수. 신원 키 · 토큰 · 지문은 넣지 않는다(`me` 의 다른 칸을 읽지 않는다).
export function diagText(info, me) {
  const ports = me ? portText(me) : null;
  const file = (k) => info.files.find((f) => f.key === k);
  return [
    `coworkAgent ${info.version} (${info.os} ${info.arch})`,
    `데이터 폴더: ${info.data_dir}`,
    ...["db", "log"].map((k) => `${k === "db" ? "DB" : "로그"}: ${file(k)?.path ?? "?"} (${fileSize(file(k)?.size)})`),
    ...(ports ? [`웹UI 포트: ${ports.ui}`, `피어 포트: ${ports.peer}`, `열린 피어 연결: ${ports.conns}`] : []),
  ].join("\n");
}

/// 고를 수 있는 모델(an_core `run_opts::MODELS` 와 같다).
export const MODELS = ["opus", "sonnet", "haiku"];
/// `claude --effort` 값(an_core `run_opts::EFFORTS` 와 같다).
export const EFFORTS = ["low", "medium", "high", "xhigh", "max"];
