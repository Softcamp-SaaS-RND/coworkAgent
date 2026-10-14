//! coworkAgent 앱 — 로컬 웹UI 서버 · 단일 인스턴스 핸드오프 · 트레이 · 감시 프로세스 · 자동 시작 · 내 정보·피어 · 에이전트·대화 API · AI 에이전트용 CLI.
pub mod autostart;
pub mod browser;
pub mod chat_api;
pub mod cli;
pub mod handoff;
pub mod notify;
pub mod peers_api;
pub mod server;
pub mod tray;
pub mod watchdog;
