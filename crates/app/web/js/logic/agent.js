// 내 에이전트 — 실행 도구 · 옵션 블록 · 셸 확인 · 저장 · 삭제 창 · 색.
import { josa } from "./format.js";

/// 모든 실행의 도구 — 탐색(읽기 전용) 전부(an_core `READ_ONLY_TOOLS` 와 같다 — 프로필마다 고르지 않는다).
export const READ_ONLY_TOOLS = ["Read", "Grep", "Glob"];
/// Claude 실행에 미리 허용하는 웹 읽기 도구 · 늘 막는 편집 도구(an_core `agents::WEB_TOOLS` · `EDIT_TOOLS` 와 같다).
export const WEB_TOOLS = ["WebFetch", "WebSearch"];
export const EDIT_TOOLS = ["Edit", "Write", "NotebookEdit"];
/// 에이전트 화면의 도구 안내 한 줄.
export const TOOLS_NOTE = `읽기 도구 항상 허용: ${[...READ_ONLY_TOOLS, ...WEB_TOOLS].join(" · ")} · 편집 도구 막음(${EDIT_TOOLS.join(" · ")}) · 그 밖은 이 PC Claude 설정의 허용 규칙`;
/// Codex 옵션 안내.
export const SKIP_GIT_HINT = "기본 켬 — --skip-git-repo-check 로 실행";
export const CODEX_READ_NOTE = "읽기 범위: 읽기 전용 샌드박스 · 셸로 읽음(폴더 밖 읽기는 지시로만 막음)";
/// 실행 도구(an_core `agents::ENGINES` 와 같다). 등록 때 하나 고르고 바꾸지 않는다.
export const ENGINES = ["claude", "codex"];
/// 실행 도구 → 보이는 이름(모르는 값 · 옛 응답 = Claude Code).
export function engineLabel(e) {
  return e === "codex" ? "Codex" : "Claude Code";
}

/// 등록 2단계 · 상세의 좌우 옵션 블록 — 고른 도구의 블록만 켜고, 다른 블록은 비활성 + 이유 문구.
export function agentEngineBlocks(engine) {
  const block = (mine) => ({
    disabled: engine !== mine,
    note: engine === mine ? "" : engine ? `${engineLabel(engine)} 에이전트는 쓰지 않음` : "실행 도구를 고르면 열림",
  });
  return { claude: block("claude"), codex: block("codex") };
}
/// "상위 한단계 탐색 허용" 도움말 — 등록 칸(index.html title)과 같은 글.
export const PARENT_NOTE = "켜면 폴더 바로 위 폴더 아래 전체(옆 폴더 포함)를 읽고 답합니다. 위가 드라이브 루트 · 사용자 홈이거나 데이터 폴더를 품으면 열지 않습니다.";
/// "셸 명령 실행 허용" 도움말 — 등록 칸(index.html title)과 같은 글.
export const SHELL_NOTE = "켜면 이 PC 에 설정된 workflow(스크립트 · 빌드 · 테스트 등)를 셸(Bash · PowerShell) 명령으로 수행해 답합니다. 소스 · 문서 · 설정의 삭제 · 편집 등 이 PC 를 바꾸는 행동은 허용하지 않습니다(workflow 가 만드는 빌드 · 테스트 산출물은 예외). 셸에는 폴더 제한이 적용되지 않아 켤 때 위험 안내를 확인합니다.";
/// 에이전트 상세 저장 완료 창 글.
export function agentSavedText(name) {
  return `에이전트 ${name}${josa(name, "을", "를")} 저장했습니다.`;
}

/// 에이전트 상세 저장 실패 창 글.
export function agentSaveFailText(name, reason) {
  return `에이전트 ${name} 저장 실패: ${reason}`;
}

/// 에이전트 삭제 확인 창 글 — 지우면 이어 쓰던 세션도 비운다(같은 이름으로 다시 등록해도 이전 맥락을 잇지 않는다).
export function agentDeleteAsk(name) {
  return `에이전트 ${name}${josa(name, "을", "를")} 삭제할까요?\n\n이어 쓰던 세션도 끊겨, 같은 이름으로 다시 등록해도 이전 맥락을 잇지 않습니다.`;
}

