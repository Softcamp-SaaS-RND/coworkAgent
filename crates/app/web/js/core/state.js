// 화면 공유 상태 — 모듈들은 이 객체의 속성만 바꾼다.
import { Latest } from "../logic/sync.js";

export const state = {
  me: null,
  peers: [],
  peerNames: new Map(),
  agents: [],
  convs: [],
  settings: {},
  railPeer: null, // 레일에서 고른 상대(null = 나 = 전부)
  railGroup: null, // 레일에서 고른 그룹(고르면 railPeer = null)
  groups: [], // 레일 그룹(`GET /api/groups` — 이 PC 에만, 레일 순서)
  current: null, // 열린 대화(요약)
  unseen: 0, // 안 본 내 활동 — 고른 타일 범위(사이드바 내비)
  unseenAll: 0, // 안 본 내 활동 — 전부(종 점)
  reqWaiting: 0, // 받은 요청 중 대기 — 고른 타일 범위(사이드바 '요청')
  waitingBy: new Map(), // 받은 요청 중 대기 — 상대별(개요 요약)
  folded: new Set(), // 접은 상대별 절 열쇠 `{화면}:{상대}`(설정 `ui_folded` — 시작할 때 읽는다)
  homeView: "preview", // 개요 보기 — `preview` · `list`(설정 `ui_home_view` — 시작할 때 읽는다)
  previewCols: 4, // 개요 미리보기 — 카드 한 줄 열 수(그린 뒤 실제 격자에서 잰다)
  previewMsgs: new Map(), // 개요 미리보기 — 대화 id → `{key, ms, seq}` 마지막으로 받은 글(`previewKey` · 받은 차례)
  previewStale: new Set(), // 개요 미리보기 — 다시 읽을 대화 id(변경 신호 · 읽기 실패)
  previewAll: true, // 개요 미리보기 — 카드의 대화를 전부 다시 읽기(개요에 들어옴 · 보기 바꿈 · 전체 변경 신호)
  reqBox: null, // '요청' 화면 — `null` = 개요, 탭 = 받은(`theirs`) · 보낸(`mine`)
  peerAgents: new Map(), // 상대 id → 그 PC 의 에이전트 이름(`null` = 받지 못함) — 타일을 고를 때 받는다(원격 왕복이라 새로 고침마다 묻지 않는다)
  detailOpen: window.innerWidth >= 1280, // 실행 상세 — 넓은 창이면 열고 시작(저장하지 않는다)
  focusAfter: null, // 다음 화면을 그린 뒤 초점을 줄 칸 id
  peerHeads: [], // 주고받은 메시지 상대 절 머리
  peerPages: new Map(), // 상대 id → 펼친 절에 받은 글(20건씩)
  peerLoads: new Map(), // 상대 id → 그 상대 글 받기 차례(한 번에 하나)
  actKind: "all", // 내 활동 거르기 탭 — `ACT_KINDS` 의 `kind`
  actFresh: new Set(), // 내 활동을 열 때 안 본 항목 — 본 것으로 표시한 뒤 다시 그려도 강조를 남긴다
  msgs: [], // 열린 대화의 글(`msgsOf` = 그 대화 id) — 머리글 부제 · 실행 상세가 쓴다
  msgsOf: null,
  pickMsg: null, // 실행 상세에서 고른 글(요청 또는 그 답) — 없으면 마지막 요청
  focusMsg: null, // 대화를 그린 뒤 갈 글(`#c/{대화}/m/{글}`) — 늦게 그려져도 그때 간다
  mention: null, // 입력창 멘션 후보 — `{start, items, active}`(열려 있을 때만)
  mentionWarned: null, // 후보와 맞지 않는 멘션이라 한 번 막은 글 — 같은 글을 다시 보내면 멘션 없이 보낸다
  netFails: 0, // 연달아 닿지 못한 요청 수(앱과 연결 끊김)
};
export const latest = new Latest();
export const previewLatest = new Latest(); // 개요 미리보기 — 늦게 온 글 읽기는 버린다