/// 셸을 켤 때 한 번 보이는 위험 확인 — 셸에 실제로 열리는 것을 적는다.
export const SHELL_WARN = [
  "셸 명령 실행은 이 PC 에 설정된 workflow(스크립트 · 빌드 · 테스트 등)를 수행하기 위한 기능입니다.",
  "소스 · 문서 · 설정 파일의 삭제 · 편집, 저장소 변경 등 이 PC 를 바꾸는 행동은 허용하지 않습니다(에이전트 규칙 — workflow 가 만드는 빌드 · 테스트 산출물 · 로그는 예외).",
  "",
  "그래도 켜면 다음 위험이 있습니다.",
  "",
  "• 연결된 상대 누구나(멘션 포함) 사람 승인 없이 이 PC 에서 명령을 실행하게 할 수 있습니다.",
  "• 셸에는 폴더 · 상위 읽기 제한이 적용되지 않습니다 — 이 PC 사용자 권한으로 어디든 읽고 쓸 수 있습니다.",
  "• coworkAgent 데이터 폴더(신원 키 · 대화 DB), Claude 인증 정보(~/.claude), ~/.ssh 같은 비밀 파일을 읽어 답에 실을 수 있습니다.",
  "• '바꾸지 않는다' 규칙은 모델에게 주는 안내일 뿐 막는 장치가 아닙니다 — 원격 글에 숨긴 지시로 파일 수정 · 삭제 · 외부 전송이 일어날 수 있습니다.",
  "• 이 에이전트를 자동 왕복의 내 에이전트로 쓰면 상대 에이전트의 답이 이 PC 의 셸을 움직입니다.",
  "• 앱 환경 변수는 기본 변수만 넘깁니다 — 다만 프록시 주소(HTTP(S)_PROXY)에 계정 · 비밀번호가 들어 있으면 셸에 보입니다. 파일 · 저장소 속 비밀은 막지 못합니다.",
  "",
  "믿는 상대만 연결된 PC 에서 켜세요. 켜시겠습니까?",
].join("\n");
/// 상세 체크박스 옆 한 줄 설명(긴 설명은 체크박스 `title`) · 셸을 켠 에이전트 상세의 경고 줄(`SHELL_ON`).
export const PARENT_HINT = "켜면 바로 위 폴더 아래 전체를 읽음";
export const SHELL_HINT = "켜면 workflow 수행용 셸 사용(위험 확인)";
export const SHELL_ON = "켜짐 — 연결된 상대가 이 PC 에서 셸 명령을 실행할 수 있습니다(폴더 제한 없음 · 사람 승인 없음).";
/// 셸 위험 확인을 띄울까 — 끔 → 켬일 때만(등록은 `before` = 끔, 켠 채 다른 칸만 고치면 묻지 않는다).
export function shellConfirmNeeded(before, after) {
  return !before && !!after;
}
/// 에이전트 상세 저장 본문 — 셸 · 상위 읽기는 읽어 온 값(`a`)에서 바꿨을 때만 싣는다. 다른 창에서 끈 셸을
/// 오래 열린 상세의 일반 편집(폴더 · 추가 지시)이 확인 없이 되살리지 않게(서버는 빠진 칸을 그대로 둔다).
export function agentPatch(a, form) {
  const body = { folder: form.folder, prompt: form.prompt };
  if (!!form.parent_read !== !!a.parent_read) body.parent_read = !!form.parent_read;
  if (!!form.shell !== !!a.shell) body.shell = !!form.shell;
  if (form.color !== undefined && form.color !== agentColor(a.color)) body.color = form.color; // 아이콘 색
  if (form.skip_git !== undefined && !!form.skip_git !== (a.skip_git ?? true)) body.skip_git = !!form.skip_git; // Codex Git 확인 건너뜀
  return body;
}
/// 에이전트 아이콘 색(an_core `agents::ICON_COLORS` 와 같다, `app.css` 색 클래스 이름 그대로). 첫 값이 기본.
export const AGENT_COLORS = ["agent-tile", "peer-1", "peer-2", "peer-3", "peer-4", "peer-5"];
/// 저장된 색 → 그릴 색(옛 응답 · 모르는 값 = 기본).
export function agentColor(c) {
  return AGENT_COLORS.includes(c) ? c : AGENT_COLORS[0];
}
